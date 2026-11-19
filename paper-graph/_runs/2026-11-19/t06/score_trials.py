#!/usr/bin/env python3
import datetime as dt
import hashlib
import json
import math
from pathlib import Path
import statistics
from run_trials import prompt_for

ROOT = Path(__file__).resolve().parent
CASE_FILES = {
    "theorem-definition-citation": ROOT / "cases/a-theorem-definition-citation.json",
    "five-relations-evidence": ROOT / "cases/b-five-relations-evidence.json",
    "ambiguous-mention-use": ROOT / "cases/c-ambiguous-mention-use.json",
}
ARRAY_KEY = {
    "theorem-definition-citation": "items",
    "five-relations-evidence": "relations",
    "ambiguous-mention-use": "observations",
}

def validate(value, schema, path="$"):
    errors = []
    if "const" in schema and value != schema["const"]:
        errors.append(f"{path}: expected const {schema['const']!r}")
    if "enum" in schema and value not in schema["enum"]:
        errors.append(f"{path}: value not in enum")
    typ = schema.get("type")
    if typ:
        ok = {
            "object": lambda x: isinstance(x, dict),
            "array": lambda x: isinstance(x, list),
            "string": lambda x: isinstance(x, str),
            "integer": lambda x: isinstance(x, int) and not isinstance(x, bool),
        }[typ](value)
        if not ok:
            return [f"{path}: expected {typ}"]
    if isinstance(value, dict):
        for key in schema.get("required", []):
            if key not in value:
                errors.append(f"{path}: missing required key {key}")
        props = schema.get("properties", {})
        if schema.get("additionalProperties") is False:
            for key in value:
                if key not in props:
                    errors.append(f"{path}: additional key {key}")
        for key, subschema in props.items():
            if key in value:
                errors.extend(validate(value[key], subschema, f"{path}.{key}"))
    if isinstance(value, list):
        if len(value) < schema.get("minItems", 0):
            errors.append(f"{path}: fewer than minItems")
        if "maxItems" in schema and len(value) > schema["maxItems"]:
            errors.append(f"{path}: more than maxItems")
        if "items" in schema:
            for i, item in enumerate(value):
                errors.extend(validate(item, schema["items"], f"{path}[{i}]"))
    if isinstance(value, str) and len(value) < schema.get("minLength", 0):
        errors.append(f"{path}: shorter than minLength")
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        if "minimum" in schema and value < schema["minimum"]:
            errors.append(f"{path}: below minimum")
    return errors

def line_span(text, start, end):
    return (text.count("\n", 0, start) + 1, text.count("\n", 0, max(start, end - 1)) + 1)

def percentile(values, q):
    if not values:
        return None
    if q == 0.5:
        return round(statistics.median(values), 1)
    ordered = sorted(values)
    return ordered[max(0, math.ceil(q * len(ordered)) - 1)]

def summarize(records):
    lat = [r["latency_ms"] for r in records]
    taxonomy = {}
    for r in records:
        for category in r["failure_taxonomy"]:
            taxonomy[category] = taxonomy.get(category, 0) + 1
    n = len(records)
    structural = sum(r["structural_pass"] for r in records)
    semantic = sum(r["semantic_pass"] for r in records)
    empty = sum(r["empty"] for r in records)
    truncated = sum(r["truncated"] for r in records)
    return {
        "trials": n,
        "structural_pass_count": structural,
        "structural_pass_rate": structural / n if n else None,
        "semantic_pass_count": semantic,
        "semantic_pass_rate": semantic / n if n else None,
        "empty_count": empty, "empty_rate": empty / n if n else None,
        "truncated_count": truncated, "truncated_rate": truncated / n if n else None,
        "latency_first_attempt_ms": {"p50": percentile(lat, .5), "p95": percentile(lat, .95)},
        "failure_taxonomy_trial_counts": dict(sorted(taxonomy.items())),
        "attempt_count": sum(len(r["attempts"]) for r in records),
    }

def score_trial(path, cases):
    log = json.loads(path.read_text())
    case = cases[log["case_id"]]
    text = case["excerpt_text"]
    attempts = []
    for a in log["attempts"]:
        attempts.append({
            "attempt": a["attempt"], "started_at": a["started_at"], "finished_at": a["finished_at"],
            "latency_ms": a["latency_ms"], "http_status": a.get("http_status"),
            "transport_ok": a.get("transport_ok"), "finish_reason": a.get("finish_reason"),
            "served_model": a.get("served_model"), "usage": a.get("usage"),
            "empty": a.get("empty"), "truncated": a.get("truncated"),
            "retry_scheduled": a.get("retry_scheduled"),
            "failure_type": a.get("failure_type"), "error_verbatim": a.get("error_verbatim"),
        })
    scored = log["attempts"][0]
    response = scored.get("response_text")
    empty = not (response or "").strip()
    truncated = scored.get("finish_reason") in ("length", "max_tokens")
    parsed = None
    parse_error = None
    if not empty:
        try:
            parsed = json.loads(response)
        except Exception as e:
            parse_error = f"{type(e).__name__}: {e}"
    schema_errors = validate(parsed, case["json_schema"]) if parsed is not None else []
    required_errors = [e for e in schema_errors if "missing required key" in e]
    structural = parsed is not None and not schema_errors
    semantic_checks = []
    if structural:
        for i, item in enumerate(parsed[ARRAY_KEY[log["case_id"]]]):
            ev = item["evidence"]
            quote = ev["quote"]
            start, end = ev["start_offset"], ev["end_offset"]
            in_bounds = 0 <= start < end <= len(text)
            offset_ok = in_bounds and text[start:end] == quote
            expected_lines = line_span(text, start, end) if in_bounds else None
            lines_ok = expected_lines is not None and list(expected_lines) == [ev["line_start"], ev["line_end"]]
            marker20 = text.index("=== PAGE 20 ===")
            expected_page = 1 if start < marker20 else 20
            page_ok = ev["page"] == expected_page
            semantic_checks.append({
                "item_index": i, "quote_exact_substring": quote in text,
                "offsets_resolve": offset_ok, "line_locator_resolves": lines_ok,
                "page_resolves": page_ok, "expected_lines": expected_lines,
            })
    semantic = structural and bool(semantic_checks) and all(
        c["quote_exact_substring"] and c["offsets_resolve"] and c["line_locator_resolves"] and c["page_resolves"]
        for c in semantic_checks
    )
    failures = []
    if not scored.get("transport_ok"):
        failures.append(scored.get("failure_type") or "transport_error")
    if empty: failures.append("empty")
    if truncated: failures.append("truncated")
    if parse_error: failures.append("json_parse_error")
    if required_errors: failures.append("missing_required_keys")
    if parsed is not None and schema_errors and not required_errors: failures.append("schema_validation_error")
    if structural and not semantic:
        categories = {
            "quote_exact_substring": "semantic_quote_not_exact_substring",
            "offsets_resolve": "semantic_offset_mismatch",
            "line_locator_resolves": "semantic_line_locator_mismatch",
            "page_resolves": "semantic_page_mismatch",
        }
        for key, category in categories.items():
            if any(not c[key] for c in semantic_checks): failures.append(category)
    prompt_material = prompt_for(case)
    return {
        "route": log["route"], "provider": log["provider"], "model": log["model"],
        "case_id": log["case_id"], "trial": log["trial"], "effort": log["effort"],
        "transport": log["transport"], "max_output_tokens": 16384, "timeout_per_attempt_ms": 180000,
        "prompt_sha256": hashlib.sha256(prompt_material.encode()).hexdigest(),
        "excerpt_sha256": hashlib.sha256(text.encode()).hexdigest(),
        "attempt_log_path": str(path.relative_to(ROOT)), "attempts": attempts,
        "scored_response_text": response,
        "json_parse_pass": parsed is not None, "json_parse_error": parse_error,
        "required_keys_pass": not required_errors and parsed is not None,
        "typed_schema_valid": parsed is not None and not schema_errors,
        "schema_errors": schema_errors, "structural_pass": structural,
        "semantic_checks": semantic_checks, "semantic_pass": semantic,
        "empty": empty, "truncated": truncated, "finish_reason": scored.get("finish_reason"),
        "failure_taxonomy": sorted(set(failures)),
        "latency_ms": scored["latency_ms"],
        "all_attempt_latency_ms": sum(a["latency_ms"] for a in log["attempts"]),
        "retry_budget": log["max_retries"],
    }

def main():
    cases = {key: json.loads(path.read_text()) for key, path in CASE_FILES.items()}
    paths = sorted((ROOT / "attempts").glob("*/*/trial-*.json"))
    if len(paths) != 30:
        raise SystemExit(f"Expected 30 scored trial logs, found {len(paths)}; refusing to write final results")
    records = [score_trial(p, cases) for p in paths]
    records.sort(key=lambda r: (r["route"], r["case_id"], r["trial"]))
    routes = sorted(set(r["route"] for r in records))
    by_route = {route: summarize([r for r in records if r["route"] == route]) for route in routes}
    by_route_case = {
        route: {case: summarize([r for r in records if r["route"] == route and r["case_id"] == case]) for case in CASE_FILES}
        for route in routes
    }
    output = {
        "schema_version": "t06-reliability-v1",
        "recorded_at": dt.datetime.now(dt.timezone.utc).astimezone().isoformat(),
        "source": {"paper_id": 3154, "pages": [1, 20], "pages_jsonl": "paper-graph/_runs/2026-11-19/pages.jsonl"},
        "design": {"trials_per_route_case": 5, "routes": 2, "cases": 3, "scored_trials": 30, "max_retries": 2, "effort": "low"},
        "transport_deviation": "Direct Magpie gateway HTTP calls, not subagent-tool calls: this delegated agent had remaining delegation depth 0. Exact requested subagent transport remains unverified.",
        "scoring": {
            "structural_pass": "Final attempt response parses as bare JSON and validates against the complete case JSON Schema (including all required keys).",
            "semantic_pass": "Structural pass and every evidence quote is an exact excerpt substring whose Unicode code-point offsets, page, and inclusive line locators all resolve. No gold correctness judgment is implied.",
            "latency": "First-attempt latency only; retries never rescue scores or alter latency. p50 uses median, p95 uses nearest-rank.",
            "empty_truncated": "Measured on the first attempt. Later retry failures remain in attempts and attempt logs.",
        },
        "aggregates": {"overall": summarize(records), "by_route": by_route, "by_route_case": by_route_case},
        "records": records,
        "unscored_probe_logs": ["probe-4k/", "probe-16k/", "probe-aborts.json"],
        "cost": {"status": "not_measured", "reason": "No published per-token price for these routes was established."},
        "arm_frozen": False,
        "w7_status": "blocked_pending_human_review",
    }
    (ROOT / "t06-reliability.json").write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n")
    lines = [
        "# T0.6 structured-output reliability", "",
        "**No arm is frozen. W7 stays blocked pending human review.** The requested subagent transport could not be used at delegation depth 1 (remaining depth 0); these are direct Magpie gateway calls, so exact subagent behavior remains unverified.", "",
        "Design: 2 routes × 3 cases × N=5 = 30 scored trials, effort low, 16,384 output tokens, 180 s/attempt, retry budget 0–2. Scores and latency use the first attempt; retries are observational and cannot rescue a failure. Six concurrent route/case streams ran trials in order. Source: paper 3154 pages 1 and 20. Structural means bare JSON plus full schema validity; semantic additionally requires every quote, code-point offset, page, and line locator to resolve.", "",
        "| Route | Case | Structural | Semantic | Empty/truncated | p50 ms | p95 ms |",
        "|---|---|---:|---:|---:|---:|---:|",
    ]
    short = {"theorem-definition-citation":"theorem/def/cite", "five-relations-evidence":"five relations", "ambiguous-mention-use":"mention vs use"}
    for route in routes:
        for case in CASE_FILES:
            a = by_route_case[route][case]
            lines.append(f"| {route} | {short[case]} | {a['structural_pass_count']}/5 | {a['semantic_pass_count']}/5 | {a['empty_count']}/{a['truncated_count']} | {a['latency_first_attempt_ms']['p50']} | {a['latency_first_attempt_ms']['p95']} |")
    lines += ["", "| Route | Structural total | Semantic total | p50 ms | p95 ms |", "|---|---:|---:|---:|---:|"]
    for route in routes:
        a = by_route[route]
        lines.append(f"| {route} | {a['structural_pass_count']}/15 | {a['semantic_pass_count']}/15 | {a['latency_first_attempt_ms']['p50']} | {a['latency_first_attempt_ms']['p95']} |")
    lines += ["", "## Failure taxonomy and decision", ""]
    for route in routes:
        lines.append(f"- **{route}:** {json.dumps(by_route[route]['failure_taxonomy_trial_counts'], ensure_ascii=False)}")
    lines += [
        "- Semantic pass is evidence recoverability only, not extraction correctness or human gold agreement.",
        "- Four completed unscored probe attempts are retained under probe-4k/ and probe-16k/; two wrapper-aborted probe calls are logged in probe-aborts.json. Retry-only work stopped after deterministic token exhaustion/timeouts; missing trials resumed with retry budget 0. Scored failures and verbatim gateway errors are under attempts/.",
        "- Freezing requires human review and, if strict protocol adherence is required, rerunning through the parent agent's subagent tool. Until then neither reliability nor latency qualifies an extraction arm.",
    ]
    (ROOT / "REPORT.md").write_text("\n".join(lines) + "\n")
    print(json.dumps({"records": len(records), "by_route": by_route, "report": str(ROOT / 'REPORT.md')}, ensure_ascii=False, indent=2))

if __name__ == "__main__":
    main()
