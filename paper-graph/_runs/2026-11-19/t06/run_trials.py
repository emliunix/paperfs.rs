#!/usr/bin/env python3
"""T0.6 direct-gateway runner; subagent delegation is unavailable at depth 1."""
import argparse
import concurrent.futures
import datetime as dt
import json
import os
from pathlib import Path
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parent
CASES = {
    "a": ROOT / "cases/a-theorem-definition-citation.json",
    "b": ROOT / "cases/b-five-relations-evidence.json",
    "c": ROOT / "cases/c-ambiguous-mention-use.json",
}
ROUTES = {
    "deepseek-flash": {
        "provider": "magpie", "model": "deepseek/deepseek-flash",
        "url": "http://127.0.0.1:3425/v1/chat/completions", "wire": "openai",
    },
    "glm-5.3": {
        "provider": "magpie-anthropic", "model": "zcode/GLM-5.3",
        "url": "http://127.0.0.1:3425/v1/messages", "wire": "anthropic",
    },
}

def now():
    return dt.datetime.now(dt.timezone.utc).astimezone().isoformat()

def load_key():
    key = os.environ.get("MAGPIE_GATEWAY_KEY")
    if key:
        return key
    env = Path.home() / ".dsh/.env"
    for line in env.read_text().splitlines():
        if line.startswith("MAGPIE_GATEWAY_KEY="):
            return line.split("=", 1)[1].strip().strip('"').strip("'")
    raise RuntimeError("MAGPIE_GATEWAY_KEY not found")

def prompt_for(case):
    return (
        "You are executing one structured-output reliability trial. Return only JSON.\n"
        "Return one JSON value conforming exactly to the supplied JSON Schema. "
        "Do not use Markdown fences, commentary, or trailing text.\n"
        + case["task"] + "\n" + case["offset_and_locator_rules"] + "\n"
        + "JSON Schema:\n" + json.dumps(case["json_schema"], ensure_ascii=False)
        + "\nExcerpt text (paper 3154, pages 1 and 20):\n" + case["excerpt_text"]
    )

def call(route, prompt, key, timeout):
    if route["wire"] == "openai":
        body = {
            "model": route["model"],
            "messages": [{"role": "user", "content": prompt}],
            "reasoning_effort": "low", "max_tokens": 16384,
        }
        headers = {"Authorization": "Bearer " + key, "Content-Type": "application/json"}
    else:
        body = {
            "model": route["model"], "max_tokens": 16384,
            "messages": [{"role": "user", "content": prompt}],
            "output_config": {"effort": "low"},
        }
        headers = {
            "x-api-key": key, "anthropic-version": "2023-06-01",
            "Content-Type": "application/json",
        }
    req = urllib.request.Request(
        route["url"], data=json.dumps(body, ensure_ascii=False).encode("utf-8"),
        headers=headers, method="POST",
    )
    started = time.monotonic()
    rec = {"started_at": now(), "timeout_ms": timeout * 1000}
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read().decode("utf-8", errors="replace")
            rec["http_status"] = resp.status
        rec["latency_ms"] = round((time.monotonic() - started) * 1000)
        rec["raw_response_body"] = raw
        payload = json.loads(raw)
        if route["wire"] == "openai":
            choice = (payload.get("choices") or [{}])[0]
            rec["response_text"] = (choice.get("message") or {}).get("content")
            rec["finish_reason"] = choice.get("finish_reason")
        else:
            rec["response_text"] = "".join(
                b.get("text", "") for b in payload.get("content", []) if b.get("type") == "text"
            )
            rec["finish_reason"] = payload.get("stop_reason")
        rec["usage"] = payload.get("usage")
        rec["served_model"] = payload.get("model")
        rec["transport_ok"] = True
    except urllib.error.HTTPError as e:
        rec["latency_ms"] = round((time.monotonic() - started) * 1000)
        rec["http_status"] = e.code
        rec["transport_ok"] = False
        rec["failure_type"] = "http_error"
        rec["error_verbatim"] = e.read().decode("utf-8", errors="replace") or str(e)
    except Exception as e:
        rec["latency_ms"] = round((time.monotonic() - started) * 1000)
        rec["transport_ok"] = False
        rec["failure_type"] = "timeout" if isinstance(e, TimeoutError) else "transport_error"
        rec["error_verbatim"] = f"{type(e).__name__}: {e}"
    rec["finished_at"] = now()
    return rec

def run_trial(route_name, case_key, trial, key, timeout, max_retries, force=False):
    route = ROUTES[route_name]
    case_path = CASES[case_key]
    case = json.loads(case_path.read_text())
    outdir = ROOT / "attempts" / route_name / case["case_id"]
    outdir.mkdir(parents=True, exist_ok=True)
    outfile = outdir / f"trial-{trial}.json"
    if outfile.exists() and not force:
        return {"skipped_existing": str(outfile)}
    prompt = prompt_for(case)
    record = {
        "schema_version": "t06-attempt-log-v1",
        "route": route_name, "provider": route["provider"], "model": route["model"],
        "case_id": case["case_id"], "case_file": case_path.name, "trial": trial,
        "effort": "low", "transport": "direct_gateway_not_subagent_depth_restricted",
        "max_retries": max_retries, "attempts": [],
    }
    for attempt in range(max_retries + 1):
        rec = call(route, prompt, key, timeout)
        rec["attempt"] = attempt
        text = rec.get("response_text")
        rec["empty"] = rec.get("transport_ok") is True and not (text or "").strip()
        rec["truncated"] = rec.get("finish_reason") in ("length", "max_tokens")
        retryable = (
            not rec.get("transport_ok") or rec["empty"] or rec["truncated"]
        )
        rec["retry_scheduled"] = bool(retryable and attempt < max_retries)
        record["attempts"].append(rec)
        outfile.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n")
        if not rec["retry_scheduled"]:
            break
        time.sleep(2 ** attempt)
    return {"file": str(outfile), "attempts": len(record["attempts"])}

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--only", help="route:case:trial, e.g. deepseek-flash:a:1")
    ap.add_argument("--timeout", type=int, default=180)
    ap.add_argument("--max-retries", type=int, default=2, choices=(0, 1, 2))
    ap.add_argument("--force", action="store_true")
    args = ap.parse_args()
    key = load_key()
    jobs = []
    if args.only:
        r, c, t = args.only.split(":")
        jobs = [(r, c, int(t))]
    else:
        jobs = [(r, c, t) for r in ROUTES for c in CASES for t in range(1, 6)]
    results = []
    # One worker per route/case stream; trials within a stream remain ordered.
    streams = {}
    for r, c, t in jobs:
        streams.setdefault((r, c), []).append(t)
    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
        futures = []
        for (r, c), trials in streams.items():
            def stream(r=r, c=c, trials=trials):
                return [run_trial(r, c, t, key, args.timeout, args.max_retries, args.force) for t in trials]
            futures.append(pool.submit(stream))
        for f in concurrent.futures.as_completed(futures):
            result = f.result()
            results.extend(result)
            print(json.dumps(result, ensure_ascii=False), flush=True)
    print(json.dumps({"completed_job_groups": len(results), "finished_at": now()}), flush=True)

if __name__ == "__main__":
    main()
