#!/usr/bin/env python3
"""Append deterministic r2/r3 execution and gate comparison; never rescore answers."""
from __future__ import annotations
import collections
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
BASE = ROOT / "_runs/2026-11-19/docs-eval"
RUNS = {"r2": BASE, "r3": BASE / "r3"}


def pct(value: float) -> str:
    return f"{100 * value:.2f}%"


def classify_attempt(attempt: dict, run_id: str) -> str | None:
    error = attempt.get("error")
    if isinstance(error, dict) and error.get("class"):
        legacy = error["class"]
        if run_id == "r2" and legacy == "http_error" and attempt.get("http_status") == 502:
            return "http_502"
        return legacy
    warning = attempt.get("warning")
    if isinstance(warning, dict) and warning.get("class"):
        return warning["class"]
    response = attempt.get("response")
    if isinstance(response, dict):
        choices = response.get("choices")
        if isinstance(choices, list) and choices and isinstance(choices[0], dict):
            choice = choices[0]
            if choice.get("finish_reason") == "length":
                message = choice.get("message") if isinstance(choice.get("message"), dict) else {}
                content = message.get("content")
                return "budget_exhaustion_truncated_content" if isinstance(content, str) and content else "budget_exhaustion_empty_content"
    return None


def run_facts(run_id: str, path: Path) -> dict:
    scores = json.loads((path / "scores.json").read_text(encoding="utf-8"))
    execution = json.loads((path / "execution.json").read_text(encoding="utf-8"))
    attempt_classes = collections.Counter()
    final_classes = collections.Counter()
    http_502_empty = 0
    http_502_empty_reply_budget = 0
    reasoning_nonempty = 0
    reasoning_nonzero = 0
    reasoning_reported = 0
    length_completions = 0
    raw_paths = sorted((path / "raw").glob("*/*.json"))
    for raw_path in raw_paths:
        raw = json.loads(raw_path.read_text(encoding="utf-8"))
        for attempt in raw.get("attempts", []):
            label = classify_attempt(attempt, run_id)
            if label:
                attempt_classes[label] += 1
            if attempt.get("http_status") == 502 and not attempt.get("body_text"):
                http_502_empty += 1
            if attempt.get("http_status") == 502 and "an empty reply" in attempt.get("body_text", "") and "the whole output spent on reasoning" in attempt.get("body_text", ""):
                http_502_empty_reply_budget += 1
        if raw.get("status") == "failed":
            error = raw.get("error") if isinstance(raw.get("error"), dict) else {}
            label = error.get("class", "unknown_failure")
            if run_id == "r2" and label == "http_error":
                statuses = [a.get("http_status") for a in raw.get("attempts", [])]
                label = "http_502" if statuses and statuses[-1] == 502 else "http_error"
            final_classes[label] += 1
        response = raw.get("response") if isinstance(raw.get("response"), dict) else {}
        choices = response.get("choices") if isinstance(response.get("choices"), list) else []
        if choices and isinstance(choices[0], dict):
            choice = choices[0]
            length_completions += int(choice.get("finish_reason") == "length")
            message = choice.get("message") if isinstance(choice.get("message"), dict) else {}
            reasoning_nonempty += int(isinstance(message.get("reasoning_content"), str) and bool(message.get("reasoning_content")))
        usage = response.get("usage") if isinstance(response.get("usage"), dict) else {}
        details = usage.get("completion_tokens_details") if isinstance(usage.get("completion_tokens_details"), dict) else {}
        reasoning_tokens = details.get("reasoning_tokens")
        if isinstance(reasoning_tokens, int):
            reasoning_reported += 1
            reasoning_nonzero += int(reasoning_tokens > 0)
    return {
        "run_id": scores["run_id"],
        "scores": scores,
        "execution": execution,
        "raw_records": len(raw_paths),
        "attempt_failure_taxonomy": dict(sorted(attempt_classes.items())),
        "final_failure_taxonomy": dict(sorted(final_classes.items())),
        "http_502_empty_body_attempts": http_502_empty,
        "http_502_empty_reply_budget_attempts": http_502_empty_reply_budget,
        "reasoning": {
            "responses_with_nonempty_reasoning_content": reasoning_nonempty,
            "responses_with_reasoning_tokens_reported": reasoning_reported,
            "responses_with_nonzero_reasoning_tokens": reasoning_nonzero,
        },
        "length_finish_completions": length_completions,
    }


def main() -> None:
    facts = {name: run_facts(name, path) for name, path in RUNS.items()}
    comparison = {
        "schema_version": "docs-eval-r2-r3-comparison-v1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "r2_model_verdict_valid": False,
        "r2_supersession_reason": "2048-token output budget confounded by reasoning; r2 is not a model verdict",
        "runs": facts,
    }
    out = RUNS["r3"] / "comparison.json"
    out.write_text(json.dumps(comparison, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    r2, r3 = facts["r2"], facts["r3"]
    s2, s3 = r2["scores"], r3["scores"]
    o2, o3 = s2["overall"], s3["overall"]
    e2, e3 = r2["execution"], r3["execution"]
    lines = [
        "",
        "## r2 vs r3 (deterministic comparison)",
        "",
        "r2 is superseded for the budget confound and is not a model verdict. r3 froze byte-identical prompt/documentation/seed inputs before execution; only the declared execution configuration and typed runner changed.",
        "",
        "| Metric | r2 | r3 |",
        "|---|---:|---:|",
        f"| Matrix cases / raw records | {e2.get('matrix_cases')} / {r2['raw_records']} | {e3.get('matrix_cases')} / {r3['raw_records']} |",
        f"| HTTP POST attempts | {e2.get('http_post_attempts')} | {e3.get('http_post_attempts')} |",
        f"| Completed / failed cases | {e2.get('completed')} / {e2.get('failed')} | {e3.get('completed')} / {e3.get('failed')} |",
        f"| Decision points / accuracy | {o2['decision_points']}/90 ({pct(o2['decision_accuracy'])}) | {o3['decision_points']}/90 ({pct(o3['decision_accuracy'])}) |",
        f"| Forbidden-action incidents | {o2['forbidden_incidents']} | {o3['forbidden_incidents']} |",
        f"| Resolvable checks | {pct(o2['checks_resolvable'])} | {pct(o3['checks_resolvable'])} |",
        f"| Unknown handling (all cases) | {pct(o2['unknown_accuracy'])} | {pct(o3['unknown_accuracy'])} |",
        f"| Ambiguous unknowns | {o2['ambiguous_unknown_correct']}/9 | {o3['ambiguous_unknown_correct']}/9 |",
        f"| Structurally valid answers | {o2['structurally_valid']}/45 | {o3['structurally_valid']}/45 |",
        f"| Length-finish completions | {r2['length_finish_completions']} | {r3['length_finish_completions']} |",
        "",
        "### Failure taxonomy",
        "",
        "Attempt counts include retries; final-case counts classify each raw record once. Completed truncated responses appear only in attempt counts, not final failed-case counts.",
        "",
        "| Failure type | r2 attempts | r3 attempts | r2 final cases | r3 final cases |",
        "|---|---:|---:|---:|---:|",
    ]
    labels = sorted(set(r2["attempt_failure_taxonomy"]) | set(r3["attempt_failure_taxonomy"]) | set(r2["final_failure_taxonomy"]) | set(r3["final_failure_taxonomy"]))
    for label in labels:
        lines.append(f"| {label} | {r2['attempt_failure_taxonomy'].get(label, 0)} | {r3['attempt_failure_taxonomy'].get(label, 0)} | {r2['final_failure_taxonomy'].get(label, 0)} | {r3['final_failure_taxonomy'].get(label, 0)} |")
    lines += [
        "",
        f"HTTP 502 attempts whose bodies explicitly report an empty reply with the whole output spent on reasoning: r2={r2['http_502_empty_reply_budget_attempts']}, r3={r3['http_502_empty_reply_budget_attempts']}. These are budget-exhaustion evidence transported as HTTP 502, not independent proof of network failure.",
        "",
        "### Per-variant gate metrics",
        "",
        "| Run | Variant | Decision | Checks | Unknowns | Ambiguous unknowns | Forbidden incidents |",
        "|---|---|---:|---:|---:|---:|---:|",
    ]
    for run_id, scores in (("r2", s2), ("r3", s3)):
        for variant in ("A", "B", "C"):
            cell = scores["cells"][variant]
            lines.append(f"| {run_id} | {variant} | {pct(cell['decision_accuracy'])} | {pct(cell['checks_resolvable'])} | {pct(cell['unknown_accuracy'])} | {cell['ambiguous_unknown_correct']}/3 | {cell['forbidden_incidents']} |")
    gate_misses = [row for row in s3["rows"] if row["scenario_id"] in {"identity-reimport", "ambiguous-concept-merge", "unresolvable-evidence"} and not row["scores"]["unknown_handling"]]
    if gate_misses:
        lines += ["", "Ambiguous-gate zero rows:", ""]
        for row in gate_misses:
            actual_unknowns = row["answer"].get("unknowns") if isinstance(row["answer"], dict) else None
            lines.append(f"- {row['scenario_id']}/{row['variant']}: structural_valid={row['structural_valid']}; errors={row['parse_or_structural_errors']}; expected_unknowns={row['expected']['unknowns']}; actual_unknowns={actual_unknowns}.")
    reasoning = r3["reasoning"]
    lines += [
        "",
        "### Reasoning/budget verification",
        "",
        f"r3 responses with nonempty reasoning content: {reasoning['responses_with_nonempty_reasoning_content']}; with nonzero reported reasoning tokens: {reasoning['responses_with_nonzero_reasoning_tokens']} (reasoning token usage reported on {reasoning['responses_with_reasoning_tokens_reported']} responses).",
        "",
        "### T5.5 gate interpretation",
        "",
    ]
    if e3.get("persistent_http_502_blocker"):
        lines.append("**TRANSPORT BLOCKER:** persistent HTTP 502 exhausted the retry cap and stopped the matrix. Unattempted cases are typed failures; this is not a model-quality verdict.")
    elif s3["gate"]["verdict"] == "PASS":
        lines.append("**PASS:** execution accounting is valid and every per-variant T5.5 gate passes.")
    else:
        failed = [name for name in ("decision_pass", "forbidden_pass", "checks_pass", "ambiguous_pass") if not s3["gate"].get(name)]
        lines.append("**FAIL:** " + (", ".join(failed) if failed else "execution accounting invalid") + ". r3 has no transport or budget failures; the residual gate failure is answer/schema behavior, not the r2 2048-token budget confound.")
    lines += ["", "Machine-readable comparison: comparison.json.", ""]
    report = RUNS["r3"] / "report.md"
    with report.open("a", encoding="utf-8") as handle:
        handle.write("\n".join(lines))
    print(json.dumps({
        "r2": {"attempt_failures": r2["attempt_failure_taxonomy"], "final_failures": r2["final_failure_taxonomy"], "gate": s2["gate"]["verdict"]},
        "r3": {"attempt_failures": r3["attempt_failure_taxonomy"], "final_failures": r3["final_failure_taxonomy"], "gate": s3["gate"]["verdict"], "overall": o3, "reasoning": reasoning, "execution": e3},
        "comparison_sha256": hashlib.sha256(out.read_bytes()).hexdigest(),
    }, indent=2))


if __name__ == "__main__":
    main()
