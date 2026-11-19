#!/usr/bin/env python3
"""Freeze all W5 inputs before any model prediction."""
from __future__ import annotations
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN = ROOT / "_runs/2026-11-19/docs-eval"
RUN_ID = "w5.4-2026-11-20-magpie-deepseek-flash-r2"
A_DOCS = [
    "knowledge-system/CONSTITUTION.md",
    "knowledge-system/method.md",
    "knowledge-system/lessons.md",
    "knowledge-system/ontology.md",
    "knowledge-system/PRAXIS.md",
    "knowledge-system/docs/owner-map.md",
    "docs/rule-check-matrix.md",
    "docs/ARCHITECTURE.md",
    "docs/DATA_MODEL.md",
    "docs/PIPELINE.md",
    "docs/EVALUATION.md",
    "docs/OPERATIONS.md",
    "docs/EXPERIMENTS.md",
    "docs/ROADMAP.md",
    "docs/REVIEW.md",
]


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def manifest_bytes(paths: list[str]) -> bytes:
    return "".join(sha(ROOT / rel) + "  " + rel + "\n" for rel in sorted(paths, key=lambda item: item.encode("utf-8"))).encode("utf-8")


def main() -> None:
    if list((RUN / "raw").glob("*/*.json")):
        raise SystemExit("Raw predictions already exist; refusing to freeze a new run")
    previous_config = RUN / "config.json"
    if previous_config.exists():
        previous = json.loads(previous_config.read_text(encoding="utf-8"))
        if previous.get("run_permitted", True):
            raise SystemExit("A permitted config already exists; refusing to re-freeze")
        for name in ("config.json", "report.md", "scores.json"):
            source = RUN / name
            if source.exists():
                archive = RUN / name.replace(".", ".aborted-r1.", 1)
                if not archive.exists():
                    archive.write_bytes(source.read_bytes())
    prompt_path = ROOT / "docs/EVAL-PROMPT.md"
    prompt_text = prompt_path.read_text(encoding="utf-8")
    common = prompt_text.split("## 3. BEGIN FROZEN COMMON PROMPT\n", 1)[1].split("## 4. END FROZEN COMMON PROMPT", 1)[0].strip("\n").encode("utf-8") + b"\n"
    (RUN / "common-prompt.txt").write_bytes(common)
    variant_a = bytearray()
    for rel in A_DOCS:
        source = (ROOT / rel).read_bytes()
        variant_a += ("===== BEGIN paper-graph/" + rel + " =====\n").encode("utf-8")
        variant_a += source
        if not source.endswith(b"\n"):
            variant_a += b"\n"
        variant_a += ("===== END paper-graph/" + rel + " =====\n\n").encode("utf-8")
    (RUN / "variant-a.txt").write_bytes(variant_a)
    variant_b = (RUN / "AGENTS.md").read_bytes()
    variant_c = bytes(variant_a) + variant_b
    doc_paths = [str(path.relative_to(ROOT)) for path in (ROOT / "docs").glob("*.md")]
    doc_paths += [str(path.relative_to(ROOT)) for path in (ROOT / "knowledge-system").glob("*.md")]
    docs_manifest = manifest_bytes(doc_paths)
    (RUN / "docs.sha256").write_bytes(docs_manifest)
    seed_paths = sorted((ROOT / "knowledge-system/eval-seeds").glob("*.json"), key=lambda path: str(path).encode("utf-8"))
    if len(seed_paths) != 15:
        raise SystemExit("Expected fifteen seeds")
    seed_rels = [str(path.relative_to(ROOT)) for path in seed_paths]
    seeds = [json.loads(path.read_text(encoding="utf-8")) for path in seed_paths]
    run_prefix = "_runs/2026-11-19/docs-eval/"
    input_paths = set(A_DOCS + seed_rels + [
        "docs/EVAL-PROMPT.md",
        "knowledge-system/lessons.jsonl",
        run_prefix + "common-prompt.txt",
        run_prefix + "variant-a.txt",
        run_prefix + "AGENTS.md",
        run_prefix + "docs.sha256",
        run_prefix + "scripts/generate_rules.py",
        run_prefix + "scripts/freeze.py",
        run_prefix + "scripts/run_eval.py",
        run_prefix + "scripts/score.py",
    ])
    inputs_manifest = manifest_bytes(sorted(input_paths))
    (RUN / "inputs.sha256").write_bytes(inputs_manifest)
    config = {
        "schema_version": "docs-eval-config-v1",
        "run_id": RUN_ID,
        "frozen_at": datetime.now(timezone.utc).isoformat(),
        "run_permitted": True,
        "abort_reason": None,
        "supersedes": "W0 baseline docs.sha256 (2026-11-19)",
        "supersedes_reason": "The legacy W0 manifest predates legitimate W1/W3/T3.9 documentation changes. The run-start freeze is authoritative; no comparison against the legacy manifest is performed.",
        "docs_manifest": {
            "path": run_prefix + "docs.sha256",
            "sha256": sha(RUN / "docs.sha256"),
            "entries": len(doc_paths),
            "globs": ["docs/*.md", "knowledge-system/*.md"],
        },
        "run_input_manifest": run_prefix + "inputs.sha256",
        "run_input_manifest_sha256": sha(RUN / "inputs.sha256"),
        "prompt": {
            "path": "docs/EVAL-PROMPT.md",
            "sha256": sha(prompt_path),
            "common_prompt_path": run_prefix + "common-prompt.txt",
            "common_prompt_sha256": sha_bytes(common),
        },
        "variants": {
            "A": {
                "description": "full docs",
                "path": run_prefix + "variant-a.txt",
                "sha256": sha_bytes(bytes(variant_a)),
                "allowlist": A_DOCS,
                "allowlist_hashes": {rel: sha(ROOT / rel) for rel in A_DOCS},
                "owner_map_resolution": "EVAL-PROMPT docs/owner-map.md resolved to existing knowledge-system/docs/owner-map.md",
            },
            "B": {
                "description": "encoded rules only",
                "path": run_prefix + "AGENTS.md",
                "sha256": sha_bytes(variant_b),
                "generator": "paper-graph-docs-eval-rule-generator-v1",
                "generator_path": run_prefix + "scripts/generate_rules.py",
                "generator_sha256": sha(RUN / "scripts/generate_rules.py"),
                "source_lessons_path": "knowledge-system/lessons.jsonl",
                "source_lessons_sha256": sha(ROOT / "knowledge-system/lessons.jsonl"),
                "source_matrix_path": "docs/rule-check-matrix.md",
                "source_matrix_sha256": sha(ROOT / "docs/rule-check-matrix.md"),
            },
            "C": {
                "description": "identical bytes A followed by B",
                "c_bytes_equal_a_plus_b": True,
                "combined_sha256": sha_bytes(variant_c),
            },
        },
        "seeds": {rel: sha(ROOT / rel) for rel in seed_rels},
        "scenario_order": [seed["scenario_id"] for seed in seeds],
        "variant_order": ["A", "B", "C"],
        "scorer": {
            "version": "paper-graph-docs-eval-deterministic-scorer-v1",
            "path": run_prefix + "scripts/score.py",
            "sha256": sha(RUN / "scripts/score.py"),
        },
        "execution": {
            "provider": "magpie",
            "family": "deepseek",
            "family_label": "single-family",
            "model": "deepseek/deepseek-flash",
            "base_url": "http://127.0.0.1:3425/v1",
            "wire": "openai-completions",
            "decoding": {"temperature": 0, "max_tokens": 2048, "stream": False},
            "context_window_tokens": 1048576,
            "route_max_output_tokens": 393216,
            "timeout_seconds": 120,
            "preflight_timeout_seconds": 15,
            "concurrency": 3,
            "retry_policy": {"max_retries": 2, "backoff_seconds": 2, "retryable": ["network error", "HTTP 429", "HTTP >=500"]},
            "planned_matrix_calls": 45,
            "spend_cap_usd": 5,
            "pricing": {
                "estimate_only": True,
                "currency": "USD",
                "input_per_million": 0.15,
                "output_per_million": 0.60,
                "cached_read_per_million": 0.003,
                "source": "worklog/2026-11-19/todo-model-worktree.md Magpie catalog snapshot (2026-11-19)"
            },
            "missing_usage": "unreported",
        },
        "gate": {
            "scope": "all 45 cases with per-variant cells",
            "decision_accuracy_min": 0.90,
            "forbidden_incidents_max": 0,
            "checks_resolvable_min": 0.80,
            "ambiguous_unknowns": "3/3 in each variant, 9/9 overall",
            "execution_accounting_required": True,
        },
    }
    (RUN / "config.json").write_text(json.dumps(config, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({"run_id": RUN_ID, "docs_entries": len(doc_paths), "input_entries": len(input_paths), "variant_bytes": {key: len(value) for key, value in (("A", variant_a), ("B", variant_b), ("C", variant_c))}, "legacy_comparison": "superseded; not performed"}, indent=2))


if __name__ == "__main__":
    main()
