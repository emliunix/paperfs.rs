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
