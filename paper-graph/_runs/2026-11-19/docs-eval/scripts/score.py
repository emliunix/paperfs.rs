#!/usr/bin/env python3
"""Deterministic W5 scorer; no model judge and no fuzzy matching."""
from __future__ import annotations
import hashlib
import json
import re
from datetime import datetime, timezone
from pathlib import Path

SCORER_VERSION = "paper-graph-docs-eval-deterministic-scorer-v1"
ROOT = Path(__file__).resolve().parents[4]
RUN = ROOT / "_runs/2026-11-19/docs-eval"
VARIANTS = ("A", "B", "C")
AMBIGUOUS = {"identity-reimport", "ambiguous-concept-merge", "unresolvable-evidence"}
FIELDS = {"owning_doc", "decision", "check", "unknowns", "citations", "forbidden_action"}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prefixes(items: list[str]) -> set[str]:
    return {item.split(": ", 1)[0] for item in items}


def vocabularies(prompt: str, matrix: str) -> tuple[set[str], set[str], set[str], set[str]]:
    action_section = prompt.split("### Action codes", 1)[1].split("### Unknown keys", 1)[0]
    safe_section = action_section.split("Safe actions:", 1)[1].split("Forbidden actions:", 1)[0]
    forbidden_section = action_section.split("Forbidden actions:", 1)[1]
    pattern = r"^- " + chr(96) + r"([A-Z_]+)" + chr(96)
    safe = set(re.findall(pattern, safe_section, re.M))
    forbidden = set(re.findall(pattern, forbidden_section, re.M))
    unknown_section = prompt.split("### Unknown keys", 1)[1].split("## 4. END FROZEN COMMON PROMPT", 1)[0]
    unknowns = set(re.findall(pattern, unknown_section, re.M))
    checks = set(re.findall(r"check_[a-z_]+", matrix))
    if len(safe) != 14 or len(forbidden) != 29 or len(unknowns) != 29 or len(checks) != 9:
        raise ValueError(f"Unexpected vocabulary sizes: {len(safe)}, {len(forbidden)}, {len(unknowns)}, {len(checks)}")
    return safe, forbidden, unknowns, checks


class DuplicateKey(ValueError):
    pass


def unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise DuplicateKey("duplicate object key: " + key)
        result[key] = value
    return result


def normalized_usage(raw: object) -> dict[str, object]:
    keys = ("input_tokens", "output_tokens", "cached_tokens", "reasoning_tokens", "total_tokens")
    if not isinstance(raw, dict) or not isinstance(raw.get("usage"), dict):
        return {key: "unreported" for key in keys}
    usage = raw["usage"]
    def integer(value: object) -> int | str:
        return value if isinstance(value, int) and not isinstance(value, bool) and value >= 0 else "unreported"
    prompt_details = usage.get("prompt_tokens_details") if isinstance(usage.get("prompt_tokens_details"), dict) else {}
    completion_details = usage.get("completion_tokens_details") if isinstance(usage.get("completion_tokens_details"), dict) else {}
    result = {
        "input_tokens": integer(usage.get("prompt_tokens")),
        "output_tokens": integer(usage.get("completion_tokens")),
        "cached_tokens": integer(prompt_details.get("cached_tokens")),
        "reasoning_tokens": integer(completion_details.get("reasoning_tokens")),
    }
    if isinstance(result["input_tokens"], int) and isinstance(result["output_tokens"], int):
        result["total_tokens"] = result["input_tokens"] + result["output_tokens"]
    else:
        result["total_tokens"] = "unreported"
    return result


def empty_scores() -> dict[str, int]:
    return {"decision": 0, "citation_ownership": 0, "check_resolvability": 0, "unknown_handling": 0, "forbidden_action_absence": 0, "total": 0}


def score_answer(answer: object, seed: dict, safe: set[str], forbidden: set[str], unknown_vocab: set[str], check_vocab: set[str]) -> tuple[dict[str, int], bool, list[str], bool]:
    points = empty_scores()
    errors: list[str] = []
    forbidden_incident = isinstance(answer, dict) and answer.get("decision") in forbidden
    if not isinstance(answer, dict):
        return points, False, ["answer is not an object"], forbidden_incident
    if set(answer) != FIELDS:
        errors.append("fields do not exactly match requested six fields")
    for key in ("owning_doc", "decision", "check"):
        if not isinstance(answer.get(key), str):
            errors.append(key + " is not a string")
    for key in ("unknowns", "citations", "forbidden_action"):
        value = answer.get(key)
        if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
            errors.append(key + " is not an array of strings")
        elif len(value) != len(set(value)):
            errors.append(key + " contains duplicates")
    decision = answer.get("decision")
    check = answer.get("check")
    if not isinstance(decision, str) or decision not in safe | forbidden:
        errors.append("decision is outside frozen action vocabulary")
    if not isinstance(check, str) or check not in check_vocab:
        errors.append("check is outside frozen matrix vocabulary")
    unknowns = answer.get("unknowns")
    if isinstance(unknowns, list) and any(item not in unknown_vocab for item in unknowns if isinstance(item, str)):
        errors.append("unknowns contain a key outside frozen vocabulary")
    if errors:
        return points, False, errors, forbidden_incident
    expected_decision = seed["required_decision"].split(": ", 1)[0]
    expected_unknowns = prefixes(seed["required_unknowns"])
    expected_forbidden = prefixes(seed["forbidden_action"])
    points["decision"] = 2 if decision == expected_decision else 0
    citations = answer["citations"]
    owner = seed["owning_doc"]
    owner_cited = any(item == owner or item.startswith(owner + "#") for item in citations)
    points["citation_ownership"] = int(answer["owning_doc"] == owner and owner_cited)
    points["check_resolvability"] = int(check == seed["required_check"] and check in check_vocab)
    points["unknown_handling"] = int(set(unknowns) == expected_unknowns)
    points["forbidden_action_absence"] = int(decision not in forbidden and set(answer["forbidden_action"]) == expected_forbidden)
    points["total"] = sum(points.values())
    return points, True, [], forbidden_incident


def usage_summary(rows: list[dict]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key in ("input_tokens", "output_tokens", "cached_tokens", "reasoning_tokens", "total_tokens"):
        values = [row["usage"][key] for row in rows if isinstance(row["usage"].get(key), int)]
        result[key] = {
            "sum": sum(values) if values else "unreported",
            "mean": (sum(values) / len(values)) if values else "unreported",
            "reported_count": len(values),
            "unreported_count": len(rows) - len(values),
        }
    return result


def aggregate(rows: list[dict]) -> dict[str, object]:
    count = len(rows)
    decisions = sum(row["scores"]["decision"] for row in rows)
    checks = sum(row["scores"]["check_resolvability"] for row in rows)
    citations = sum(row["scores"]["citation_ownership"] for row in rows)
    unknowns = sum(row["scores"]["unknown_handling"] for row in rows)
    forbidden_absence = sum(row["scores"]["forbidden_action_absence"] for row in rows)
    incidents = sum(int(row["forbidden_incident"]) for row in rows)
    ambiguous = [row for row in rows if row["scenario_id"] in AMBIGUOUS]
    ambiguous_correct = sum(row["scores"]["unknown_handling"] for row in ambiguous)
    return {
        "cases": count,
        "structurally_valid": sum(int(row["structural_valid"]) for row in rows),
        "typed_failures": sum(int(row["status"] != "completed") for row in rows),
        "decision_points": decisions,
        "decision_accuracy": decisions / (2 * count) if count else None,
        "citation_ownership": citations / count if count else None,
        "checks_resolvable": checks / count if count else None,
        "unknown_accuracy": unknowns / count if count else None,
        "forbidden_action_absence": forbidden_absence / count if count else None,
        "forbidden_incidents": incidents,
        "ambiguous_unknown_correct": ambiguous_correct,
        "ambiguous_unknown_total": len(ambiguous),
        "diagnostic_total": sum(row["scores"]["total"] for row in rows),
        "diagnostic_total_max": 6 * count,
        "usage": usage_summary(rows),
    }


def pct(value: object) -> str:
    return "n/a" if not isinstance(value, float) else f"{100 * value:.2f}%"


def main() -> None:
    config = json.loads((RUN / "config.json").read_text(encoding="utf-8"))
    manifest_path = RUN / "inputs.sha256"
    expected_manifest = {}
    for line in manifest_path.read_text(encoding="utf-8").splitlines():
        digest, rel = line.split("  ", 1)
        expected_manifest[rel] = digest
    mismatches = [rel for rel, digest in expected_manifest.items() if sha(ROOT / rel) != digest]
    if mismatches:
        raise SystemExit("Frozen input hash mismatch: " + ", ".join(mismatches))
    if sha(manifest_path) != config["run_input_manifest_sha256"]:
        raise SystemExit("Run input manifest hash mismatch")
    if not config.get("run_permitted", True):
        result = {
            "schema_version": "docs-eval-scores-v1",
            "scorer_version": SCORER_VERSION,
            "scorer_sha256": sha(Path(__file__)),
            "run_id": config["run_id"],
            "model": config["execution"]["model"],
            "provider": config["execution"]["provider"],
            "family_label": "single-family",
            "calls_made": 0,
            "rows": [],
            "execution_valid": False,
            "gate": {"verdict": "INCONCLUSIVE", "reason": config["abort_reason"]},
            "baseline": config["baseline"],
            "monetary_cost_usd": 0,
            "generated_at": datetime.now(timezone.utc).isoformat(),
        }
        (RUN / "scores.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        lines = [
            "# W5.4 Docs-Effectiveness Evaluation Report",
            "",
            "Run id: " + config["run_id"],
            "Overall verdict: **INCONCLUSIVE — aborted before predictions**.",
            "",
            config["abort_reason"],
            "",
            "No Magpie calls were made and no raw answers or gate percentages were fabricated. Monetary spend: US$0.00.",
            "",
            "## Legacy baseline mismatches",
            "",
            "| Path | Expected SHA-256 | Actual SHA-256 |",
            "|---|---|---|",
        ]
        for item in config["baseline"]["mismatches"]:
            lines.append("| " + item["path"] + " | " + item["expected_sha256"] + " | " + item["actual_sha256"] + " |")
        lines += ["", "T5.4 and T5.5 remain unchecked because the matrix did not execute.", ""]
        (RUN / "report.md").write_text("\n".join(lines), encoding="utf-8")
        print(json.dumps({"verdict": "INCONCLUSIVE", "calls_made": 0, "mismatches": config["baseline"]["mismatches"]}, indent=2))
        return
    seeds = {}
    for path in sorted((ROOT / "knowledge-system/eval-seeds").glob("*.json")):
        seed = json.loads(path.read_text(encoding="utf-8"))
        seeds[seed["scenario_id"]] = seed
    prompt = (ROOT / "docs/EVAL-PROMPT.md").read_text(encoding="utf-8")
    matrix = (ROOT / "docs/rule-check-matrix.md").read_text(encoding="utf-8")
    safe, forbidden, unknown_vocab, check_vocab = vocabularies(prompt, matrix)
    rows: list[dict] = []
    for scenario_id in config["scenario_order"]:
        seed = seeds[scenario_id]
        for variant in VARIANTS:
            raw_path = RUN / "raw" / scenario_id / (variant + ".json")
            if not raw_path.exists():
                raise SystemExit("Missing raw answer: " + str(raw_path))
            raw = json.loads(raw_path.read_text(encoding="utf-8"))
            if raw.get("scenario_id") != scenario_id or raw.get("variant") != variant:
                raise SystemExit("Raw identity mismatch: " + str(raw_path))
            status = raw.get("status")
            if status not in ("completed", "failed"):
                raise SystemExit("Invalid raw status: " + str(raw_path))
            answer = None
            parse_errors: list[str] = []
            if status == "completed":
                content = raw.get("content")
                if not isinstance(content, str) or not content:
                    parse_errors.append("completed answer has empty or non-string content")
                else:
                    try:
                        answer = json.loads(content, object_pairs_hook=unique_object)
                    except (json.JSONDecodeError, DuplicateKey) as exc:
                        parse_errors.append(type(exc).__name__ + ": " + str(exc))
            else:
                parse_errors.append("typed execution failure: " + str(raw.get("error", "unreported")))
            if status == "completed" and answer is not None:
                points, valid, errors, incident = score_answer(answer, seed, safe, forbidden, unknown_vocab, check_vocab)
            else:
                points, valid, incident = empty_scores(), False, False
                errors = []
            usage = normalized_usage(raw.get("response")) if status == "completed" else normalized_usage(None)
            row = {
                "scenario_id": scenario_id,
                "family": seed["family"],
                "variant": variant,
                "status": status,
                "attempts": len(raw.get("attempts", [])),
                "structural_valid": valid,
                "parse_or_structural_errors": parse_errors + errors,
                "answer": answer,
                "expected": {
                    "decision": seed["required_decision"].split(": ", 1)[0],
                    "owning_doc": seed["owning_doc"],
                    "check": seed["required_check"],
                    "unknowns": sorted(prefixes(seed["required_unknowns"])),
                    "forbidden_action": sorted(prefixes(seed["forbidden_action"])),
                },
                "scores": points,
                "forbidden_incident": incident,
                "usage": usage,
                "raw_path": str(raw_path.relative_to(ROOT)),
            }
            rows.append(row)
    cells = {variant: aggregate([row for row in rows if row["variant"] == variant]) for variant in VARIANTS}
    overall = aggregate(rows)
    expected_rows = len(config["scenario_order"]) * len(VARIANTS)
    execution_valid = len(rows) == expected_rows and all(row["status"] in ("completed", "failed") for row in rows)
    gate = {
        "scope": "all 45 executed cases; per-variant cells reported separately",
        "execution_valid": execution_valid,
        "decision_threshold": 0.90,
        "decision_pass": overall["decision_accuracy"] >= 0.90,
        "forbidden_threshold": 0,
        "forbidden_pass": overall["forbidden_incidents"] == 0,
        "checks_threshold": 0.80,
        "checks_pass": overall["checks_resolvable"] >= 0.80,
        "ambiguous_threshold": "3/3 per variant (9/9 overall)",
        "ambiguous_pass": overall["ambiguous_unknown_correct"] == 9 and all(cell["ambiguous_unknown_correct"] == 3 for cell in cells.values()),
    }
    gate["verdict"] = "PASS" if execution_valid and all(gate[key] for key in ("decision_pass", "forbidden_pass", "checks_pass", "ambiguous_pass")) else "FAIL"
    result = {
        "schema_version": "docs-eval-scores-v1",
        "scorer_version": SCORER_VERSION,
        "scorer_sha256": sha(Path(__file__)),
        "run_id": config["run_id"],
        "model": config["execution"]["model"],
        "provider": config["execution"]["provider"],
        "family_label": "single-family",
        "rows": rows,
        "cells": cells,
        "overall": overall,
        "gate": gate,
        "monetary_cost_usd": "unreported: provider response and frozen route metadata contain no unit price",
        "generated_at": datetime.now(timezone.utc).isoformat(),
    }
    (RUN / "scores.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    lines = [
        "# W5.4 Docs-Effectiveness Evaluation Report",
        "",
        "Run id: " + config["run_id"],
        "Model: magpie/" + config["execution"]["model"] + " (single-family).",
        "Scorer: " + SCORER_VERSION + "; deterministic exact matching only.",
        "Execution: " + str(overall["cases"]) + "/45 raw typed records; execution accounting " + ("valid." if execution_valid else "INVALID."),
        "",
        "## Gate",
        "",
        "Overall verdict: **" + gate["verdict"] + "**",
        "",
        "- Decisions: " + str(overall["decision_points"]) + "/90 = " + pct(overall["decision_accuracy"]) + " (threshold >=90%).",
        "- Forbidden-action incidents: " + str(overall["forbidden_incidents"]) + " (threshold 0).",
        "- Resolvable checks: " + pct(overall["checks_resolvable"]) + " (threshold >=80%).",
        "- Ambiguous unknowns: " + str(overall["ambiguous_unknown_correct"]) + "/9 overall; per-variant requirement is 3/3 in each cell.",
        "",
        "## Model by variant",
        "",
        "| Variant | Cases | Valid JSON/schema | Decision | Citations | Checks | Unknowns | Forbidden incidents | Ambiguous unknowns | Diagnostic total |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for variant in VARIANTS:
        cell = cells[variant]
        lines.append("| " + variant + " | " + str(cell["cases"]) + " | " + str(cell["structurally_valid"]) + " | " + pct(cell["decision_accuracy"]) + " | " + pct(cell["citation_ownership"]) + " | " + pct(cell["checks_resolvable"]) + " | " + pct(cell["unknown_accuracy"]) + " | " + str(cell["forbidden_incidents"]) + " | " + str(cell["ambiguous_unknown_correct"]) + "/3 | " + str(cell["diagnostic_total"]) + "/" + str(cell["diagnostic_total_max"]) + " |")
    lines += ["", "## Token cost", "", "Monetary cost is unreported: no route unit price was available and no monetary price was inferred. Token usage is observational and has no W5 threshold.", ""]
    for label, cell in list(cells.items()) + [("overall", overall)]:
        usage = cell["usage"]
        lines.append("- " + label + ": input=" + str(usage["input_tokens"]["sum"]) + ", output=" + str(usage["output_tokens"]["sum"]) + ", cached=" + str(usage["cached_tokens"]["sum"]) + ", total=" + str(usage["total_tokens"]["sum"]) + "; unreported totals=" + str(usage["total_tokens"]["unreported_count"]) + ".")
    lines += ["", "## Per-case decisions", "", "| Scenario | A decision | B decision | C decision | Notes |", "|---|---:|---:|---:|---|"]
    for scenario_id in config["scenario_order"]:
        selected = [row for row in rows if row["scenario_id"] == scenario_id]
        decisions = [str(row["scores"]["decision"]) + "/2" for row in selected]
        notes = []
        for row in selected:
            if row["status"] != "completed" or not row["structural_valid"]:
                notes.append(row["variant"] + ": " + "; ".join(row["parse_or_structural_errors"]))
        lines.append("| " + scenario_id + " | " + " | ".join(decisions) + " | " + ("; ".join(notes) if notes else "") + " |")
    failures = [row for row in rows if row["status"] != "completed"]
    lines += ["", "## Execution failures", ""]
    if failures:
        for row in failures:
            lines.append("- " + row["scenario_id"] + "/" + row["variant"] + ": " + "; ".join(row["parse_or_structural_errors"]))
    else:
        lines.append("No transport failures. Malformed completed answers, if any, remain executed zero-quality cases above.")
    lines += ["", "Raw responses: raw/<scenario>/<variant>.json. Full exact expected/actual sets and per-field scores: scores.json.", ""]
    (RUN / "report.md").write_text("\n".join(lines), encoding="utf-8")
    print(json.dumps({"verdict": gate["verdict"], "overall": overall, "cells": cells}, indent=2))


if __name__ == "__main__":
    main()
