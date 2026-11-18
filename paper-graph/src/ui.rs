//! Loopback-only search and synchronization UI built on axum.

use anyhow::{Context, Result};
use axum::{
    extract::{rejection::QueryRejection, Query, State},
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        HeaderValue, StatusCode,
    },
    middleware,
    response::{Html, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{SecondsFormat, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use crate::embeddings::EmbeddingClient;
use crate::pipeline::{prepare_sync, SyncReport};
use crate::store::SearchRow;
use crate::store::{
    bm25_search, hybrid_search, open_database, rebuild, stats, vector_search, work_detail, Stats,
    WorkDetail,
};

const PAGE: &str = include_str!("ui.html");

struct AppState {
    db: Mutex<hawdb::Database>,
    db_path: PathBuf,
    zotero_path: PathBuf,
    embedding_url: String,
    model: String,
}

type ApiError = (StatusCode, Json<Value>);

fn api_error(error: impl std::fmt::Display) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": error.to_string() })),
    )
}

fn lock_db(state: &AppState) -> MutexGuard<'_, hawdb::Database> {
    state
        .db
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub async fn serve(
    db_path: PathBuf,
    zotero_path: PathBuf,
    embedding_url: String,
    model: String,
    host: String,
    port: u16,
) -> Result<()> {
    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create database directory {}", parent.display()))?;
        }
    }
    let db = open_database(Some(&db_path))?;
    let state = Arc::new(AppState {
        db: Mutex::new(db),
        db_path,
        zotero_path,
        embedding_url,
        model,
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/stats", get(api_stats))
        .route("/api/search", get(api_search))
        .route("/api/work", get(api_work))
        .route("/api/history", get(api_history))
        .route("/api/sync", post(api_sync))
        .fallback(not_found)
        .layer(middleware::map_response(cache_headers))
        .with_state(state);

    let address = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .with_context(|| format!("bind UI server to {address}"))?;
    println!("Paper graph UI: http://{address}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serve paper graph UI")?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn cache_headers(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    let json_content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.starts_with("application/json"))
        .unwrap_or(false);
    if json_content_type {
        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
    }
    response
}

async fn not_found() -> (StatusCode, Json<Value>) {
    (StatusCode::NOT_FOUND, Json(json!({ "error": "Not found" })))
}

async fn index() -> Html<&'static str> {
    Html(PAGE)
}

async fn api_stats(State(state): State<Arc<AppState>>) -> Result<Json<Stats>, ApiError> {
    let mut db = lock_db(&state);
    stats(&mut db).map(Json).map_err(api_error)
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    q: String,
    mode: Option<String>,
    top_k: Option<i64>,
    bm25_weight: Option<f64>,
    vector_weight: Option<f64>,
}

async fn api_search(
    State(state): State<Arc<AppState>>,
    query: Result<Query<SearchParams>, QueryRejection>,
) -> Result<Json<Vec<SearchRow>>, ApiError> {
    let Query(params) = query.map_err(|rejection| api_error(rejection.body_text()))?;
    let mode = params.mode.as_deref().unwrap_or("hybrid");
    let top_k = params.top_k.unwrap_or(10).clamp(1, 50) as usize;
    let bm25_weight = params.bm25_weight.unwrap_or(1.0);
    let vector_weight = params.vector_weight.unwrap_or(1.0);

    let rows = match mode {
        "bm25" => {
            let mut db = lock_db(&state);
            bm25_search(&mut db, &params.q, top_k).map_err(api_error)?
        }
        "vector" | "hybrid" => {
            let client = EmbeddingClient::new(&state.embedding_url, &state.model);
            let (vectors, _) = client
                .embed(std::slice::from_ref(&params.q), 1)
                .await
                .map_err(api_error)?;
            let vector = vectors
                .first()
                .ok_or_else(|| api_error("Embedding server returned no vector"))?;
            let mut db = lock_db(&state);
            if mode == "vector" {
                vector_search(&mut db, vector, top_k).map_err(api_error)?
            } else {
                hybrid_search(
                    &mut db,
                    &params.q,
                    vector,
                    top_k,
                    bm25_weight,
                    vector_weight,
                )
                .map_err(api_error)?
            }
        }
        other => return Err(api_error(format!("Unknown search mode: {other}"))),
    };

    append_history(&state.db_path, &params.q, mode, top_k, &rows).map_err(api_error)?;
    Ok(Json(rows))
}

#[derive(Debug, Deserialize)]
struct WorkParams {
    id: i64,
}

async fn api_work(
    State(state): State<Arc<AppState>>,
    query: Result<Query<WorkParams>, QueryRejection>,
) -> Result<Json<WorkDetail>, ApiError> {
    let Query(params) = query.map_err(|rejection| api_error(rejection.body_text()))?;
    let mut db = lock_db(&state);
    match work_detail(&mut db, params.id).map_err(api_error)? {
        Some(work) => Ok(Json(work)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Work not found" })),
        )),
    }
}

async fn api_history(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Value>>, ApiError> {
    let path = history_path(&state.db_path);
    if !path.exists() {
        return Ok(Json(Vec::new()));
    }
    let text = tokio::fs::read_to_string(&path).await.map_err(api_error)?;
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(20);
    let mut rows = Vec::with_capacity(lines.len() - start);
    for line in &lines[start..] {
        rows.push(serde_json::from_str(line).map_err(api_error)?);
    }
    Ok(Json(rows))
}

async fn api_sync(State(state): State<Arc<AppState>>) -> Result<Json<SyncReport>, ApiError> {
    let started = Instant::now();
    let client = EmbeddingClient::new(&state.embedding_url, &state.model);
    let prepared = prepare_sync(&state.zotero_path, &client, 16, None)
        .await
        .map_err(api_error)?;
    let stats = {
        let mut db = lock_db(&state);
        rebuild(&mut db, &prepared.library, &prepared.embeddings).map_err(api_error)?;
        stats(&mut db).map_err(api_error)?
    };
    Ok(Json(SyncReport::new(
        &state.db_path,
        &client,
        prepared.tokens,
        started,
        stats,
    )))
}

fn history_path(db_path: &Path) -> PathBuf {
    let parent = match db_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    parent.join("search-history.jsonl")
}

fn append_history(
    db_path: &Path,
    query: &str,
    mode: &str,
    top_k: usize,
    rows: &[SearchRow],
) -> Result<()> {
    let path = history_path(db_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create search history directory {}", parent.display()))?;
    }
    let mut history = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open search history {}", path.display()))?;
    let entry = json!({
        "ts": Utc::now().to_rfc3339_opts(SecondsFormat::Micros, false),
        "query": query,
        "mode": mode,
        "top_k": top_k,
        "item_ids": rows.iter().map(|row| row.item_id).collect::<Vec<_>>(),
    });
    writeln!(history, "{entry}")?;
    Ok(())
}
