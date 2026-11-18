//! HawDB schema, full rebuild, and search queries for the paper graph.

use anyhow::{bail, Context, Result};
use hawdb::{Database, DatabaseTransaction, QueryOutput, Value, ValueRef, Vector};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::zotero::{Attachment, Library, Work};

const WORK_COLUMNS: &str = "
    item_id BIGINT PRIMARY KEY,
    zotero_key TEXT NOT NULL,
    item_type TEXT NOT NULL,
    title TEXT NOT NULL,
    date TEXT,
    year BIGINT,
    venue TEXT,
    abstract TEXT,
    authors TEXT,
    collections TEXT,
    tags TEXT,
    search_text TEXT,
    embedding VECTOR(1024)
";

const INSERT_WORK: &str = "
    INSERT INTO works (
        item_id, zotero_key, item_type, title, date, year, venue,
        abstract, authors, collections, tags, search_text, embedding
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
";

const BM25_QUERY: &str = "
    SELECT w.item_id, w.title, w.year, w.authors, w.venue, w.collections,
           w.score
    FROM text_search('works_search', $1, top_k => $2) AS w
    ORDER BY w.score DESC, w.item_id
";

/// Open a database by path, or an in-memory database when `path` is `None`.
pub fn open_database(path: Option<&std::path::Path>) -> Result<Database> {
    match path {
        Some(path) => {
            Database::open(path).with_context(|| format!("open database {}", path.display()))
        }
        None => Ok(Database::new()),
    }
}

pub fn initialize(db: &mut Database) -> Result<()> {
    let table = db
        .query_sql("SELECT table_name FROM information_schema.tables WHERE table_name = 'works'")?;
    if table.rows.is_empty() {
        db.query_sql(&format!("CREATE TABLE works ({WORK_COLUMNS})"))?;
    }
    db.query_sql("CREATE SEARCH INDEX IF NOT EXISTS works_search ON works (search_text)")?;
    Ok(())
}

/// Replace all derived data in one transaction, keyed by Zotero IDs.
pub fn rebuild(db: &mut Database, library: &Library, embeddings: &[Vec<f32>]) -> Result<()> {
    if library.works.len() != embeddings.len() {
        bail!("Each work must have exactly one embedding");
    }
    initialize(db)?;

    let mut sql_rows: Vec<Vec<Value>> = Vec::with_capacity(library.works.len());
    for (work, embedding) in library.works.iter().zip(embeddings) {
        sql_rows.push(vec![
            Value::Int(work.id),
            Value::String(work.key.clone()),
            Value::String(work.item_type.clone()),
            Value::String(work.title.clone()),
            optional_string(work.date.clone()),
            optional_int(work.year()),
            optional_string(work.venue.clone()),
            Value::String(work.abstract_note.clone()),
            Value::String(work.author_names()),
            Value::String(work.collection_names()),
            Value::String(work.tag_names()),
            Value::String(work.search_text()),
            Value::Vector(Vector::new(embedding).context("build work embedding vector")?),
        ]);
    }

    let mut people: BTreeMap<i64, String> = BTreeMap::new();
    let mut collections: BTreeMap<i64, String> = BTreeMap::new();
    let mut tags: BTreeMap<i64, String> = BTreeMap::new();
    let mut venues: BTreeSet<String> = BTreeSet::new();
    for work in &library.works {
        for person in &work.authors {
            people.insert(person.id, person.name.clone());
        }
        for collection in &work.collections {
            collections.insert(collection.id, collection.name.clone());
        }
        for tag in &work.tags {
            tags.insert(tag.id, tag.name.clone());
        }
        if let Some(venue) = &work.venue {
            if !venue.is_empty() {
                venues.insert(venue.clone());
            }
        }
    }

    let mut transaction = db.begin_transaction()?;
    transaction.query_sql("DELETE FROM works WHERE item_id IS NOT NULL")?;
    transaction.query("MATCH (n) DETACH DELETE n")?;
    for row in &sql_rows {
        transaction.query_sql_with_params(INSERT_WORK, row)?;
    }

    let work_nodes: Vec<Value> = library
        .works
        .iter()
        .map(|work| {
            object(vec![
                ("id", Value::Int(work.id)),
                ("key", Value::String(work.key.clone())),
                ("title", Value::String(work.title.clone())),
                ("year", optional_int(work.year())),
            ])
        })
        .collect();
    run_unwind(
        &mut transaction,
        "UNWIND $rows AS row CREATE (:Work {id: row.id, zoteroKey: row.key, title: row.title, year: row.year})",
        work_nodes,
    )?;

    let person_nodes: Vec<Value> = people
        .iter()
        .map(|(id, name)| {
            object(vec![
                ("id", Value::Int(*id)),
                ("name", Value::String(name.clone())),
            ])
        })
        .collect();
    run_unwind(
        &mut transaction,
        "UNWIND $rows AS row CREATE (:Person {id: row.id, name: row.name})",
        person_nodes,
    )?;

    let collection_nodes: Vec<Value> = collections
        .iter()
        .map(|(id, name)| {
            object(vec![
                ("id", Value::Int(*id)),
                ("name", Value::String(name.clone())),
            ])
        })
        .collect();
    run_unwind(
        &mut transaction,
        "UNWIND $rows AS row CREATE (:Collection {id: row.id, name: row.name})",
        collection_nodes,
    )?;

    let tag_nodes: Vec<Value> = tags
        .iter()
        .map(|(id, name)| {
            object(vec![
                ("id", Value::Int(*id)),
                ("name", Value::String(name.clone())),
            ])
        })
        .collect();
    run_unwind(
        &mut transaction,
        "UNWIND $rows AS row CREATE (:Tag {id: row.id, name: row.name})",
        tag_nodes,
    )?;

    let venue_nodes: Vec<Value> = venues
        .iter()
        .map(|name| object(vec![("name", Value::String(name.clone()))]))
        .collect();
    run_unwind(
        &mut transaction,
        "UNWIND $rows AS row CREATE (:Venue {name: row.name})",
        venue_nodes,
    )?;

    insert_graph_edges(&mut transaction, &library.works, &library.attachments)?;
    transaction.commit()?;
    Ok(())
}

fn insert_graph_edges(
    transaction: &mut DatabaseTransaction<'_>,
    works: &[Work],
    attachments: &[Attachment],
) -> Result<()> {
    let mut author_edges = Vec::new();
    let mut collection_edges = Vec::new();
    let mut tag_edges = Vec::new();
    let mut venue_edges = Vec::new();
    for work in works {
        for person in &work.authors {
            author_edges.push(object(vec![
                ("work_id", Value::Int(work.id)),
                ("person_id", Value::Int(person.id)),
                ("order", Value::Int(person.order_index)),
            ]));
        }
        for collection in &work.collections {
            collection_edges.push(object(vec![
                ("work_id", Value::Int(work.id)),
                ("collection_id", Value::Int(collection.id)),
            ]));
        }
        for tag in &work.tags {
            tag_edges.push(object(vec![
                ("work_id", Value::Int(work.id)),
                ("tag_id", Value::Int(tag.id)),
            ]));
        }
        if let Some(venue) = &work.venue {
            if !venue.is_empty() {
                venue_edges.push(object(vec![
                    ("work_id", Value::Int(work.id)),
                    ("venue", Value::String(venue.clone())),
                ]));
            }
        }
    }

    run_unwind(
        transaction,
        "UNWIND $rows AS row MATCH (p:Person {id: row.person_id}), (w:Work {id: row.work_id})
         CREATE (p)-[:AUTHORED {authorOrder: row.order}]->(w)",
        author_edges,
    )?;
    run_unwind(
        transaction,
        "UNWIND $rows AS row MATCH (w:Work {id: row.work_id}), (c:Collection {id: row.collection_id})
         CREATE (w)-[:IN_COLLECTION]->(c)",
        collection_edges,
    )?;
    run_unwind(
        transaction,
        "UNWIND $rows AS row MATCH (w:Work {id: row.work_id}), (t:Tag {id: row.tag_id})
         CREATE (w)-[:TAGGED]->(t)",
        tag_edges,
    )?;
    run_unwind(
        transaction,
        "UNWIND $rows AS row MATCH (w:Work {id: row.work_id}), (v:Venue {name: row.venue})
         CREATE (w)-[:PUBLISHED_IN]->(v)",
        venue_edges,
    )?;

    let attachment_nodes: Vec<Value> = attachments
        .iter()
        .map(|item| {
            object(vec![
                ("id", Value::Int(item.id)),
                ("key", Value::String(item.key.clone())),
                ("path", optional_string(item.path.clone())),
                ("content_type", optional_string(item.content_type.clone())),
            ])
        })
        .collect();
    run_unwind(
        transaction,
        "UNWIND $rows AS row
         CREATE (:Attachment {id: row.id, zoteroKey: row.key, path: row.path, contentType: row.content_type})",
        attachment_nodes,
    )?;

    let attachment_edges: Vec<Value> = attachments
        .iter()
        .map(|item| {
            object(vec![
                ("work_id", Value::Int(item.parent_item_id)),
                ("attachment_id", Value::Int(item.id)),
            ])
        })
        .collect();
    run_unwind(
        transaction,
        "UNWIND $rows AS row MATCH (w:Work {id: row.work_id}), (a:Attachment {id: row.attachment_id})
         CREATE (w)-[:HAS_ATTACHMENT]->(a)",
        attachment_edges,
    )?;
    Ok(())
}

fn run_unwind(
    transaction: &mut DatabaseTransaction<'_>,
    cypher: &str,
    rows: Vec<Value>,
) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    let parameters = BTreeMap::from([("rows".to_string(), Value::List(rows))]);
    transaction
        .query_with_params(cypher, &parameters)
        .with_context(|| format!("run graph batch: {cypher}"))?;
    Ok(())
}

fn object(pairs: Vec<(&str, Value)>) -> Value {
    Value::Map(
        pairs
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn optional_string(value: Option<String>) -> Value {
    value.map(Value::String).unwrap_or(Value::Null)
}

fn optional_int(value: Option<i64>) -> Value {
    value.map(Value::Int).unwrap_or(Value::Null)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SearchRow {
    pub item_id: i64,
    pub title: String,
    pub year: Option<i64>,
    pub authors: Option<String>,
    pub venue: Option<String>,
    pub collections: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bm25_rank: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_rank: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rrf_score: Option<f64>,
}

pub fn bm25_search(db: &mut Database, query: &str, top_k: usize) -> Result<Vec<SearchRow>> {
    let output = db.query_sql_with_params(
        BM25_QUERY,
        &[Value::String(query.to_string()), Value::Int(top_k as i64)],
    )?;
    Ok(collect_search_rows(&output, true, false))
}

pub fn vector_search(db: &mut Database, embedding: &[f32], top_k: usize) -> Result<Vec<SearchRow>> {
    if !(1..=10_000).contains(&top_k) {
        bail!("top_k must be between 1 and 10000");
    }
    let sql = format!(
        "SELECT item_id, title, year, authors, venue, collections,
                cosine_distance(embedding, $1) AS distance
         FROM works
         WHERE embedding IS NOT NULL
         ORDER BY distance, item_id
         LIMIT {top_k}"
    );
    let vector = Vector::new(embedding).context("build query embedding vector")?;
    let output = db.query_sql_with_params(&sql, &[Value::Vector(vector)])?;
    Ok(collect_search_rows(&output, false, true))
}

pub fn hybrid_search(
    db: &mut Database,
    query: &str,
    embedding: &[f32],
    top_k: usize,
    bm25_weight: f64,
    vector_weight: f64,
) -> Result<Vec<SearchRow>> {
    let candidate_k = 10_000.min(20.max(top_k.saturating_mul(5)));
    let lexical_rows = bm25_search(db, query, candidate_k)?;
    let vector_rows = vector_search(db, embedding, candidate_k)?;
    let mut combined: BTreeMap<i64, SearchRow> = BTreeMap::new();

    for (index, row) in lexical_rows.into_iter().enumerate() {
        let rank = index + 1;
        let item = combined.entry(row.item_id).or_insert_with(|| row.clone());
        item.bm25_rank = Some(rank);
        item.rrf_score = Some(item.rrf_score.unwrap_or(0.0) + bm25_weight / (60.0 + rank as f64));
    }
    for (index, row) in vector_rows.into_iter().enumerate() {
        let rank = index + 1;
        let item = combined.entry(row.item_id).or_insert_with(|| row.clone());
        item.vector_rank = Some(rank);
        item.distance = row.distance;
        item.rrf_score = Some(item.rrf_score.unwrap_or(0.0) + vector_weight / (60.0 + rank as f64));
    }

    let mut rows: Vec<SearchRow> = combined.into_values().collect();
    rows.sort_by(|left, right| {
        let left_score = left.rrf_score.unwrap_or(0.0);
        let right_score = right.rrf_score.unwrap_or(0.0);
        right_score
            .total_cmp(&left_score)
            .then_with(|| left.item_id.cmp(&right.item_id))
    });
    rows.truncate(top_k);
    Ok(rows)
}

fn collect_search_rows(
    output: &QueryOutput,
    with_score: bool,
    with_distance: bool,
) -> Vec<SearchRow> {
    (0..output.rows.len())
        .map(|index| {
            let get = |name: &str| output.rows.get(index, name);
            SearchRow {
                item_id: as_i64(get("item_id")).unwrap_or(0),
                title: as_string(get("title")).unwrap_or_default(),
                year: as_i64(get("year")),
                authors: as_string(get("authors")),
                venue: as_string(get("venue")),
                collections: as_string(get("collections")),
                score: with_score.then(|| as_f64(get("score")).unwrap_or(0.0)),
                distance: with_distance.then(|| as_f64(get("distance")).unwrap_or(0.0)),
                bm25_rank: None,
                vector_rank: None,
                rrf_score: None,
            }
        })
        .collect()
}

fn as_i64(value: Option<ValueRef<'_>>) -> Option<i64> {
    match value {
        Some(ValueRef::Int(value)) => Some(value),
        _ => None,
    }
}

fn as_f64(value: Option<ValueRef<'_>>) -> Option<f64> {
    match value {
        Some(ValueRef::Float(value)) => Some(value),
        Some(ValueRef::Int(value)) => Some(value as f64),
        _ => None,
    }
}

fn as_string(value: Option<ValueRef<'_>>) -> Option<String> {
    match value {
        Some(ValueRef::String(value)) => Some(value.to_string()),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct Stats {
    pub works: i64,
    pub unfiled: i64,
    pub persons: i64,
    pub collections: i64,
    pub tags: i64,
    pub venues: i64,
    pub attachments: i64,
}

pub fn stats(db: &mut Database) -> Result<Stats> {
    let works = scalar(db.query_sql("SELECT COUNT(*) AS count FROM works")?);
    let unfiled = scalar(db.query_sql(
        "SELECT COUNT(*) AS count FROM works WHERE collections IS NULL OR collections = ''",
    )?);
    Ok(Stats {
        works,
        unfiled,
        persons: graph_count(db, "Person")?,
        collections: graph_count(db, "Collection")?,
        tags: graph_count(db, "Tag")?,
        venues: graph_count(db, "Venue")?,
        attachments: graph_count(db, "Attachment")?,
    })
}

fn graph_count(db: &mut Database, label: &str) -> Result<i64> {
    let output = db.query(&format!("MATCH (n:{label}) RETURN count(n) AS count"))?;
    Ok(scalar(output))
}

fn scalar(output: QueryOutput) -> i64 {
    as_i64(output.rows.get(0, "count")).unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AuthorWork {
    pub person: String,
    pub item_id: i64,
    pub title: String,
    pub year: Option<i64>,
}

pub fn author_works(db: &mut Database, name: &str) -> Result<Vec<AuthorWork>> {
    let parameters = BTreeMap::from([("name".to_string(), Value::String(name.to_string()))]);
    let output = db.query_with_params(
        "MATCH (p:Person)-[:AUTHORED]->(w:Work)
         WHERE p.name CONTAINS $name
         RETURN p.name AS person, w.id AS item_id, w.title AS title, w.year AS year
         ORDER BY p.name, year, title",
        &parameters,
    )?;
    Ok((0..output.rows.len())
        .map(|index| {
            let get = |column: &str| output.rows.get(index, column);
            AuthorWork {
                person: as_string(get("person")).unwrap_or_default(),
                item_id: as_i64(get("item_id")).unwrap_or(0),
                title: as_string(get("title")).unwrap_or_default(),
                year: as_i64(get("year")),
            }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct WorkGraph {
    pub people: Vec<PersonRef>,
    pub collections: Vec<NamedRef>,
    pub tags: Vec<NamedRef>,
    pub attachments: Vec<AttachmentRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PersonRef {
    pub id: i64,
    pub name: String,
    #[serde(rename = "authorOrder")]
    pub author_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NamedRef {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AttachmentRef {
    pub id: i64,
    #[serde(rename = "zoteroKey")]
    pub zotero_key: String,
    pub path: Option<String>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WorkDetail {
    pub item_id: i64,
    pub zotero_key: String,
    pub item_type: String,
    pub title: String,
    pub date: Option<String>,
    pub year: Option<i64>,
    pub venue: Option<String>,
    #[serde(rename = "abstract")]
    pub abstract_note: String,
    pub authors: Option<String>,
    pub collections: Option<String>,
    pub tags: Option<String>,
    pub graph: WorkGraph,
}

pub fn work_detail(db: &mut Database, item_id: i64) -> Result<Option<WorkDetail>> {
    let output = db.query_sql_with_params(
        "SELECT item_id, zotero_key, item_type, title, date, year, venue,
                abstract, authors, collections, tags
         FROM works WHERE item_id = $1",
        &[Value::Int(item_id)],
    )?;
    if output.rows.is_empty() {
        return Ok(None);
    }
    let get = |column: &str| output.rows.get(0, column);
    let mut detail = WorkDetail {
        item_id: as_i64(get("item_id")).unwrap_or(item_id),
        zotero_key: as_string(get("zotero_key")).unwrap_or_default(),
        item_type: as_string(get("item_type")).unwrap_or_default(),
        title: as_string(get("title")).unwrap_or_default(),
        date: as_string(get("date")),
        year: as_i64(get("year")),
        venue: as_string(get("venue")),
        abstract_note: as_string(get("abstract")).unwrap_or_default(),
        authors: as_string(get("authors")),
        collections: as_string(get("collections")),
        tags: as_string(get("tags")),
        graph: WorkGraph::default(),
    };

    let parameters = BTreeMap::from([("id".to_string(), Value::Int(item_id))]);

    let people = db.query_with_params(
        "MATCH (p:Person)-[r:AUTHORED]->(w:Work {id: $id}) RETURN p.id AS id, p.name AS name, r.authorOrder AS authorOrder ORDER BY authorOrder",
        &parameters,
    )?;
    detail.graph.people = (0..people.rows.len())
        .map(|index| {
            let get = |column: &str| people.rows.get(index, column);
            PersonRef {
                id: as_i64(get("id")).unwrap_or(0),
                name: as_string(get("name")).unwrap_or_default(),
                author_order: as_i64(get("authorOrder")),
            }
        })
        .collect();

    let collections = db.query_with_params(
        "MATCH (w:Work {id: $id})-[:IN_COLLECTION]->(c:Collection) RETURN c.id AS id, c.name AS name ORDER BY name",
        &parameters,
    )?;
    detail.graph.collections = collect_named(&collections);

    let tags = db.query_with_params(
        "MATCH (w:Work {id: $id})-[:TAGGED]->(t:Tag) RETURN t.id AS id, t.name AS name ORDER BY name",
        &parameters,
    )?;
    detail.graph.tags = collect_named(&tags);

    let attachments = db.query_with_params(
        "MATCH (w:Work {id: $id})-[:HAS_ATTACHMENT]->(a:Attachment) RETURN a.id AS id, a.zoteroKey AS zoteroKey, a.path AS path, a.contentType AS contentType",
        &parameters,
    )?;
    detail.graph.attachments = (0..attachments.rows.len())
        .map(|index| {
            let get = |column: &str| attachments.rows.get(index, column);
            AttachmentRef {
                id: as_i64(get("id")).unwrap_or(0),
                zotero_key: as_string(get("zoteroKey")).unwrap_or_default(),
                path: as_string(get("path")),
                content_type: as_string(get("contentType")),
            }
        })
        .collect();

    Ok(Some(detail))
}

fn collect_named(output: &QueryOutput) -> Vec<NamedRef> {
    (0..output.rows.len())
        .map(|index| {
            let get = |column: &str| output.rows.get(index, column);
            NamedRef {
                id: as_i64(get("id")).unwrap_or(0),
                name: as_string(get("name")).unwrap_or_default(),
            }
        })
        .collect()
}
