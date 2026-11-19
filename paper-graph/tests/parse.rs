use paper_graph::parse::{
    admit_corpus, detect_system_pdf_tool, load_corpus_manifest, parse_admitted_corpus,
    read_pages_jsonl, write_pages_jsonl, zotero_storage_root, AdmissionRecord, AdmissionReport,
    AdmissionStatus, PageParseStatus, ADMISSION_SCHEMA_VERSION,
};
use paper_graph::zotero::{default_zotero_db, read_library};
use std::fs;
use std::path::PathBuf;

#[test]
fn seeded_corrupt_pdf_records_typed_parse_failure_and_bad_schema_fails_validation() {
    let dir = tempfile::tempdir().expect("tempdir");
    let corrupt_pdf = dir.path().join("corrupt.pdf");
    // Starts with %PDF- so Stage 1 admits it, but body is truncated garbage so PDF parser fails
    let bytes = b"%PDF-1.4\ngarbage_not_a_valid_xref_table";
    fs::write(&corrupt_pdf, bytes).unwrap();
    let (_, len, sig_ok, sha256, _) = paper_graph::parse::inspect_pdf_file(&corrupt_pdf);
    assert!(sig_ok);

    let admission = AdmissionReport {
        schema_version: ADMISSION_SCHEMA_VERSION.to_string(),
        corpus_schema_version: "round-001-corpus-v1".to_string(),
        storage_root: dir.path().display().to_string(),
        total_papers: 1,
        admitted_count: 1,
        failed_count: 0,
        records: vec![AdmissionRecord {
            item_id: 999,
            zotero_key: Some("CORRUPT1".to_string()),
            split: "development".to_string(),
            domain: "type_inference".to_string(),
            title: "Corrupt PDF Fixture".to_string(),
            attachment_id: Some(1999),
            attachment_key: Some("ATT1999".to_string()),
            path: Some(corrupt_pdf.display().to_string()),
            bytes: len,
            signature_ok: true,
            sha256,
            status: AdmissionStatus::Admitted,
            error: None,
            candidates_checked: vec![],
        }],
    };

    let tool = detect_system_pdf_tool().expect("system pdf tool available");
    let (pages, report) =
        parse_admitted_corpus(&admission, Some("development"), &tool).expect("parse handles failure");

    assert_eq!(report.target_papers, 1);
    assert_eq!(report.succeeded_papers, 0);
    assert_eq!(report.failed_papers, 1);
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].status, PageParseStatus::Failed);
    assert!(pages[0].error.is_some());

    let pages_path = dir.path().join("pages.jsonl");
    write_pages_jsonl(&pages_path, &pages).expect("write failed page record");
    let reloaded = read_pages_jsonl(&pages_path).expect("reload pages.jsonl");
    assert_eq!(reloaded, pages);

    // Seeded defect: a status=ok PageRecord with empty text MUST be rejected before write
    let mut bad_page = pages[0].clone();
    bad_page.status = PageParseStatus::Ok;
    bad_page.page = 1;
    bad_page.text = "   ".to_string();
    let err = write_pages_jsonl(&pages_path, &[bad_page])
        .expect_err("seeded defect: status=ok with empty text must fail");
    assert!(format!("{err:#}").contains("empty text"));
}

#[test]
fn parses_all_four_development_pdfs_with_page_numbers() {
    let zotero_db = default_zotero_db();
    if !zotero_db.is_file() {
        return;
    }
    let corpus_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("experiments")
        .join("ontology-extraction")
        .join("round-001-corpus.json");
    let corpus = load_corpus_manifest(&corpus_path).expect("load corpus");
    let library = read_library(&zotero_db).expect("read zotero");
    let storage = zotero_storage_root(&zotero_db);
    let admission = admit_corpus(&corpus, &library, &storage).expect("admit corpus");

    let tool = detect_system_pdf_tool().expect("detect system PDF tool");
    let (pages, report) =
        parse_admitted_corpus(&admission, Some("development"), &tool).expect("parse 4 dev PDFs");

    assert_eq!(report.target_papers, 4);
    assert_eq!(report.succeeded_papers, 4, "expected 4/4 parsed: {report:#?}");
    assert_eq!(report.failed_papers, 0);
    assert!(pages.len() >= 100, "expected >100 pages across the 4 dev PDFs");
    for summary in &report.papers {
        assert!(summary.parsed_ok);
        assert!(summary.total_pages >= 10);
        assert!(summary.total_chars > 10_000);
    }
    // Check 1-based contiguous page numbers per paper
    for summary in &report.papers {
        let paper_pages: Vec<usize> = pages
            .iter()
            .filter(|p| p.paper_id == summary.paper_id)
            .map(|p| p.page)
            .collect();
        let expected: Vec<usize> = (1..=summary.total_pages).collect();
        assert_eq!(paper_pages, expected);
    }
}
