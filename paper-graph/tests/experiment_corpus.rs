use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

#[test]
fn round_001_corpus_is_disjoint_and_stratified() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("experiments")
        .join("ontology-extraction")
        .join("round-001-corpus.json");
    let corpus: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("read corpus"))
        .expect("parse corpus");
    let papers = corpus["papers"].as_array().expect("papers array");

    assert_eq!(corpus["status"], "pdf_verified_gold_pending");
    assert_eq!(papers.len(), 15);
    let ids: HashSet<i64> = papers
        .iter()
        .map(|paper| paper["item_id"].as_i64().expect("numeric item_id"))
        .collect();
    assert_eq!(ids.len(), 15);

    let development: Vec<&Value> = papers
        .iter()
        .filter(|paper| paper["split"] == "development")
        .collect();
    let test: Vec<&Value> = papers
        .iter()
        .filter(|paper| paper["split"] == "test")
        .collect();
    assert_eq!(development.len(), 4);
    assert_eq!(test.len(), 11);

    let development_ids: HashSet<i64> = development
        .iter()
        .map(|paper| paper["item_id"].as_i64().expect("numeric item_id"))
        .collect();
    let test_ids: HashSet<i64> = test
        .iter()
        .map(|paper| paper["item_id"].as_i64().expect("numeric item_id"))
        .collect();
    assert!(development_ids.is_disjoint(&test_ids));

    let domains: HashSet<&str> = [
        "type_inference",
        "recursion_schemes",
        "effects_control",
        "dependent_lean",
    ]
    .into_iter()
    .collect();
    assert_eq!(
        development
            .iter()
            .map(|paper| paper["domain"].as_str().unwrap_or_default())
            .collect::<HashSet<_>>(),
        domains
    );
    assert_eq!(
        test.iter()
            .map(|paper| paper["domain"].as_str().unwrap_or_default())
            .collect::<HashSet<_>>(),
        domains
    );
    assert!(papers
        .iter()
        .all(|paper| !paper["title"].as_str().unwrap_or_default().is_empty()));
}
