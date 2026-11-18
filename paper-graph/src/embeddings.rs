//! OpenAI-compatible embeddings served by the user's llama.cpp deployment.

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use std::time::Duration;

pub const DEFAULT_BASE_URL: &str = "https://llama.gw.emliunix.cc/v1";
pub const FALLBACK_BASE_URL: &str = "http://192.168.50.18:8080/v1";
pub const DEFAULT_MODEL: &str = "bge-m3";
pub const EXPECTED_DIMENSIONS: usize = 1024;

#[derive(Debug, Clone)]
pub struct EmbeddingClient {
    pub base_url: String,
    pub model: String,
    pub timeout: Duration,
}

impl Default for EmbeddingClient {
    fn default() -> Self {
        Self {
            base_url: std::env::var("PAPER_GRAPH_EMBEDDING_URL")
                .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string()),
            model: DEFAULT_MODEL.to_string(),
            timeout: Duration::from_secs(120),
        }
    }
}

impl EmbeddingClient {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            timeout: Duration::from_secs(120),
        }
    }

    /// Return embeddings in input order and the server's token count.
    pub async fn embed(&self, texts: &[String], batch_size: usize) -> Result<(Vec<Vec<f32>>, u64)> {
        if batch_size < 1 {
            bail!("batch_size must be positive");
        }
        let http = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .context("build embedding HTTP client")?;
        let mut vectors = Vec::with_capacity(texts.len());
        let mut total_tokens = 0_u64;

        for (start, batch) in texts.chunks(batch_size).enumerate() {
            let start = start * batch_size;
            let lengths: Vec<usize> = batch.iter().map(|text| text.chars().count()).collect();
            let payload = json!({ "model": self.model, "input": batch });
            let response = self
                .post_embeddings(&http, &payload)
                .await
                .map_err(|error| {
                    anyhow!(
                        "Embedding batch starting at input {start} failed \
                         (batch_size={}, character_lengths={lengths:?}): {error}",
                        batch.len()
                    )
                })?;

            let data = response
                .get("data")
                .and_then(Value::as_array)
                .context("embedding response has no data array")?;
            let mut rows: Vec<&Value> = data.iter().collect();
            rows.sort_by_key(|row| row.get("index").and_then(Value::as_i64).unwrap_or(0));

            if rows.len() != batch.len() {
                bail!(
                    "Embedding server returned {} rows for {} inputs",
                    rows.len(),
                    batch.len()
                );
            }

            for row in rows {
                let embedding = row
                    .get("embedding")
                    .and_then(Value::as_array)
                    .context("embedding row has no embedding array")?;
                if embedding.len() != EXPECTED_DIMENSIONS {
                    bail!(
                        "Expected {EXPECTED_DIMENSIONS} dimensions, got {}",
                        embedding.len()
                    );
                }
                let mut vector = Vec::with_capacity(EXPECTED_DIMENSIONS);
                for coordinate in embedding {
                    let coordinate = coordinate
                        .as_f64()
                        .context("embedding coordinate is not a number")?;
                    if !coordinate.is_finite() {
                        bail!("Embedding server returned a non-finite coordinate");
                    }
                    let coordinate = coordinate as f32;
                    if !coordinate.is_finite() {
                        bail!("Embedding server returned a coordinate outside f32 range");
                    }
                    vector.push(coordinate);
                }
                vectors.push(vector);
            }

            total_tokens += response
                .pointer("/usage/total_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0);
        }

        Ok((vectors, total_tokens))
    }

    async fn post_embeddings(&self, http: &reqwest::Client, payload: &Value) -> Result<Value> {
        let url = format!("{}/embeddings", self.base_url.trim_end_matches('/'));
        let mut last_error: Option<anyhow::Error> = None;
        for attempt in 0..3 {
            match http.post(&url).json(payload).send().await {
                Ok(response) if response.status().is_success() => {
                    return response
                        .json()
                        .await
                        .context("decode embedding response as JSON");
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let body: String = body.chars().take(4000).collect();
                    last_error = Some(anyhow!("HTTP {}: {body}", status.as_u16()));
                }
                Err(error) => {
                    last_error = Some(anyhow!(error));
                }
            }
            if attempt < 2 {
                tokio::time::sleep(Duration::from_secs_f64(0.5 * 2_f64.powi(attempt))).await;
            }
        }
        Err(anyhow!(
            "Embedding request failed at {}: {}",
            self.base_url,
            last_error
                .map(|error| error.to_string())
                .unwrap_or_default()
        ))
    }
}
