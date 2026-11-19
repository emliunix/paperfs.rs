//! Stage 3 text normalization and section-aware chunking with serde contract validation.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::parse::{is_valid_sha256_hex, PageParseStatus, PageRecord};

pub const CHUNK_SCHEMA_VERSION: &str = "stage-3-chunks-v1";
pub const CHUNKER_VERSION: &str = "section-window-v1";
pub const DEFAULT_MAX_CHUNK_CHARS: usize = 2400;

/// Contract record emitted to `chunks.jsonl`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChunkRecord {
    pub schema_version: String,
    pub chunk_id: String,
    pub paper_id: i64,
    pub attachment_id: i64,
    pub parser_version: String,
    pub chunker_version: String,
    pub page_range: [usize; 2],
    pub section_path: Vec<String>,
    pub ordinal: usize,
    pub content_hash: String,
    pub raw_text: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

impl ChunkRecord {
    pub fn compute_content_hash(raw_text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw_text.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn format_chunk_id(
        attachment_id: i64,
        parser_version: &str,
        ordinal: usize,
        content_hash: &str,
    ) -> String {
        let prefix = content_hash.get(..12).unwrap_or(content_hash);
        format!("att{attachment_id}:{parser_version}:ord{ordinal:04}:{prefix}")
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != CHUNK_SCHEMA_VERSION {
            bail!(
                "unexpected chunk schema_version {}, expected {}",
                self.schema_version,
                CHUNK_SCHEMA_VERSION
            );
        }
        if self.attachment_id <= 0 {
            bail!("invalid attachment_id {} in chunk {}", self.attachment_id, self.chunk_id);
        }
        if self.paper_id <= 0 {
            bail!("invalid paper_id {} in chunk {}", self.paper_id, self.chunk_id);
        }
        if self.parser_version.trim().is_empty() {
            bail!("empty parser_version in chunk {}", self.chunk_id);
        }
        if self.chunker_version.trim().is_empty() {
            bail!("empty chunker_version in chunk {}", self.chunk_id);
        }
        if self.page_range[0] == 0 || self.page_range[0] > self.page_range[1] {
            bail!(
                "invalid page_range {:?} in chunk {}",
                self.page_range,
                self.chunk_id
            );
        }
        if self.section_path.is_empty()
            || self.section_path.iter().any(|segment| segment.trim().is_empty())
        {
            bail!(
                "invalid section_path {:?} in chunk {}",
                self.section_path,
                self.chunk_id
            );
        }
        if self.raw_text.trim().is_empty() {
            bail!("empty raw_text in chunk {}", self.chunk_id);
        }
        if !is_valid_sha256_hex(&self.content_hash) {
            bail!("invalid content_hash {} in chunk {}", self.content_hash, self.chunk_id);
        }
        let expected_hash = Self::compute_content_hash(&self.raw_text);
        if self.content_hash != expected_hash {
            bail!(
                "content_hash mismatch in chunk {}: got {}, expected {}",
                self.chunk_id,
                self.content_hash,
                expected_hash
            );
        }
        let char_len = self.raw_text.chars().count();
        if self.end_offset <= self.start_offset {
            bail!(
                "end_offset {} <= start_offset {} in chunk {}",
                self.end_offset,
                self.start_offset,
                self.chunk_id
            );
        }
        if self.end_offset - self.start_offset != char_len {
            bail!(
                "offset span {}..{} (len {}) != raw_text char count {} in chunk {}",
                self.start_offset,
                self.end_offset,
                self.end_offset - self.start_offset,
                char_len,
                self.chunk_id
            );
        }
        let expected_id = Self::format_chunk_id(
            self.attachment_id,
            &self.parser_version,
            self.ordinal,
            &self.content_hash,
        );
        if self.chunk_id != expected_id {
            bail!(
                "chunk_id mismatch: got {}, expected {}",
                self.chunk_id,
                expected_id
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PaperChunkSummary {
    pub paper_id: i64,
    pub attachment_id: i64,
    pub parser_version: String,
    pub chunked_ok: bool,
    pub chunk_count: usize,
    pub document_chars: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChunkRunReport {
    pub schema_version: String,
    pub chunker_version: String,
    pub max_chunk_chars: usize,
    pub total_papers: usize,
    pub succeeded_papers: usize,
    pub failed_papers: usize,
    pub total_chunks: usize,
    pub papers: Vec<PaperChunkSummary>,
}

#[derive(Debug, Clone)]
struct LineSpan {
    page: usize,
    char_start: usize,
    char_end: usize,
    byte_start: usize,
    byte_end: usize,
    heading: Option<(usize, String)>,
}

fn parse_heading_line(trimmed: &str, seen_intro_or_abstract: bool, page: usize) -> Option<(usize, String)> {
    if trimmed.is_empty() || trimmed.chars().count() > 80 {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    for named in [
        "abstract",
        "acknowledgments",
        "acknowledgements",
        "references",
        "bibliography",
        "conclusion",
        "conclusions",
    ] {
        if lower == named {
            return Some((1, trimmed.to_string()));
        }
    }
    if lower == "appendix" || lower.starts_with("appendix ") {
        if !trimmed.ends_with('.') && trimmed.split_whitespace().count() <= 10 {
            return Some((1, trimmed.to_string()));
        }
    }

    // Numbered section heading: e.g., "1 Introduction", "1. Introduction", "2.1 Typing in Detail"
    let mut split_idx = None;
    for (idx, ch) in trimmed.char_indices() {
        if ch.is_whitespace() {
            split_idx = Some(idx);
            break;
        }
    }
    let split_idx = split_idx?;
    let prefix = &trimmed[..split_idx];
    let rest = trimmed[split_idx..].trim();
    if rest.is_empty() {
        return None;
    }

    let num_core = prefix.strip_suffix('.').unwrap_or(prefix);
    let segments: Vec<&str> = num_core.split('.').collect();
    if segments.is_empty() || segments.len() > 4 {
        return None;
    }
    if !segments.iter().all(|s| !s.is_empty() && s.len() <= 2 && s.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    let top_num: usize = segments[0].parse().ok()?;
    if top_num == 0 || top_num > 25 {
        return None;
    }

    let first_rest = rest.chars().next()?;
    if !first_rest.is_ascii_uppercase() {
        return None;
    }
    if rest.ends_with('.') || rest.ends_with(',') || rest.ends_with(';') || rest.ends_with('-') {
        return None;
    }
    if rest.contains('@') || rest.contains("http://") || rest.contains("https://") {
        return None;
    }
    if rest.matches(',').count() >= 2 {
        return None;
    }
    let words: Vec<&str> = rest.split_whitespace().collect();
    if words.len() > 12 {
        return None;
    }

    let is_intro = rest.to_ascii_lowercase().starts_with("introduction");
    if !seen_intro_or_abstract && !is_intro {
        // Avoid affiliation superscripts ("1 Microsoft Research") on page 1 before Abstract/Introduction.
        return None;
    }
    if segments.len() == 1 && !prefix.ends_with('.') && page >= 2 && top_num == page && words.len() <= 5 {
        // Guard against running page headers like "2 Zhixuan Yang and Nicolas Wu" when " and " links two capitalized names
        if rest.contains(" and ") && words.iter().filter(|w| &&w.to_ascii_lowercase() != &"and").all(|w| w.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false)) && !is_intro {
            // Allow genuine headings like "2 Syntax and Semantics" by checking common scholarly nouns vs author headers on even pages:
            let scholarly = [
                "syntax", "semantics", "types", "typing", "system", "systems", "calculus",
                "model", "models", "method", "methods", "evaluation", "experiments",
                "implementation", "discussion", "background", "preliminaries", "overview",
                "related", "proof", "proofs", "theorems", "logic", "effects", "handlers",
                "monads", "recursion", "schemes", "kernel", "elaboration", "interface",
            ];
            let has_scholarly = words
                .iter()
                .any(|w| scholarly.contains(&w.to_ascii_lowercase().as_str()));
            if !has_scholarly {
                return None;
            }
        }
    }

    let depth = segments.len();
    Some((depth, format!("{prefix} {rest}")))
}

/// Reconstruct the canonical joined document text for a paper's parsed pages.
///
/// Pages are ordered by `page` ascending and joined with `"\n\n"`. Every `ChunkRecord`'s
/// `start_offset..end_offset` indexes characters in this exact reconstructed stream.
pub fn reconstruct_document_text(pages: &[&PageRecord]) -> String {
    let mut sorted: Vec<&PageRecord> = pages
        .iter()
        .copied()
        .filter(|p| p.status != PageParseStatus::Failed && !p.text.is_empty())
        .collect();
    sorted.sort_by_key(|p| p.page);
    sorted
        .into_iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn chunk_single_paper(
    paper_id: i64,
    attachment_id: i64,
    parser_version: &str,
    pages: &[&PageRecord],
    max_chunk_chars: usize,
) -> Result<(Vec<ChunkRecord>, usize)> {
    let max_chars = max_chunk_chars.max(200);
    let mut ordered_pages: Vec<&PageRecord> = pages.to_vec();
    ordered_pages.sort_by_key(|p| p.page);

    for p in &ordered_pages {
        p.validate()?;
        if p.status == PageParseStatus::Failed {
            bail!(
                "paper {} has failed page record: {}",
                paper_id,
                p.error.as_deref().unwrap_or("unknown parse error")
            );
        }
    }

    let non_empty_pages: Vec<&PageRecord> = ordered_pages
        .into_iter()
        .filter(|p| !p.text.is_empty())
        .collect();
    if non_empty_pages.is_empty() {
        bail!("paper {} has 0 non-empty pages to chunk", paper_id);
    }

    let mut doc_text = String::new();
    let mut char_cursor = 0usize;
    let mut lines: Vec<LineSpan> = Vec::new();
    let mut seen_intro_or_abstract = false;

    for (p_idx, page_rec) in non_empty_pages.iter().enumerate() {
        if p_idx > 0 {
            doc_text.push_str("\n\n");
            char_cursor += 2;
        }
        let raw_lines: Vec<&str> = page_rec.text.split('\n').collect();
        for (l_idx, line_str) in raw_lines.iter().enumerate() {
            if l_idx > 0 {
                doc_text.push('\n');
                char_cursor += 1;
            }
            let byte_start = doc_text.len();
            let char_start = char_cursor;
            doc_text.push_str(line_str);
            let line_chars = line_str.chars().count();
            char_cursor += line_chars;
            let byte_end = doc_text.len();
            let char_end = char_cursor;

            let trimmed = line_str.trim();
            let heading = parse_heading_line(trimmed, seen_intro_or_abstract, page_rec.page);
            if let Some((_, ref h_title)) = heading {
                let hl = h_title.to_ascii_lowercase();
                if hl == "abstract" || hl.contains("introduction") {
                    seen_intro_or_abstract = true;
                }
            }
            lines.push(LineSpan {
                page: page_rec.page,
                char_start,
                char_end,
                byte_start,
                byte_end,
                heading,
            });
        }
    }

    // Group lines into contiguous section segments
    #[derive(Debug)]
    struct SectionSegment {
        section_path: Vec<String>,
        lines: Vec<usize>,
    }

    let mut stack: Vec<(usize, String)> = Vec::new();
    let mut segments: Vec<SectionSegment> = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        if let Some((depth, ref title)) = line.heading {
            while let Some((top_depth, _)) = stack.last() {
                if *top_depth >= depth {
                    stack.pop();
                } else {
                    break;
                }
            }
            stack.push((depth, title.clone()));
            let path: Vec<String> = stack.iter().map(|(_, t)| t.clone()).collect();
            segments.push(SectionSegment {
                section_path: path,
                lines: vec![idx],
            });
        } else if let Some(current) = segments.last_mut() {
            current.lines.push(idx);
        } else {
            segments.push(SectionSegment {
                section_path: vec!["Front Matter".to_string()],
                lines: vec![idx],
            });
        }
    }

    let mut chunks = Vec::new();
    let mut ordinal = 0usize;

    for seg in segments {
        let mut i = 0usize;
        while i < seg.lines.len() {
            // Skip leading blank lines so chunks do not begin with empty lines
            while i < seg.lines.len() {
                let l = &lines[seg.lines[i]];
                if doc_text[l.byte_start..l.byte_end].trim().is_empty() {
                    i += 1;
                } else {
                    break;
                }
            }
            if i >= seg.lines.len() {
                break;
            }

            let first_line = &lines[seg.lines[i]];
            let first_len = first_line.char_end - first_line.char_start;

            if first_len > max_chars {
                // Split a single very long line cleanly on char boundaries
                let slice_text = &doc_text[first_line.byte_start..first_line.byte_end];
                let chars: Vec<(usize, char)> = slice_text.char_indices().collect();
                let mut c_pos = 0usize;
                while c_pos < chars.len() {
                    let c_end = (c_pos + max_chars).min(chars.len());
                    let b_start = first_line.byte_start + chars[c_pos].0;
                    let b_end = if c_end < chars.len() {
                        first_line.byte_start + chars[c_end].0
                    } else {
                        first_line.byte_end
                    };
                    let sub = &doc_text[b_start..b_end];
                    if !sub.trim().is_empty() {
                        let start_offset = first_line.char_start + c_pos;
                        let end_offset = first_line.char_start + c_end;
                        let content_hash = ChunkRecord::compute_content_hash(sub);
                        let chunk_id = ChunkRecord::format_chunk_id(
                            attachment_id,
                            parser_version,
                            ordinal,
                            &content_hash,
                        );
                        let rec = ChunkRecord {
                            schema_version: CHUNK_SCHEMA_VERSION.to_string(),
                            chunk_id,
                            paper_id,
                            attachment_id,
                            parser_version: parser_version.to_string(),
                            chunker_version: CHUNKER_VERSION.to_string(),
                            page_range: [first_line.page, first_line.page],
                            section_path: seg.section_path.clone(),
                            ordinal,
                            content_hash,
                            raw_text: sub.to_string(),
                            start_offset,
                            end_offset,
                        };
                        rec.validate()?;
                        chunks.push(rec);
                        ordinal += 1;
                    }
                    c_pos = c_end;
                }
                i += 1;
                continue;
            }

            let mut end_i = i;
            let mut min_page = first_line.page;
            let mut max_page = first_line.page;

            while end_i + 1 < seg.lines.len() {
                let candidate = &lines[seg.lines[end_i + 1]];
                let span_chars = candidate.char_end - first_line.char_start;
                if span_chars > max_chars {
                    break;
                }
                end_i += 1;
            }

            // Trim trailing blank lines from this chunk window
            let mut trimmed_end_i = end_i;
            while trimmed_end_i > i {
                let l = &lines[seg.lines[trimmed_end_i]];
                if doc_text[l.byte_start..l.byte_end].trim().is_empty() {
                    trimmed_end_i -= 1;
                } else {
                    break;
                }
            }

            for k in i..=trimmed_end_i {
                let p = lines[seg.lines[k]].page;
                min_page = min_page.min(p);
                max_page = max_page.max(p);
            }

            let last_line = &lines[seg.lines[trimmed_end_i]];
            let raw_slice = &doc_text[first_line.byte_start..last_line.byte_end];
            if !raw_slice.trim().is_empty() {
                let start_offset = first_line.char_start;
                let end_offset = last_line.char_end;
                let content_hash = ChunkRecord::compute_content_hash(raw_slice);
                let chunk_id = ChunkRecord::format_chunk_id(
                    attachment_id,
                    parser_version,
                    ordinal,
                    &content_hash,
                );
                let rec = ChunkRecord {
                    schema_version: CHUNK_SCHEMA_VERSION.to_string(),
                    chunk_id,
                    paper_id,
                    attachment_id,
                    parser_version: parser_version.to_string(),
                    chunker_version: CHUNKER_VERSION.to_string(),
                    page_range: [min_page, max_page],
                    section_path: seg.section_path.clone(),
                    ordinal,
                    content_hash,
                    raw_text: raw_slice.to_string(),
                    start_offset,
                    end_offset,
                };
                rec.validate()?;
                chunks.push(rec);
                ordinal += 1;
            }

            i = end_i + 1;
        }
    }

    // Verify every chunk resolves to the exact character range in doc_text
    let doc_chars: Vec<char> = doc_text.chars().collect();
    for chunk in &chunks {
        let resolved: String = doc_chars[chunk.start_offset..chunk.end_offset]
            .iter()
            .collect();
        if resolved != chunk.raw_text {
            bail!(
                "internal chunk offset bug on {}: slice at {}..{} did not match raw_text",
                chunk.chunk_id,
                chunk.start_offset,
                chunk.end_offset
            );
        }
    }

    Ok((chunks, doc_chars.len()))
}

/// Run Stage 3 chunking over Stage 2 `PageRecord`s.
///
/// Validates all input `PageRecord`s before chunking and validates every emitted
/// `ChunkRecord` via serde roundtrip and contract invariants.
pub fn chunk_pages(
    pages: &[PageRecord],
    max_chunk_chars: usize,
) -> Result<(Vec<ChunkRecord>, ChunkRunReport)> {
    for (idx, p) in pages.iter().enumerate() {
        p.validate()
            .with_context(|| format!("validate input PageRecord at index {idx}"))?;
    }

    let mut by_paper: BTreeMap<i64, Vec<&PageRecord>> = BTreeMap::new();
    for page in pages {
        by_paper.entry(page.paper_id).or_default().push(page);
    }

    let mut all_chunks = Vec::new();
    let mut paper_summaries = Vec::with_capacity(by_paper.len());
    let mut succeeded_papers = 0usize;
    let mut failed_papers = 0usize;

    for (paper_id, paper_pages) in by_paper {
        let first = paper_pages[0];
        let attachment_id = first.attachment_id;
        let parser_version = first.parser_version.clone();

        if let Some(failed) = paper_pages
            .iter()
            .find(|p| p.status == PageParseStatus::Failed)
        {
            failed_papers += 1;
            paper_summaries.push(PaperChunkSummary {
                paper_id,
                attachment_id,
                parser_version,
                chunked_ok: false,
                chunk_count: 0,
                document_chars: 0,
                error: Some(format!(
                    "stage 2 parse failure prevented chunking: {}",
                    failed.error.as_deref().unwrap_or("unknown")
                )),
            });
            continue;
        }

        match chunk_single_paper(
            paper_id,
            attachment_id,
            &parser_version,
            &paper_pages,
            max_chunk_chars,
        ) {
            Ok((chunks, doc_chars)) => {
                succeeded_papers += 1;
                paper_summaries.push(PaperChunkSummary {
                    paper_id,
                    attachment_id,
                    parser_version,
                    chunked_ok: true,
                    chunk_count: chunks.len(),
                    document_chars: doc_chars,
                    error: None,
                });
                all_chunks.extend(chunks);
            }
            Err(err) => {
                failed_papers += 1;
                paper_summaries.push(PaperChunkSummary {
                    paper_id,
                    attachment_id,
                    parser_version,
                    chunked_ok: false,
                    chunk_count: 0,
                    document_chars: 0,
                    error: Some(format!("{err:#}")),
                });
            }
        }
    }

    validate_chunk_stream(&all_chunks)?;

    let report = ChunkRunReport {
        schema_version: CHUNK_SCHEMA_VERSION.to_string(),
        chunker_version: CHUNKER_VERSION.to_string(),
        max_chunk_chars,
        total_papers: paper_summaries.len(),
        succeeded_papers,
        failed_papers,
        total_chunks: all_chunks.len(),
        papers: paper_summaries,
    };

    Ok((all_chunks, report))
}

pub fn validate_chunk_stream(chunks: &[ChunkRecord]) -> Result<()> {
    let mut seen_ids = HashSet::new();
    let mut last_by_attachment: BTreeMap<i64, (usize, usize)> = BTreeMap::new();

    for (idx, chunk) in chunks.iter().enumerate() {
        let json = serde_json::to_string(chunk)
            .with_context(|| format!("serde serialize chunk at index {idx}"))?;
        let decoded: ChunkRecord = serde_json::from_str(&json)
            .with_context(|| format!("serde deserialize chunk at index {idx}"))?;
        decoded
            .validate()
            .with_context(|| format!("validate chunk contract at index {idx}"))?;

        if !seen_ids.insert(decoded.chunk_id.clone()) {
            bail!("duplicate chunk_id {}", decoded.chunk_id);
        }

        if let Some((prev_ord, prev_end)) = last_by_attachment.get(&decoded.attachment_id) {
            if decoded.ordinal != prev_ord + 1 {
                bail!(
                    "non-contiguous ordinal on attachment {}: expected {}, got {}",
                    decoded.attachment_id,
                    prev_ord + 1,
                    decoded.ordinal
                );
            }
            if decoded.start_offset < *prev_end {
                bail!(
                    "overlapping or out-of-order offsets on attachment {}: start_offset {} < previous end_offset {}",
                    decoded.attachment_id,
                    decoded.start_offset,
                    prev_end
                );
            }
        } else if decoded.ordinal != 0 {
            bail!(
                "first chunk on attachment {} must start at ordinal 0, got {}",
                decoded.attachment_id,
                decoded.ordinal
            );
        }

        last_by_attachment.insert(decoded.attachment_id, (decoded.ordinal, decoded.end_offset));
    }
    Ok(())
}

/// Validate every `ChunkRecord` through serde before writing `chunks.jsonl`.
pub fn write_chunks_jsonl(path: &Path, chunks: &[ChunkRecord]) -> Result<()> {
    validate_chunk_stream(chunks)?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create directory {}", parent.display()))?;
        }
    }
    let mut out = String::new();
    for (idx, chunk) in chunks.iter().enumerate() {
        let line = serde_json::to_string(chunk)
            .with_context(|| format!("serialize chunk {idx}"))?;
        let check: ChunkRecord = serde_json::from_str(&line)
            .with_context(|| format!("serde pre-write check failed on chunk {idx}"))?;
        check.validate()?;
        out.push_str(&line);
        out.push('\n');
    }
    fs::write(path, out).with_context(|| format!("write chunks jsonl {}", path.display()))
}

pub fn read_chunks_jsonl(path: &Path) -> Result<Vec<ChunkRecord>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut chunks = Vec::new();
    for (line_idx, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("read {}:{}", path.display(), line_idx + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let rec: ChunkRecord = serde_json::from_str(&line)
            .with_context(|| format!("parse chunk JSON at {}:{}", path.display(), line_idx + 1))?;
        rec.validate().with_context(|| {
            format!("validate chunk contract at {}:{}", path.display(), line_idx + 1)
        })?;
        chunks.push(rec);
    }
    validate_chunk_stream(&chunks)?;
    Ok(chunks)
}
