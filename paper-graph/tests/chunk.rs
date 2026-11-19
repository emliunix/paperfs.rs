use paper_graph::chunk::{
    chunk_pages, read_chunks_jsonl, reconstruct_document_text, write_chunks_jsonl, ChunkRecord,
    DEFAULT_MAX_CHUNK_CHARS,
};
use paper_graph::parse::{
    PageOrderMetadata, PageParseStatus, PageRecord, PAGE_SCHEMA_VERSION,
};
use std::fs;

fn synthetic_pages() -> Vec<PageRecord> {
    let p1_text = "Sample Paper Title\nAda Lovelace\nAbstract\nWe prove soundness of bidirectional typing.\n1. Introduction\nBidirectional typechecking splits synthesis and checking.";
    let p2_text = "2. Formal System\nTheorem 1 (Soundness). Every well-typed term is safe.\n2.1 Metatheory\nBy induction on the typing derivation.\nReferences\n[1] Dunfield and Krishnaswami 2013.";
    vec![
        PageRecord {
            schema_version: PAGE_SCHEMA_VERSION.to_string(),
            paper_id: 42,
            attachment_id: 420,
            attachment_sha256: "a".repeat(64),
            page: 1,
            text: p1_text.to_string(),
            order_metadata: PageOrderMetadata {
                page_index_zero_based: 0,
                total_pages_in_document: 2,
                reading_order_mode: "synthetic".to_string(),
                char_count: p1_text.chars().count(),
                line_count: p1_text.lines().count(),
                replacement_char_count: 0,
            },
            parser_version: "test-parser:1.0".to_string(),
            status: PageParseStatus::Ok,
            error: None,
        },
        PageRecord {
            schema_version: PAGE_SCHEMA_VERSION.to_string(),
            paper_id: 42,
            attachment_id: 420,
            attachment_sha256: "a".repeat(64),
            page: 2,
            text: p2_text.to_string(),
            order_metadata: PageOrderMetadata {
                page_index_zero_based: 1,
                total_pages_in_document: 2,
                reading_order_mode: "synthetic".to_string(),
                char_count: p2_text.chars().count(),
                line_count: p2_text.lines().count(),
                replacement_char_count: 0,
            },
            parser_version: "test-parser:1.0".to_string(),
            status: PageParseStatus::Ok,
            error: None,
        },
    ]
}

#[test]
fn chunk_contract_is_deterministic_and_resolves_offsets_and_sections() {
    let pages = synthetic_pages();
    let (chunks_a, report_a) = chunk_pages(&pages, DEFAULT_MAX_CHUNK_CHARS).expect("chunk run A");
    let (chunks_b, _report_b) = chunk_pages(&pages, DEFAULT_MAX_CHUNK_CHARS).expect("chunk run B");

    // Deterministic hashes and offsets across two runs
    assert_eq!(chunks_a, chunks_b);
    assert_eq!(report_a.succeeded_papers, 1);
    assert_eq!(report_a.failed_papers, 0);
    assert!(chunks_a.len() >= 5);

    let refs: Vec<&PageRecord> = pages.iter().collect();
    let doc_text = reconstruct_document_text(&refs);
    let doc_chars: Vec<char> = doc_text.chars().collect();

    for (expected_ord, chunk) in chunks_a.iter().enumerate() {
        assert_eq!(chunk.ordinal, expected_ord);
        assert_eq!(chunk.attachment_id, 420);
        assert_eq!(chunk.parser_version, "test-parser:1.0");
        assert_eq!(chunk.content_hash, ChunkRecord::compute_content_hash(&chunk.raw_text));
        let slice: String = doc_chars[chunk.start_offset..chunk.end_offset]
            .iter()
            .collect();
        assert_eq!(slice, chunk.raw_text);
    }

    // Verify hierarchical section paths (["2. Formal System", "2.1 Metatheory"])
    assert!(chunks_a
        .iter()
        .any(|c| c.section_path == vec!["2. Formal System", "2.1 Metatheory"]));
    assert!(chunks_a
        .iter()
        .any(|c| c.section_path == vec!["References"]));

    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("chunks.jsonl");
    write_chunks_jsonl(&out, &chunks_a).expect("write chunks.jsonl");
    let reloaded = read_chunks_jsonl(&out).expect("read chunks.jsonl");
    assert_eq!(reloaded, chunks_a);
}

#[test]
fn seeded_chunk_defects_fail_serde_and_contract_validation_before_write() {
    let pages = synthetic_pages();
    let (chunks, _) = chunk_pages(&pages, DEFAULT_MAX_CHUNK_CHARS).expect("chunk");
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("bad_chunks.jsonl");

    // Seeded defect 1: tampered content_hash
    let mut bad_hash = chunks.clone();
    bad_hash[0].content_hash = "0".repeat(64);
    let err = write_chunks_jsonl(&out, &bad_hash)
        .expect_err("seeded defect: mismatched content_hash must fail before write");
    assert!(format!("{err:#}").contains("content_hash mismatch"));
    assert!(!out.exists(), "invalid chunks must not write output file");

    // Seeded defect 2: broken offset span (end_offset - start_offset != char len)
    let mut bad_offsets = chunks.clone();
    bad_offsets[0].end_offset += 1;
    let err = write_chunks_jsonl(&out, &bad_offsets)
        .expect_err("seeded defect: drifted end_offset must fail before write");
    assert!(format!("{err:#}").contains("offset span"));

    // Seeded defect 3: non-contiguous ordinal
    let mut bad_ordinal = chunks;
    bad_ordinal[1].ordinal = 5;
    bad_ordinal[1].chunk_id = ChunkRecord::format_chunk_id(
        bad_ordinal[1].attachment_id,
        &bad_ordinal[1].parser_version,
        5,
        &bad_ordinal[1].content_hash,
    );
    let err = write_chunks_jsonl(&out, &bad_ordinal)
        .expect_err("seeded defect: skipped ordinal must fail before write");
    assert!(format!("{err:#}").contains("non-contiguous ordinal"));

    // Seeded defect 4: unknown JSON field rejected by #[serde(deny_unknown_fields)]
    let unknown_field_path = dir.path().join("unknown_field.jsonl");
    fs::write(
        &unknown_field_path,
        r#"{"schema_version":"stage-3-chunks-v1","chunk_id":"x","paper_id":1,"attachment_id":1,"parser_version":"v","chunker_version":"c","page_range":[1,1],"section_path":["Abstract"],"ordinal":0,"content_hash":"0000000000000000000000000000000000000000000000000000000000000000","raw_text":"hi","start_offset":0,"end_offset":2,"unexpected":"boom"}"#,
    )
    .unwrap();
    let err = read_chunks_jsonl(&unknown_field_path)
        .expect_err("seeded defect: unknown field must fail serde deserialization");
    assert!(format!("{err:#}").contains("unknown field"));
}
