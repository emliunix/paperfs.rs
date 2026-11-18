//! Command-line interface for the local paper knowledge graph.

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::path::PathBuf;

use crate::embeddings::{EmbeddingClient, DEFAULT_BASE_URL, DEFAULT_MODEL};
use crate::pipeline::{probe, sync_library};
use crate::store::{author_works, bm25_search, hybrid_search, open_database, stats, vector_search};
use crate::ui;
use crate::zotero::default_zotero_db;

#[derive(Debug, Parser)]
#[command(
    name = "paper-graph",
    version,
    about = "Search a Zotero library with HawDB, BM25, and local bge-m3 embeddings"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Fully rebuild from Zotero
    Sync(SyncArgs),
    /// Check Zotero and embeddings
    Probe(SharedArgs),
    /// Search indexed works
    Search(SearchArgs),
    /// Show graph counts
    Stats(SharedArgs),
    /// Find works by author name
    Author(AuthorArgs),
    /// Start the local experiment UI
    Ui(UiArgs),
}

#[derive(Debug, Clone, Args)]
struct SharedArgs {
    #[arg(long, default_value_os_t = default_db_path())]
    db: PathBuf,
    #[arg(long, default_value_os_t = default_zotero_db())]
    zotero: PathBuf,
    #[arg(long, default_value_t = default_embedding_url())]
    embedding_url: String,
    #[arg(long, default_value = DEFAULT_MODEL)]
    model: String,
}

#[derive(Debug, Args)]
struct SyncArgs {
    #[command(flatten)]
    shared: SharedArgs,
    #[arg(long, default_value_t = 16)]
    batch_size: usize,
    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SearchMode {
    #[value(name = "bm25")]
    Bm25,
    #[value(name = "vector")]
    Vector,
    #[value(name = "hybrid")]
    Hybrid,
}

#[derive(Debug, Args)]
struct SearchArgs {
    #[command(flatten)]
    shared: SharedArgs,
    query: String,
    #[arg(long, value_enum, default_value = "hybrid")]
    mode: SearchMode,
    #[arg(long, default_value_t = 10)]
    top_k: usize,
    #[arg(long, default_value_t = 1.0)]
    bm25_weight: f64,
    #[arg(long, default_value_t = 1.0)]
    vector_weight: f64,
}

#[derive(Debug, Args)]
struct AuthorArgs {
    #[command(flatten)]
    shared: SharedArgs,
    name: String,
}

#[derive(Debug, Args)]
struct UiArgs {
    #[command(flatten)]
    shared: SharedArgs,
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value_t = 8765)]
    port: u16,
}

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt().with_target(false).init();
    let cli = Cli::parse();
    match cli.command {
        Command::Sync(args) => {
            let report = sync_library(
                &args.shared.db,
                &args.shared.zotero,
                &args.shared.embedding_url,
                &args.shared.model,
                args.batch_size,
                args.limit,
            )
            .await?;
            print_json(&report)?;
        }
        Command::Probe(args) => {
            let value = probe(&args.zotero, &args.embedding_url, &args.model).await?;
            print_json(&value)?;
        }
        Command::Search(args) => {
            let client = EmbeddingClient::new(&args.shared.embedding_url, &args.shared.model);
            let query_embedding = match args.mode {
                SearchMode::Bm25 => None,
                SearchMode::Vector | SearchMode::Hybrid => {
                    let (vectors, _) = client.embed(std::slice::from_ref(&args.query), 1).await?;
                    Some(
                        vectors.into_iter().next().ok_or_else(|| {
                            anyhow::anyhow!("Embedding server returned no vector")
                        })?,
                    )
                }
            };
            let mut db = open_database(Some(&args.shared.db))?;
            let rows = match args.mode {
                SearchMode::Bm25 => bm25_search(&mut db, &args.query, args.top_k)?,
                SearchMode::Vector => vector_search(
                    &mut db,
                    query_embedding
                        .as_deref()
                        .ok_or_else(|| anyhow::anyhow!("missing query embedding"))?,
                    args.top_k,
                )?,
                SearchMode::Hybrid => hybrid_search(
                    &mut db,
                    &args.query,
                    query_embedding
                        .as_deref()
                        .ok_or_else(|| anyhow::anyhow!("missing query embedding"))?,
                    args.top_k,
                    args.bm25_weight,
                    args.vector_weight,
                )?,
            };
            drop(db);
            print_json(&rows)?;
        }
        Command::Stats(args) => {
            let mut db = open_database(Some(&args.db))?;
            let value = stats(&mut db)?;
            drop(db);
            print_json(&value)?;
        }
        Command::Author(args) => {
            let mut db = open_database(Some(&args.shared.db))?;
            let rows = author_works(&mut db, &args.name)?;
            drop(db);
            print_json(&rows)?;
        }
        Command::Ui(args) => {
            ui::serve(
                args.shared.db,
                args.shared.zotero,
                args.shared.embedding_url,
                args.shared.model,
                args.host,
                args.port,
            )
            .await?;
        }
    }
    Ok(())
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn default_db_path() -> PathBuf {
    if let Some(path) = std::env::var_os("PAPER_GRAPH_DB") {
        return PathBuf::from(path);
    }
    let local = std::env::current_dir()
        .unwrap_or_default()
        .join("data")
        .join("library.hawdb");
    if local.is_dir() {
        return local;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("library.hawdb")
}

fn default_embedding_url() -> String {
    std::env::var("PAPER_GRAPH_EMBEDDING_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}
