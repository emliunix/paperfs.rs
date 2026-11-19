#!/usr/bin/env python3
"""Generate compact AGENTS.md rules solely from lessons.jsonl and rule-check-matrix.md."""
from __future__ import annotations
import json
import re
from pathlib import Path

GENERATOR_ID = "paper-graph-docs-eval-rule-generator-v1"
ROOT = Path(__file__).resolve().parents[5]
RUN = ROOT / "_runs/2026-11-19/docs-eval/r3"
LESSONS = ROOT / "knowledge-system/lessons.jsonl"
MATRIX = ROOT / "docs/rule-check-matrix.md"


def matrix_rows(text: str) -> dict[str, dict[str, object]]:
    rows: dict[str, dict[str, object]] = {}
    for line in text.splitlines():
        match = re.match(r"\| (L\d\d) \| (.*?) \| (.*?) \| (.*?) \| (.*?) \| (.*?) \|$", line)
        if not match:
            continue
        lesson, rule, check_type, owner, check_cell, path_cell = match.groups()
        checks = sorted(set(re.findall(r"check_[a-z_]+", check_cell)))
        links = re.findall(r"\]\(([^)]+)\)", path_cell)
        docs: list[str] = []
        for link in links:
            target = link.split("#", 1)[0]
            if not target.endswith(".md") or target.startswith("../tests/"):
                continue
            if target.startswith("../knowledge-system/"):
                target = "knowledge-system/" + target.removeprefix("../knowledge-system/")
            elif target.startswith("../"):
                continue
            else:
                target = "docs/" + target
            if target not in docs:
                docs.append(target)
        rows[lesson] = {
            "matrix_rule": rule,
            "check_type": check_type,
            "owner": owner.replace(chr(96), ""),
            "checks": checks,
            "docs": docs,
            "human_judgment": "Judgment (human)" in check_cell,
        }
    return rows


def main() -> None:
    lessons = [json.loads(line) for line in LESSONS.read_text(encoding="utf-8").splitlines() if line.strip()]
    rows = matrix_rows(MATRIX.read_text(encoding="utf-8"))
    if len(lessons) != 14 or set(rows) != {item["id"] for item in lessons}:
        raise SystemExit("Expected exactly fourteen lessons with fourteen matrix rows")
    out = [
        "# AGENTS.md — Paper Graph governed decision rules",
        "",
        "Generator: " + GENERATOR_ID + ".",
        "Sources: paper-graph/knowledge-system/lessons.jsonl and paper-graph/docs/rule-check-matrix.md.",
        "Scope: these are the only encoded rules available in variant B; no full-document fallback is implied.",
        "Apply the narrowest matching lesson. Preserve every stated boundary and unknown; never infer an implementation, measurement, human adjudication, locator, or usage value. Prefer a fail-closed action and name the listed deterministic check when one applies. A human-judgment entry never licenses an agent to make that judgment.",
        "",
    ]
    for item in lessons:
        row = rows[item["id"]]
        out += [
            "## " + item["id"] + " — " + item["title"],
            "",
            "Rule: " + item["rule"],
            "",
            "Boundary: " + item["boundary"],
            "",
            "Enforcement: " + item["enforcement"],
            "",
            "Rejected alternative: " + item["rejected_alternative"],
            "",
            "Matrix rule: " + row["matrix_rule"],
            "Owner: " + row["owner"] + ".",
            "Canonical owning document(s): " + (", ".join("paper-graph/" + doc for doc in row["docs"]) if row["docs"] else "none listed") + ".",
            "Check type: " + row["check_type"] + ".",
            "Deterministic check id(s): " + (", ".join(row["checks"]) if row["checks"] else "none; human judgment required") + ".",
            "Human judgment explicitly required: " + ("yes" if row["human_judgment"] else "no") + ".",
            "",
        ]
    (RUN / "AGENTS.md").write_text("\n".join(out), encoding="utf-8")
    print("Generated AGENTS.md from " + str(len(lessons)) + " lessons")


if __name__ == "__main__":
    main()
