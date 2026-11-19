use anyhow::{bail, Context, Result};
use paper_graph::embeddings::{EmbeddingClient, EXPECTED_DIMENSIONS};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

async fn read_request(stream: &mut tokio::net::TcpStream) -> Result<(String, Value)> {
    let mut bytes = Vec::new();
    let header_end = loop {
        if let Some(position) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break position;
        }
        let mut buffer = [0_u8; 4096];
        let read = stream
            .read(&mut buffer)
            .await
            .context("read request headers")?;
        if read == 0 {
            bail!("connection closed before request headers");
        }
        bytes.extend_from_slice(&buffer[..read]);
    };
    let headers = std::str::from_utf8(&bytes[..header_end]).context("request headers are UTF-8")?;
    let request_line = headers.lines().next().context("request line")?.to_string();
    let content_length = headers
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .context("JSON request has no valid content-length")?;
    let body_start = header_end + 4;
    while bytes.len() < body_start + content_length {
        let mut buffer = [0_u8; 4096];
        let read = stream
            .read(&mut buffer)
            .await
            .context("read request body")?;
        if read == 0 {
            bail!("connection closed before complete request body");
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    let body = serde_json::from_slice(&bytes[body_start..body_start + content_length])
        .context("decode request JSON")?;
    Ok((request_line, body))
}

async fn spawn_embedding_server(
    dimensions: usize,
    request_count: usize,
) -> Result<(
    String,
    mpsc::UnboundedReceiver<Value>,
    JoinHandle<Result<()>>,
)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let (sender, receiver) = mpsc::unbounded_channel();
    let task = tokio::spawn(async move {
        for _ in 0..request_count {
            let (mut stream, _) = listener
                .accept()
                .await
                .context("accept embedding request")?;
            let (request_line, body) = read_request(&mut stream).await?;
            if request_line != "POST /v1/embeddings HTTP/1.1" {
                bail!("unexpected embedding request: {request_line}");
            }
            let inputs = body
                .get("input")
                .and_then(Value::as_array)
                .context("embedding request has no input array")?;
            let mut rows = Vec::with_capacity(inputs.len());
            for (index, input) in inputs.iter().enumerate() {
                let marker = input
                    .as_str()
                    .context("fixture input is not a string")?
                    .parse::<f64>()
                    .context("fixture input is not numeric")?;
                rows.push(json!({
                    "index": index,
                    "embedding": vec![marker; dimensions],
                }));
            }
            // Deliberately reverse rows: the client must restore input order by index.
            rows.reverse();
            let response = json!({"data": rows, "usage": {"total_tokens": 7}}).to_string();
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                response.len()
            );
            stream.write_all(headers.as_bytes()).await?;
            stream.write_all(response.as_bytes()).await?;
            stream.shutdown().await?;
            sender.send(body).context("record embedding request")?;
        }
        Ok(())
    });
    Ok((format!("http://{address}/v1"), receiver, task))
}

#[tokio::test]
async fn embeds_1024_dimensions_in_batches_and_input_order() {
    let (base_url, mut requests, server) = spawn_embedding_server(EXPECTED_DIMENSIONS, 3)
        .await
        .expect("start local embedding fixture");
    let client = EmbeddingClient::new(base_url, "fixture-model");
    let texts: Vec<String> = (0..5).map(|value| value.to_string()).collect();

    let (vectors, tokens) = client.embed(&texts, 2).await.expect("embed fixture inputs");
    server
        .await
        .expect("join embedding fixture")
        .expect("serve all batches");

    assert_eq!(tokens, 21);
    assert_eq!(vectors.len(), texts.len());
    for (index, vector) in vectors.iter().enumerate() {
        assert_eq!(vector.len(), EXPECTED_DIMENSIONS);
        assert!(vector.iter().all(|coordinate| *coordinate == index as f32));
    }
    let batches: Vec<Value> = (0..3)
        .map(|_| requests.try_recv().expect("recorded request"))
        .collect();
    let sizes: Vec<usize> = batches
        .iter()
        .map(|body| body["input"].as_array().expect("input array").len())
        .collect();
    assert_eq!(sizes, vec![2, 2, 1]);
    assert!(batches.iter().all(|body| body["model"] == "fixture-model"));
    assert!(requests.try_recv().is_err(), "unexpected extra batch");
}

#[tokio::test]
async fn zero_batch_size_is_rejected_before_any_request() {
    let client = EmbeddingClient::new("http://127.0.0.1:9/v1", "fixture-model");
    let error = client
        .embed(&["input".to_string()], 0)
        .await
        .expect_err("zero batch size must fail");
    assert!(
        error.to_string().contains("batch_size must be positive"),
        "{error:#}"
    );
}

#[tokio::test]
async fn seeded_short_embedding_is_rejected_instead_of_padded() {
    let (base_url, _requests, server) = spawn_embedding_server(EXPECTED_DIMENSIONS - 1, 1)
        .await
        .expect("start defective embedding fixture");
    let client = EmbeddingClient::new(base_url, "fixture-model");

    let error = client
        .embed(&["0".to_string()], 1)
        .await
        .expect_err("seeded 1023-dimensional response must fail");
    server
        .await
        .expect("join defective fixture")
        .expect("serve response");
    assert!(
        error
            .to_string()
            .contains("Expected 1024 dimensions, got 1023"),
        "{error:#}"
    );
}

#[tokio::test]
async fn unreachable_router_fails_explicitly_never_returns_zero_vectors() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve unused port");
    let address = listener.local_addr().expect("unused port address");
    drop(listener);
    let mut client = EmbeddingClient::new(format!("http://{address}/v1"), "fixture-model");
    client.timeout = std::time::Duration::from_millis(100);

    let error = client
        .embed(&["input".to_string()], 1)
        .await
        .expect_err("unreachable router must return an error, not a zero vector");
    let message = format!("{error:#}");
    assert!(message.contains("Embedding request failed"), "{message}");
    // Seeded defect: converting request failure to Ok(vec![vec![0.0; 1024]])
    // makes the expect_err above fail and turns this check red.
}
