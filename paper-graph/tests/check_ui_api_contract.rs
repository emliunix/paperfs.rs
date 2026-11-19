use paper_graph::store::{initialize, open_database};
use paper_graph::ui;
use serde_json::{json, Value};
use std::time::Duration;

#[tokio::test]
async fn temp_hawdb_stats_and_missing_work_follow_http_json_contract() {
    // The fixture is created locally; no live HawDB, Zotero DB, or embeddings
    // router is needed. Failure to create it is a test failure, not a silent skip.
    let directory = tempfile::tempdir().expect("temporary UI HawDB fixture unavailable");
    let db_path = directory.path().join("library.hawdb");
    {
        let mut db = open_database(Some(&db_path)).expect("create temporary HawDB fixture");
        initialize(&mut db).expect("initialize temporary HawDB schema");
    }
    let reservation = std::net::TcpListener::bind("127.0.0.1:0").expect("reserve UI test port");
    let port = reservation.local_addr().expect("UI test address").port();
    drop(reservation);

    let server = tokio::spawn(ui::serve(
        db_path,
        directory.path().join("missing-zotero.sqlite"),
        "http://127.0.0.1:9/v1".to_string(),
        "fixture-model".to_string(),
        "127.0.0.1".to_string(),
        port,
    ));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("build UI test client");
    let base = format!("http://127.0.0.1:{port}");
    let mut last_error = String::new();
    let mut ready = false;
    for _ in 0..100 {
        if server.is_finished() {
            break;
        }
        match client.get(format!("{base}/")).send().await {
            Ok(response) if response.status() == reqwest::StatusCode::OK => {
                ready = true;
                break;
            }
            Ok(response) => last_error = format!("index returned {}", response.status()),
            Err(error) => last_error = error.to_string(),
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    if !ready {
        server.abort();
        panic!("UI fixture did not start: {last_error}");
    }

    let stats = client
        .get(format!("{base}/api/stats"))
        .send()
        .await
        .expect("request stats");
    assert_eq!(stats.status(), reqwest::StatusCode::OK);
    assert_eq!(stats.headers()["cache-control"], "no-store");
    assert_eq!(
        stats.headers()["content-type"],
        "application/json; charset=utf-8"
    );
    let body: Value = stats.json().await.expect("decode stats JSON");
    assert_eq!(
        body,
        json!({
            "works": 0,
            "unfiled": 0,
            "persons": 0,
            "collections": 0,
            "tags": 0,
            "venues": 0,
            "attachments": 0
        })
    );

    let missing = client
        .get(format!("{base}/api/work?id=999"))
        .send()
        .await
        .expect("request seeded missing work");
    assert_eq!(missing.status(), reqwest::StatusCode::NOT_FOUND);
    assert_eq!(missing.headers()["cache-control"], "no-store");
    let body: Value = missing.json().await.expect("decode error JSON");
    assert_eq!(body, json!({"error": "Work not found"}));

    server.abort();
    let _ = server.await;
    // Seeded defect: returning 200 for work 999 or omitting a stats field makes
    // the status/body assertions above fail.
}
