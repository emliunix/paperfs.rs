#!/usr/bin/env python3
"""Execute the frozen W5 matrix through Magpie without exposing credentials."""
from __future__ import annotations
import concurrent.futures
import hashlib
import json
import os
import threading
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
RUN = ROOT / "_runs/2026-11-19/docs-eval/r3"
VARIANTS = ("A", "B", "C")


def utcnow() -> str:
    return datetime.now(timezone.utc).isoformat()


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_key() -> str:
    env_path = Path.home() / ".dsh/.env"
    for line in env_path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or "=" not in stripped:
            continue
        name, value = stripped.split("=", 1)
        name = name.strip().removeprefix("export ").strip()
        if name == "MAGPIE_GATEWAY_KEY":
            value = value.strip()
            if len(value) >= 2 and value[0] == value[-1] and value[0] in ("'", '"'):
                value = value[1:-1]
            if not value:
                break
            return value
    raise RuntimeError("MAGPIE_GATEWAY_KEY is missing or empty in ~/.dsh/.env")


def verify_freeze(config: dict) -> None:
    manifest = RUN / "inputs.sha256"
    if sha_bytes(manifest.read_bytes()) != config["run_input_manifest_sha256"]:
        raise RuntimeError("Run input manifest hash mismatch")
    for line in manifest.read_text(encoding="utf-8").splitlines():
        expected, rel = line.split("  ", 1)
        actual = sha_bytes((ROOT / rel).read_bytes())
        if actual != expected:
            raise RuntimeError("Frozen input hash mismatch: " + rel)


def request(url: str, key: str, payload: dict | None, timeout: float) -> tuple[int, bytes]:
    data = None if payload is None else json.dumps(payload, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    headers = {"Authorization": "Bearer " + key, "Accept": "application/json"}
    if data is not None:
        headers["Content-Type"] = "application/json"
    req = urllib.request.Request(url, data=data, headers=headers, method="POST" if data is not None else "GET")
    try:
        with urllib.request.urlopen(req, timeout=timeout) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read()


def preflight(config: dict, key: str) -> None:
    url = config["execution"]["base_url"].rstrip("/") + "/models"
    started = time.monotonic()
    try:
        status, body = request(url, key, None, config["execution"]["preflight_timeout_seconds"])
        parsed = json.loads(body.decode("utf-8"))
        if status != 200:
            raise RuntimeError("Magpie preflight returned HTTP " + str(status) + ": " + body.decode("utf-8", "replace")[:1000])
        models = [item for item in parsed.get("data", []) if isinstance(item, dict)] if isinstance(parsed, dict) else []
        model_ids = [item.get("id") for item in models]
        selected = next((item for item in models if item.get("id") == config["execution"]["model"]), {})
        requested_effort = config["execution"]["decoding"]["reasoning_effort"]
        supported_efforts = [item.get("effort") for item in selected.get("supported_reasoning_levels", []) if isinstance(item, dict)]
        record = {"checked_at": utcnow(), "url": url, "http_status": status, "latency_ms": round((time.monotonic() - started) * 1000, 3), "requested_model_present": config["execution"]["model"] in model_ids, "model_count": len(model_ids), "requested_reasoning_effort": requested_effort, "supported_reasoning_efforts": supported_efforts, "reasoning_effort_supported": requested_effort in supported_efforts, "route_max_output_tokens": selected.get("max_output_tokens"), "configured_max_tokens": config["execution"]["decoding"]["max_tokens"]}
        (RUN / "preflight.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        if not record["requested_model_present"]:
            raise RuntimeError("Requested model is absent from Magpie /models")
        if not record["reasoning_effort_supported"]:
            raise RuntimeError("Requested reasoning effort is absent from Magpie model metadata")
        if not isinstance(selected.get("max_output_tokens"), int) or config["execution"]["decoding"]["max_tokens"] > selected["max_output_tokens"]:
            raise RuntimeError("Configured max_tokens exceeds model cap")
    except Exception as exc:
        record = {"checked_at": utcnow(), "url": url, "status": "unreachable_or_invalid", "error": type(exc).__name__ + ": " + str(exc)}
        (RUN / "preflight.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        (RUN / "report.md").write_text("# W5.4 evaluation not run\n\nMagpie endpoint preflight failed; no matrix predictions were attempted.\n\n" + record["error"] + "\n", encoding="utf-8")
        raise


def main() -> None:
    config = json.loads((RUN / "config.json").read_text(encoding="utf-8"))
    verify_freeze(config)
    if not config.get("run_permitted", True):
        raise RuntimeError(config.get("abort_reason") or "Frozen configuration does not permit execution")
    existing = list((RUN / "raw").glob("*/*.json"))
    if existing:
        raise RuntimeError("Raw directory is not empty; refusing to overwrite or resume a frozen run")
    key = load_key()
    preflight(config, key)
    common = (RUN / "common-prompt.txt").read_bytes()
    variant_a = (RUN / "variant-a.txt").read_bytes()
    variant_b = (RUN / "AGENTS.md").read_bytes()
    documentation = {"A": variant_a, "B": variant_b, "C": variant_a + variant_b}
    if documentation["C"] != documentation["A"] + documentation["B"]:
        raise RuntimeError("Variant C identity assertion failed")
    seeds = {}
    for path in sorted((ROOT / "knowledge-system/eval-seeds").glob("*.json")):
        seed = json.loads(path.read_text(encoding="utf-8"))
        seeds[seed["scenario_id"]] = seed
    tasks = [(scenario_id, variant) for scenario_id in config["scenario_order"] for variant in VARIANTS]
    base_url = config["execution"]["base_url"].rstrip("/") + "/chat/completions"
    max_attempts = 1 + config["execution"]["retry_policy"]["max_retries"]
    timeout = config["execution"]["timeout_seconds"]
    failure_log = RUN / "failures.jsonl"
    failure_log.write_text("", encoding="utf-8")
    stop_event = threading.Event()
    max_prompt_bytes = max(len(common) + len(documentation[variant]) + 4096 for variant in VARIANTS)
    if max_prompt_bytes + config["execution"]["decoding"]["max_tokens"] > config["execution"]["context_window_tokens"]:
        raise RuntimeError("Conservative prompt+output budget exceeds context window")

    def execute(task: tuple[str, str]) -> dict:
        scenario_id, variant = task
        seed = seeds[scenario_id]
        scenario_payload = {"scenario_id": scenario_id, "situation": seed["situation"]}
        prompt = common + b"\n\n### Documentation for variant " + variant.encode("ascii") + b"\n\n" + documentation[variant] + b"\n\n### Scenario payload\n\n" + json.dumps(scenario_payload, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        payload = {
            "model": config["execution"]["model"],
            "temperature": config["execution"]["decoding"]["temperature"],
            "max_tokens": config["execution"]["decoding"]["max_tokens"],
            "reasoning_effort": config["execution"]["decoding"]["reasoning_effort"],
            "stream": False,
            "messages": [{"role": "user", "content": prompt.decode("utf-8")}],
        }
        request_bytes = json.dumps(payload, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        record = {
            "schema_version": "docs-eval-raw-v1",
            "run_id": config["run_id"],
            "scenario_id": scenario_id,
            "variant": variant,
            "model": config["execution"]["model"],
            "provider": "magpie",
            "started_at": utcnow(),
            "request_sha256": sha_bytes(request_bytes),
            "prompt_bytes": len(prompt),
            "attempts": [],
            "status": "failed",
            "response": None,
            "content": None,
            "error": None,
        }
        if stop_event.is_set():
            record["error"] = {"class": "transport_blocker_not_attempted", "message": "Not attempted after persistent HTTP 502 transport blocker"}
            record["completed_at"] = utcnow()
            destination = RUN / "raw" / scenario_id
            destination.mkdir(parents=True, exist_ok=True)
            (destination / (variant + ".json")).write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
            return {"scenario_id": scenario_id, "variant": variant, "status": "failed", "attempts": 0, "error_class": "transport_blocker_not_attempted"}
        for attempt in range(1, max_attempts + 1):
            started = time.monotonic()
            attempt_record = {"attempt": attempt, "started_at": utcnow()}
            try:
                status, body = request(base_url, key, payload, timeout)
                body_text = body.decode("utf-8", "replace")
                attempt_record.update({"http_status": status, "latency_ms": round((time.monotonic() - started) * 1000, 3), "body_text": body_text})
                try:
                    parsed = json.loads(body_text)
                    attempt_record["response"] = parsed
                except json.JSONDecodeError as exc:
                    parsed = None
                    attempt_record["body_json_error"] = str(exc)
                if status != 200:
                    error_class = "http_502" if status == 502 else "http_429" if status == 429 else "http_5xx" if status >= 500 else "http_4xx"
                    attempt_record["error"] = {"class": error_class, "message": "Magpie returned HTTP " + str(status), "body_empty": not bool(body_text)}
                elif not isinstance(parsed, dict):
                    attempt_record["error"] = {"class": "invalid_http_json", "message": attempt_record.get("body_json_error", "HTTP 200 body is not a JSON object")}
                else:
                    choices = parsed.get("choices")
                    if isinstance(choices, list) and choices and isinstance(choices[0], dict):
                        choice = choices[0]
                        message = choice.get("message")
                        content = message.get("content") if isinstance(message, dict) else None
                        reasoning = message.get("reasoning_content") if isinstance(message, dict) else None
                        finish_reason = choice.get("finish_reason")
                        usage = parsed.get("usage") if isinstance(parsed.get("usage"), dict) else {}
                        details = usage.get("completion_tokens_details") if isinstance(usage.get("completion_tokens_details"), dict) else {}
                        attempt_record.update({"finish_reason": finish_reason, "content_chars": len(content) if isinstance(content, str) else None, "reasoning_chars": len(reasoning) if isinstance(reasoning, str) else None, "reasoning_tokens": details.get("reasoning_tokens", "unreported")})
                        if isinstance(content, str) and content:
                            record["status"] = "completed"
                            record["response"] = parsed
                            record["content"] = content
                            record["finish_reason"] = finish_reason
                            if finish_reason == "length":
                                record["budget_exhaustion"] = True
                                attempt_record["warning"] = {"class": "budget_exhaustion_truncated_content", "message": "Output budget exhausted with nonempty truncated content"}
                            record["attempts"].append(attempt_record)
                            break
                        if finish_reason == "length":
                            attempt_record["error"] = {"class": "budget_exhaustion_empty_content", "message": "Output budget exhausted before nonempty content; inspect content/reasoning split", "content_chars": len(content) if isinstance(content, str) else None, "reasoning_chars": len(reasoning) if isinstance(reasoning, str) else None, "reasoning_tokens": details.get("reasoning_tokens", "unreported")}
                        elif isinstance(content, str):
                            attempt_record["error"] = {"class": "empty_content", "message": "HTTP 200 response has empty content without length finish reason", "finish_reason": finish_reason}
                        else:
                            attempt_record["error"] = {"class": "missing_content", "message": "HTTP 200 response has no string choices[0].message.content", "finish_reason": finish_reason}
                    else:
                        attempt_record["error"] = {"class": "missing_choice", "message": "HTTP 200 response has no choices[0]"}
            except Exception as exc:
                error_class = "timeout" if isinstance(exc, TimeoutError) or "timed out" in str(exc).lower() else "network_error"
                attempt_record.update({"latency_ms": round((time.monotonic() - started) * 1000, 3), "error": {"class": error_class, "message": type(exc).__name__ + ": " + str(exc)}})
            record["attempts"].append(attempt_record)
            error_class = attempt_record.get("error", {}).get("class")
            retryable = error_class in {"network_error", "timeout", "http_429", "http_502", "http_5xx"}
            if attempt < max_attempts and retryable:
                time.sleep(config["execution"]["retry_policy"]["backoff_seconds"] * attempt)
            else:
                break
        record["completed_at"] = utcnow()
        if record["status"] != "completed":
            last = record["attempts"][-1] if record["attempts"] else {}
            record["error"] = last.get("error", {"class": "unknown_failure", "message": "No response"})
            if record["error"].get("class") == "http_502":
                stop_event.set()
            with failure_log.open("a", encoding="utf-8") as handle:
                handle.write(json.dumps({"scenario_id": scenario_id, "variant": variant, "error": record["error"], "attempts": len(record["attempts"])}, ensure_ascii=False) + "\n")
        destination = RUN / "raw" / scenario_id
        destination.mkdir(parents=True, exist_ok=True)
        (destination / (variant + ".json")).write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        return {"scenario_id": scenario_id, "variant": variant, "status": record["status"], "attempts": len(record["attempts"]), "error_class": record.get("error", {}).get("class") if isinstance(record.get("error"), dict) else None, "finish_reason": record.get("finish_reason"), "budget_exhaustion": record.get("budget_exhaustion", False)}

    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=config["execution"]["concurrency"]) as pool:
        futures = {pool.submit(execute, task): task for task in tasks}
        for future in concurrent.futures.as_completed(futures):
            task = futures[future]
            try:
                result = future.result()
            except Exception as exc:
                raise RuntimeError("Unhandled matrix execution failure for " + str(task) + ": " + str(exc)) from exc
            results.append(result)
            print(json.dumps(result), flush=True)
    summary = {
        "run_id": config["run_id"],
        "completed_at": utcnow(),
        "matrix_cases": len(results),
        "completed": sum(item["status"] == "completed" for item in results),
        "failed": sum(item["status"] != "completed" for item in results),
        "http_post_attempts": sum(item["attempts"] for item in results),
        "persistent_http_502_blocker": stop_event.is_set(),
        "results": sorted(results, key=lambda item: (item["scenario_id"], item["variant"])),
    }
    (RUN / "execution.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in summary.items() if key != "results"}), flush=True)


if __name__ == "__main__":
    main()
