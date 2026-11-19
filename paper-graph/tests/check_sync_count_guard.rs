use anyhow::{bail, Context, Result};
use hawdb::{Database, ValueRef};
use paper_graph::store::{open_database, rebuild};
use paper_graph::zotero::{Library, Work};
use std::collections::BTreeMap;

const METADATA_TABLES: &[&str] = &["works"];
const METADATA_LABELS: &[&str] = &["Work", "Person", "Collection", "Tag", "Venue", "Attachment"];

fn scalar(output: hawdb::QueryOutput) -> Result<i64> {
    match output.rows.get(0, "count") {
        Some(ValueRef::Int(count)) => Ok(count),
        _ => bail!("count query returned no integer count"),
    }
}

/// Test-side specification of the production pre-sync hook.
///
/// Unknown schemas and failed count queries are errors, never zero. Graph rows
/// are counted generically so a new non-metadata label cannot bypass the guard.
fn non_metadata_counts(db: &mut Database) -> Result<BTreeMap<String, i64>> {
    let tables = db
        .query_sql("SELECT table_name FROM information_schema.tables")
        .context("enumerate tables before sync")?;
    let mut counts = BTreeMap::new();
    for index in 0..tables.rows.len() {
        let Some(ValueRef::String(table)) = tables.rows.get(index, "table_name") else {
            bail!("table enumeration returned a non-string table_name");
        };
        if METADATA_TABLES.contains(&table) {
            continue;
        }
        if table.is_empty()
            || !table
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            bail!("unsafe table identifier returned by schema: {table:?}");
        }
        let count = scalar(
            db.query_sql(&format!("SELECT COUNT(*) AS count FROM {table}"))
                .with_context(|| format!("count non-metadata table {table}"))?,
        )?;
        counts.insert(format!("table:{table}"), count);
    }

    let total_nodes = scalar(
        db.query("MATCH (n) RETURN count(n) AS count")
            .context("count all graph nodes before sync")?,
    )?;
    let mut metadata_nodes = 0;
    for label in METADATA_LABELS {
        metadata_nodes += scalar(
            db.query(&format!("MATCH (n:{label}) RETURN count(n) AS count"))
                .with_context(|| format!("count metadata label {label}"))?,
        )?;
    }
    let derived_nodes = total_nodes
        .checked_sub(metadata_nodes)
        .context("metadata node count exceeded total node count")?;
    counts.insert("graph:non_metadata".to_string(), derived_nodes);
    Ok(counts)
}

fn pre_sync_count_guard(db: &mut Database) -> Result<()> {
    let counts = non_metadata_counts(db)?;
    let populated: Vec<String> = counts
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(name, count)| format!("{name}={count}"))
        .collect();
    if !populated.is_empty() {
        bail!(
            "non-metadata count guard refused sync: {}",
            populated.join(", ")
        );
    }
    Ok(())
}

fn library() -> (Library, Vec<Vec<f32>>) {
    (
        Library {
            works: vec![Work {
                id: 1,
                key: "SYNC0001".to_string(),
                item_type: "journalArticle".to_string(),
                title: "Guard fixture".to_string(),
                ..Work::default()
            }],
            attachments: Vec::new(),
        },
        vec![vec![0.0; 1024]],
    )
}

fn count_claims(db: &mut Database) -> Result<i64> {
    scalar(db.query_sql("SELECT COUNT(*) AS count FROM claims")?)
}

#[test]
fn pre_sync_non_metadata_count_guard_fails_closed() {
    let (library, embeddings) = library();
    let mut db = open_database(None).expect("open in-memory database");
    rebuild(&mut db, &library, &embeddings).expect("populate metadata");
    db.query_sql("CREATE TABLE claims (id BIGINT PRIMARY KEY, statement TEXT NOT NULL)")
        .expect("create derived table");
    db.query_sql("INSERT INTO claims VALUES (1, 'must survive'), (2, 'must also survive')")
        .expect("populate derived table");

    let error = pre_sync_count_guard(&mut db).expect_err("derived rows must block sync");
    assert!(error.to_string().contains("table:claims=2"), "{error:#}");
    assert_eq!(count_claims(&mut db).expect("count after refusal"), 2);

    // Seeded defect: treating this populated table as metadata would make the
    // expected guard error disappear and turn this test red.
}

#[test]
fn populated_derived_graph_sync_preserves_extraction_counts() {
    let (library, embeddings) = library();
    let mut db = open_database(None).expect("open in-memory database");
    rebuild(&mut db, &library, &embeddings).expect("populate metadata");
    for id in 1..=3 {
        db.query(&format!("CREATE (:Claim {{id: {id}}})"))
            .expect("populate derived graph");
    }

    let error = pre_sync_count_guard(&mut db).expect_err("derived nodes must block sync");
    assert!(
        error.to_string().contains("graph:non_metadata=3"),
        "{error:#}"
    );
    let after = scalar(
        db.query("MATCH (n:Claim) RETURN count(n) AS count")
            .expect("count derived nodes after refusal"),
    )
    .expect("integer claim count");
    assert_eq!(after, 3, "derived rows changed: 3/{after}");
    println!("derived rows unchanged: 3/3");
}

#[test]
fn metadata_only_database_passes_guard_and_rebuilds() {
    let (library, embeddings) = library();
    let mut db = open_database(None).expect("open in-memory database");
    rebuild(&mut db, &library, &embeddings).expect("populate metadata");
    pre_sync_count_guard(&mut db).expect("metadata-only database is safe");
    rebuild(&mut db, &library, &embeddings).expect("guarded rebuild");
    pre_sync_count_guard(&mut db).expect("rebuild leaves no non-metadata state");
}
