use paper_graph::parse::{
    admit_corpus, load_corpus_manifest, read_admission_report, write_admission_report,
    zotero_storage_root, AdmissionStatus, CorpusManifest, CorpusPaper,
};
use paper_graph::zotero::{default_zotero_db, read_library, Attachment, Library, Work};
use std::fs;
use std::path::PathBuf;

fn sample_manifest() -> CorpusManifest {
    CorpusManifest {
        schema_version: "round-001-corpus-v1".to_string(),
        status: "test".to_string(),
        verified_on: "2026-11-19".to_string(),
        verification: vec!["test".to_string()],
        papers: vec![
            CorpusPaper {
                item_id: 101,
                split: "development".to_string(),
                domain: "type_inference".to_string(),
                title: "Valid PDF Paper".to_string(),
            },
            CorpusPaper {
                item_id: 102,
                split: "development".to_string(),
                domain: "effects_control".to_string(),
                title: "Seeded Missing File Paper".to_string(),
            },
            CorpusPaper {
                item_id: 103,
                split: "test".to_string(),
                domain: "recursion_schemes".to_string(),
                title: "Seeded Bad Signature Paper".to_string(),
            },
            CorpusPaper {
                item_id: 104,
                split: "test".to_string(),
                domain: "dependent_lean".to_string(),
                title: "Seeded Missing Work Paper".to_string(),
            },
        ],
    }
}

#[test]
fn seeded_defects_emit_typed_failure_records_and_never_skip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let storage = dir.path().join("storage");

    // 101: valid %PDF- file
    let key101 = storage.join("KEY00101");
    fs::create_dir_all(&key101).unwrap();
    fs::write(key101.join("valid.pdf"), b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n").unwrap();

    // 102: directory exists, PDF file is missing
    let key102 = storage.join("KEY00102");
    fs::create_dir_all(&key102).unwrap();

    // 103: file exists, bad signature (HTML error page saved as .pdf)
    let key103 = storage.join("KEY00103");
    fs::create_dir_all(&key103).unwrap();
    fs::write(key103.join("bad.pdf"), b"<html>not a pdf</html>").unwrap();

    let library = Library {
        works: vec![
            Work {
                id: 101,
                key: "WKEY0101".to_string(),
                item_type: "journalArticle".to_string(),
                title: "Valid PDF Paper".to_string(),
                ..Work::default()
            },
            Work {
                id: 102,
                key: "WKEY0102".to_string(),
                item_type: "journalArticle".to_string(),
                title: "Seeded Missing File Paper".to_string(),
                ..Work::default()
            },
            Work {
                id: 103,
                key: "WKEY0103".to_string(),
                item_type: "journalArticle".to_string(),
                title: "Seeded Bad Signature Paper".to_string(),
                ..Work::default()
            },
        ],
        attachments: vec![
            Attachment {
                id: 201,
                key: "KEY00101".to_string(),
                parent_item_id: 101,
                path: Some("storage:valid.pdf".to_string()),
                content_type: Some("application/pdf".to_string()),
            },
            Attachment {
                id: 202,
                key: "KEY00102".to_string(),
                parent_item_id: 102,
                path: Some("storage:missing.pdf".to_string()),
                content_type: Some("application/pdf".to_string()),
            },
            Attachment {
                id: 203,
                key: "KEY00103".to_string(),
                parent_item_id: 103,
                path: Some("storage:bad.pdf".to_string()),
                content_type: Some("application/pdf".to_string()),
            },
        ],
    };

    let manifest = sample_manifest();
    let report = admit_corpus(&manifest, &library, &storage).expect("admit_corpus succeeds");

    // Never skip: all 4 corpus papers must have a typed record
    assert_eq!(report.total_papers, 4);
    assert_eq!(report.records.len(), 4);
    assert_eq!(report.admitted_count, 1);
    assert_eq!(report.failed_count, 3);

    assert_eq!(report.records[0].item_id, 101);
    assert_eq!(report.records[0].status, AdmissionStatus::Admitted);
    assert!(report.records[0].signature_ok);
    assert_eq!(report.records[0].sha256.as_ref().map(String::len), Some(64));

    assert_eq!(report.records[1].item_id, 102);
    assert_eq!(report.records[1].status, AdmissionStatus::MissingFile);
    assert!(!report.records[1].signature_ok);
    assert!(report.records[1].error.as_deref().unwrap_or_default().contains("file not found"));

    assert_eq!(report.records[2].item_id, 103);
    assert_eq!(report.records[2].status, AdmissionStatus::InvalidSignature);
    assert!(!report.records[2].signature_ok);
    // SHA-256 and byte count are still recorded on a bad-signature file
    assert_eq!(report.records[2].bytes, Some(22));
    assert_eq!(report.records[2].sha256.as_ref().map(String::len), Some(64));

    assert_eq!(report.records[3].item_id, 104);
    assert_eq!(report.records[3].status, AdmissionStatus::MissingWork);

    // Roundtrip serde write + read
    let out_file = dir.path().join("admission.json");
    write_admission_report(&out_file, &report).expect("write report");
    let reloaded = read_admission_report(&out_file).expect("read report");
    assert_eq!(reloaded, report);

    // Seeded defect on validator: claiming Admitted when signature_ok is false MUST fail validation
    let mut tampered = reloaded.clone();
    tampered.records[0].signature_ok = false;
    let err = tampered
        .validate()
        .expect_err("seeded defect: admitted with signature_ok=false must fail");
    assert!(err.to_string().contains("signature_ok=false"));

    // Seeded defect on validator: silently dropping a failed record MUST fail validation
    let mut dropped = reloaded;
    dropped.records.pop();
    let err = dropped
        .validate()
        .expect_err("seeded defect: dropping a paper record must fail");
    assert!(err.to_string().contains("record count"));
}

#[test]
fn corpus_15_papers_admitted_against_local_zotero_when_present() {
    let zotero_db = default_zotero_db();
    if !zotero_db.is_file() {
        return;
    }
    let corpus_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("experiments")
        .join("ontology-extraction")
        .join("round-001-corpus.json");
    let corpus = load_corpus_manifest(&corpus_path).expect("load corpus");
    assert_eq!(corpus.papers.len(), 15);

    let library = read_library(&zotero_db).expect("read zotero snapshot");
    let storage = zotero_storage_root(&zotero_db);
    let report = admit_corpus(&corpus, &library, &storage).expect("admit 15 corpus papers");

    assert_eq!(report.total_papers, 15);
    assert_eq!(report.admitted_count, 15, "expected 15/15 admitted: {report:#?}");
    assert_eq!(report.failed_count, 0);
    for rec in &report.records {
        assert_eq!(rec.status, AdmissionStatus::Admitted);
        assert!(rec.signature_ok);
        assert!(rec.bytes.unwrap_or(0) > 100);
        assert_eq!(rec.sha256.as_ref().map(String::len), Some(64));
    }
}
