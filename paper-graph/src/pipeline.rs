//! Manual synchronization shared by the CLI and local UI.

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::embeddings::EmbeddingClient;
use crate::store::{open_database, rebuild, stats, Stats};
use crate::zotero::{read_library, Library};

#[derive(Debug, Clone)]
pub struct PreparedSync {
    pub library: Library,
    pub embeddings: Vec<Vec<f32>>,
    pub tokens: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub database: String,
    pub embedding_model: String,
    pub embedding_url: String,
    pub embedding_tokens: u64,
    pub elapsed_seconds: f64,
    #[serde(flatten)]
    pub stats: Stats,
}

impl SyncReport {
    pub fn new(
        database: &Path,
        client: &EmbeddingClient,
        tokens: u64,
        started: Instant,
        stats: Stats,
    ) -> Self {
        Self {
            database: database.display().to_string(),
            embedding_model: client.model.clone(),
            embedding_url: client.base_url.clone(),
            embedding_tokens: tokens,
            elapsed_seconds: round_seconds(started.elapsed()),
            stats,
        }
    }
}

/// Read Zotero and embed every work without touching HawDB.
///
/// The UI calls this before taking its database lock, so a slow embedding
/// server does not block searches.
pub async fn prepare_sync(
    zotero_path: &Path,
    client: &EmbeddingClient,
    batch_size: usize,
    limit: Option<usize>,
) -> Result<PreparedSync> {
    let path = zotero_path.to_path_buf();
    let mut library = tokio::task::spawn_blocking(move || read_library(&path))
        .await
        .context("Zotero reader task panicked")??;
    if let Some(limit) = limit {
        library.truncate(limit);
    }
    let texts: Vec<String> = library
        .works
        .iter()
        .map(|work| work.search_text())
        .collect();
    let (embeddings, tokens) = client.embed(&texts, batch_size).await?;
    Ok(PreparedSync {
        library,
        embeddings,
        tokens,
    })
}

pub async fn sync_library(
    db_path: &Path,
    zotero_path: &Path,
    embedding_url: &str,
    model: &str,
    batch_size: usize,
    limit: Option<usize>,
) -> Result<SyncReport> {
    let started = Instant::now();
    let client = EmbeddingClient::new(embedding_url, model);
    let prepared = prepare_sync(zotero_path, &client, batch_size, limit).await?;

    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create database directory {}", parent.display()))?;
        }
    }

    let mut db = open_database(Some(db_path))?;
    rebuild(&mut db, &prepared.library, &prepared.embeddings)?;
    let stats = stats(&mut db)?;
    drop(db);
    Ok(SyncReport::new(
        db_path,
        &client,
        prepared.tokens,
        started,
        stats,
    ))
}

/// Check that Zotero can be read and that the embedding server answers.
pub async fn probe(zotero_path: &Path, embedding_url: &str, model: &str) -> Result<Value> {
    let path = zotero_path.to_path_buf();
    let library = tokio::task::spawn_blocking(move || read_library(&path))
        .await
        .context("Zotero reader task panicked")??;
    let client = EmbeddingClient::new(embedding_url, model);
    let texts = vec!["Paper graph embedding probe".to_string()];
    let (vectors, tokens) = client.embed(&texts, 1).await?;
    let dimensions = vectors.first().map(Vec::len).unwrap_or(0);
    Ok(json!({
        "zotero": zotero_path.display().to_string(),
        "works": library.works.len(),
        "attachments": library.attachments.len(),
        "embedding_model": model,
        "embedding_url": embedding_url,
        "embedding_dimensions": dimensions,
        "embedding_tokens": tokens,
    }))
}

fn round_seconds(duration: Duration) -> f64 {
    (duration.as_secs_f64() * 1000.0).round() / 1000.0
}

use crate::chunk::{
    chunk_pages, write_chunks_jsonl, ChunkRunReport, DEFAULT_MAX_CHUNK_CHARS,
};
use crate::parse::{
    admit_corpus, detect_system_pdf_tool, load_corpus_manifest, parse_admitted_corpus,
    read_admission_report, read_pages_jsonl, write_admission_report, write_pages_jsonl,
    zotero_storage_root, AdmissionReport, ParseRunReport,
};

#[derive(Debug, Clone, Serialize)]
pub struct Stage1To3RunSummary {
    pub out_dir: String,
    pub admission_file: String,
    pub pages_file: String,
    pub chunks_file: String,
    pub parse_report_file: String,
    pub chunk_report_file: String,
    pub admitted_papers: usize,
    pub admission_failures: usize,
    pub parsed_papers: usize,
    pub parse_failures: usize,
    pub total_pages: usize,
    pub chunked_papers: usize,
    pub chunk_failures: usize,
    pub total_chunks: usize,
    pub parser_version: String,
}

/// Run typed Stages 1–3 end-to-end and write validated artifacts to `out_dir`.
pub fn run_stages_1_to_3(
    corpus_path: &Path,
    zotero_path: &Path,
    out_dir: &Path,
    parse_split: Option<&str>,
    max_chunk_chars: Option<usize>,
) -> Result<Stage1To3RunSummary> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("create output directory {}", out_dir.display()))?;

    let corpus = load_corpus_manifest(corpus_path)?;
    let library = read_library(zotero_path)?;
    let storage_root = zotero_storage_root(zotero_path);

    // Stage 1: Admission
    let admission: AdmissionReport = admit_corpus(&corpus, &library, &storage_root)?;
    let admission_path = out_dir.join("admission.json");
    write_admission_report(&admission_path, &admission)?;
    let verified_admission = read_admission_report(&admission_path)?;

    // Stage 2: Parse
    let tool = detect_system_pdf_tool()?;
    let (pages, parse_report): (_, ParseRunReport) =
        parse_admitted_corpus(&verified_admission, parse_split, &tool)?;
    let pages_path = out_dir.join("pages.jsonl");
    write_pages_jsonl(&pages_path, &pages)?;
    let parse_report_path = out_dir.join("parse-report.json");
    std::fs::write(
        &parse_report_path,
        format!("{}\n", serde_json::to_string_pretty(&parse_report)?),
    )?;
    let verified_pages = read_pages_jsonl(&pages_path)?;

    // Stage 3: Chunk
    let (chunks, chunk_report): (_, ChunkRunReport) = chunk_pages(
        &verified_pages,
        max_chunk_chars.unwrap_or(DEFAULT_MAX_CHUNK_CHARS),
    )?;
    let chunks_path = out_dir.join("chunks.jsonl");
    write_chunks_jsonl(&chunks_path, &chunks)?;
    let chunk_report_path = out_dir.join("chunk-report.json");
    std::fs::write(
        &chunk_report_path,
        format!("{}\n", serde_json::to_string_pretty(&chunk_report)?),
    )?;

    Ok(Stage1To3RunSummary {
        out_dir: out_dir.display().to_string(),
        admission_file: admission_path.display().to_string(),
        pages_file: pages_path.display().to_string(),
        chunks_file: chunks_path.display().to_string(),
        parse_report_file: parse_report_path.display().to_string(),
        chunk_report_file: chunk_report_path.display().to_string(),
        admitted_papers: verified_admission.admitted_count,
        admission_failures: verified_admission.failed_count,
        parsed_papers: parse_report.succeeded_papers,
        parse_failures: parse_report.failed_papers,
        total_pages: verified_pages.len(),
        chunked_papers: chunk_report.succeeded_papers,
        chunk_failures: chunk_report.failed_papers,
        total_chunks: chunks.len(),
        parser_version: tool.parser_version,
    })
}

