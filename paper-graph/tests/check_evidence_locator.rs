use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Chunk {
    attachment_item_id: i64,
    chunk_id: String,
    page: u32,
    raw_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct EvidenceSpan {
    attachment_item_id: i64,
    chunk_id: String,
    page: u32,
    quote: String,
    start_char: usize,
    end_char: usize,
}

fn byte_offset(text: &str, character_offset: usize) -> Result<usize> {
    if character_offset == text.chars().count() {
        return Ok(text.len());
    }
    text.char_indices()
        .nth(character_offset)
        .map(|(offset, _)| offset)
        .ok_or_else(|| anyhow::anyhow!("character offset {character_offset} is out of bounds"))
}

/// Exact-substring validator stub against test fixtures. Production pages and
/// chunks do not exist yet; this deliberately performs no fuzzy normalization.
fn validate_evidence(span: &EvidenceSpan, chunks: &[Chunk]) -> Result<()> {
    if span.quote.is_empty() || span.start_char >= span.end_char {
        bail!("evidence quote and character range must be non-empty");
    }
    let chunk = chunks
        .iter()
        .find(|chunk| {
            chunk.attachment_item_id == span.attachment_item_id
                && chunk.chunk_id == span.chunk_id
                && chunk.page == span.page
        })
        .ok_or_else(|| anyhow::anyhow!("evidence locator does not resolve to a chunk/page"))?;
    let start = byte_offset(&chunk.raw_text, span.start_char)?;
    let end = byte_offset(&chunk.raw_text, span.end_char)?;
    let located = &chunk.raw_text[start..end];
    if located != span.quote || !chunk.raw_text.contains(&span.quote) {
        bail!("evidence quote is not an exact substring at the supplied locator");
    }
    if span.end_char - span.start_char != span.quote.chars().count() {
        bail!("evidence character range length does not match quote length");
    }
    Ok(())
}

fn fixture() -> (Vec<Chunk>, EvidenceSpan) {
    let text = "Prefix λ calculus: the result holds.
Suffix"
        .to_string();
    let quote = "λ calculus: the result holds.".to_string();
    let start_char = "Prefix ".chars().count();
    let chunks = vec![Chunk {
        attachment_item_id: 11,
        chunk_id: "chunk-0001".to_string(),
        page: 3,
        raw_text: text,
    }];
    let span = EvidenceSpan {
        attachment_item_id: 11,
        chunk_id: "chunk-0001".to_string(),
        page: 3,
        end_char: start_char + quote.chars().count(),
        quote,
        start_char,
    };
    (chunks, span)
}

#[test]
fn exact_quote_and_character_locator_resolve() {
    let (chunks, span) = fixture();
    validate_evidence(&span, &chunks).expect("valid exact evidence span");

    // Keep the fixture honest: the locator is a range over Unicode characters,
    // not UTF-8 bytes, and the slice is exactly the recorded quote.
    let chunk = &chunks[0];
    let start = byte_offset(&chunk.raw_text, span.start_char).expect("start offset");
    let end = byte_offset(&chunk.raw_text, span.end_char).expect("end offset");
    assert_eq!(&chunk.raw_text[start..end], span.quote);
}

#[test]
fn seeded_paraphrase_is_rejected() {
    let (chunks, mut span) = fixture();
    span.quote = "the result probably holds".to_string();
    let error = validate_evidence(&span, &chunks).expect_err("paraphrase must fail");
    assert!(error.to_string().contains("exact substring"), "{error:#}");
}

#[test]
fn seeded_wrong_character_offset_is_rejected() {
    let (chunks, mut span) = fixture();
    span.start_char += 1;
    let error = validate_evidence(&span, &chunks).expect_err("wrong offset must fail");
    assert!(error.to_string().contains("exact substring"), "{error:#}");
}

#[test]
fn seeded_unresolvable_page_is_rejected() {
    let (chunks, mut span) = fixture();
    span.page = 99;
    let error = validate_evidence(&span, &chunks).expect_err("wrong page must fail");
    assert!(error.to_string().contains("does not resolve"), "{error:#}");
}

#[test]
fn locator_lookup_is_unique_for_attachment_chunk_and_page() {
    let (chunks, span) = fixture();
    let index: BTreeMap<(i64, &str, u32), &Chunk> = chunks
        .iter()
        .map(|chunk| {
            (
                (
                    chunk.attachment_item_id,
                    chunk.chunk_id.as_str(),
                    chunk.page,
                ),
                chunk,
            )
        })
        .collect();
    let key = (span.attachment_item_id, span.chunk_id.as_str(), span.page);
    assert!(index.contains_key(&key));
}
