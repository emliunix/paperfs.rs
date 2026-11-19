use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvalManifest {
    schema_version: u32,
    run_id: String,
    commit: String,
    corpus_size: usize,
    corpus_hash: String,
    chunk_hash: String,
    competency_questions_hash: String,
    prompt_hash: String,
    schema_hash: String,
    model: String,
    extra_body: serde_json::Value,
    repeat_index: u32,
    decoding: Decoding,
    call_budget_preflight: String,
    scorer_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Decoding {
    temperature: f64,
    seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ExecutionStatus {
    Executed,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaseResult {
    case_id: String,
    status: ExecutionStatus,
    expected: String,
    actual: String,
    raw_output: String,
}

#[derive(Debug, PartialEq, Eq)]
struct EvalReport {
    executed: usize,
    skipped: usize,
    failed: usize,
}

fn validate_manifest(manifest: &EvalManifest) -> Result<()> {
    if manifest.schema_version != 1 {
        bail!("unsupported eval manifest schema_version");
    }
    if manifest.corpus_size == 0 {
        bail!("eval corpus_size must be non-zero");
    }
    for (field, value) in [
        ("run_id", &manifest.run_id),
        ("commit", &manifest.commit),
        ("corpus_hash", &manifest.corpus_hash),
        ("chunk_hash", &manifest.chunk_hash),
        (
            "competency_questions_hash",
            &manifest.competency_questions_hash,
        ),
        ("prompt_hash", &manifest.prompt_hash),
        ("schema_hash", &manifest.schema_hash),
        ("model", &manifest.model),
        ("call_budget_preflight", &manifest.call_budget_preflight),
        ("scorer_version", &manifest.scorer_version),
    ] {
        if value.trim().is_empty() {
            bail!("eval manifest field {field} must not be empty");
        }
    }
    if !manifest.extra_body.is_object() {
        bail!("eval manifest extra_body must record the request configuration object");
    }
    if !manifest.decoding.temperature.is_finite() || manifest.decoding.temperature < 0.0 {
        bail!("eval manifest decoding temperature must be finite and non-negative");
    }
    Ok(())
}

fn run_eval(manifest: &EvalManifest, results: &[CaseResult]) -> Result<EvalReport> {
    validate_manifest(manifest)?;
    let mut case_ids = BTreeSet::new();
    let mut report = EvalReport {
        executed: 0,
        skipped: 0,
        failed: 0,
    };
    for result in results {
        if result.case_id.trim().is_empty() || !case_ids.insert(&result.case_id) {
            bail!("eval case IDs must be non-empty and unique");
        }
        match result.status {
            ExecutionStatus::Executed => {
                report.executed += 1;
                if result.expected.trim().is_empty()
                    || result.actual.trim().is_empty()
                    || result.raw_output.trim().is_empty()
                {
                    bail!(
                        "executed case {} lacks expected/actual/raw output",
                        result.case_id
                    );
                }
            }
            ExecutionStatus::Skipped => report.skipped += 1,
            ExecutionStatus::Failed => report.failed += 1,
        }
    }
    if report.skipped > 0 {
        bail!(
            "eval skip exits non-zero: {} skipped case(s)",
            report.skipped
        );
    }
    if report.failed > 0 {
        bail!(
            "eval failed cases exit non-zero: {} failed case(s)",
            report.failed
        );
    }
    if report.executed != manifest.corpus_size || results.len() != manifest.corpus_size {
        bail!(
            "executed != corpus: executed={} records={} corpus={}",
            report.executed,
            results.len(),
            manifest.corpus_size
        );
    }
    Ok(report)
}

fn eval_exit_code(result: &Result<EvalReport>) -> i32 {
    if result.is_ok() {
        0
    } else {
        1
    }
}

fn manifest() -> EvalManifest {
    EvalManifest {
        schema_version: 1,
        run_id: "run-001".to_string(),
        commit: "test-commit".to_string(),
        corpus_size: 2,
        corpus_hash: "corpus-sha256".to_string(),
        chunk_hash: "chunk-sha256".to_string(),
        competency_questions_hash: "questions-sha256".to_string(),
        prompt_hash: "prompt-sha256".to_string(),
        schema_hash: "schema-sha256".to_string(),
        model: "frozen-model".to_string(),
        extra_body: serde_json::json!({"thinking": false}),
        repeat_index: 0,
        decoding: Decoding {
            temperature: 0.0,
            seed: 7,
        },
        call_budget_preflight: "pass".to_string(),
        scorer_version: "scorer-v1".to_string(),
    }
}

fn results() -> Vec<CaseResult> {
    ["case-a", "case-b"]
        .into_iter()
        .map(|case_id| CaseResult {
            case_id: case_id.to_string(),
            status: ExecutionStatus::Executed,
            expected: "BLOCKED".to_string(),
            actual: "BLOCKED".to_string(),
            raw_output: format!("raw output for {case_id}"),
        })
        .collect()
}

#[test]
fn complete_run_matches_corpus_and_manifest_round_trips() {
    let manifest = manifest();
    let encoded = serde_json::to_string(&manifest).expect("serialize manifest");
    let decoded: EvalManifest = serde_json::from_str(&encoded).expect("manifest schema round trip");
    let result = run_eval(&decoded, &results());
    assert_eq!(eval_exit_code(&result), 0);
    assert_eq!(
        result.expect("complete run"),
        EvalReport {
            executed: 2,
            skipped: 0,
            failed: 0,
        }
    );
}

#[test]
fn seeded_missing_manifest_field_is_rejected() {
    let mut json = serde_json::to_value(manifest()).expect("manifest JSON");
    json.as_object_mut()
        .expect("manifest object")
        .remove("model");
    let error = serde_json::from_value::<EvalManifest>(json)
        .expect_err("seeded missing model must fail manifest schema");
    assert!(error.to_string().contains("model"), "{error}");
}

#[test]
fn seeded_skip_exits_non_zero() {
    let mut results = results();
    results[1].status = ExecutionStatus::Skipped;
    let result = run_eval(&manifest(), &results);
    assert_eq!(eval_exit_code(&result), 1);
    let error = result.expect_err("seeded skip must fail");
    assert!(
        error.to_string().contains("skip exits non-zero"),
        "{error:#}"
    );
}

#[test]
fn seeded_execution_count_mismatch_is_rejected() {
    let results = &results()[..1];
    let error = run_eval(&manifest(), results).expect_err("one of two cases is not evidence");
    assert!(
        error.to_string().contains("executed != corpus"),
        "{error:#}"
    );
}

#[test]
fn seeded_duplicate_case_id_is_rejected() {
    let mut results = results();
    results[1].case_id = results[0].case_id.clone();
    let error = run_eval(&manifest(), &results).expect_err("duplicate IDs must not inflate counts");
    assert!(error.to_string().contains("unique"), "{error:#}");
}
