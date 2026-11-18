use paper_graph::zotero::read_library;
use rusqlite::Connection;

fn zotero_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().expect("temp dir");
    let path = directory.path().join("zotero.sqlite");
    let db = Connection::open(&path).expect("create fixture");
    db.execute_batch(
        r#"
        CREATE TABLE items (itemID INTEGER PRIMARY KEY, key TEXT NOT NULL, itemTypeID INTEGER NOT NULL);
        CREATE TABLE itemTypes (itemTypeID INTEGER PRIMARY KEY, typeName TEXT NOT NULL);
        CREATE TABLE deletedItems (itemID INTEGER PRIMARY KEY);
        CREATE TABLE fields (fieldID INTEGER PRIMARY KEY, fieldName TEXT NOT NULL);
        CREATE TABLE itemDataValues (valueID INTEGER PRIMARY KEY, value TEXT);
        CREATE TABLE itemData (itemID INTEGER, fieldID INTEGER, valueID INTEGER);
        CREATE TABLE creators (creatorID INTEGER PRIMARY KEY, firstName TEXT, lastName TEXT);
        CREATE TABLE itemCreators (itemID INTEGER, creatorID INTEGER, creatorTypeID INTEGER, orderIndex INTEGER);
        CREATE TABLE collections (collectionID INTEGER PRIMARY KEY, collectionName TEXT);
        CREATE TABLE collectionItems (itemID INTEGER, collectionID INTEGER);
        CREATE TABLE tags (tagID INTEGER PRIMARY KEY, name TEXT);
        CREATE TABLE itemTags (itemID INTEGER, tagID INTEGER);
        CREATE TABLE itemAttachments (itemID INTEGER PRIMARY KEY, parentItemID INTEGER, path TEXT, contentType TEXT);

        INSERT INTO itemTypes VALUES (1, 'journalArticle'), (2, 'attachment');
        INSERT INTO fields VALUES (1, 'title'), (2, 'date'), (3, 'publicationTitle'), (4, 'abstractNote');
        INSERT INTO items VALUES (1, 'ABCD2345', 1), (2, 'ATTACH01', 2), (3, 'DELETED1', 1);
        INSERT INTO deletedItems VALUES (3);
        INSERT INTO itemData VALUES (1, 1, 1), (1, 2, 2), (1, 3, 3), (1, 4, 4);
        INSERT INTO itemDataValues VALUES
            (1, 'Example title'),
            (2, '2023-00-00 2023'),
            (3, 'Example venue'),
            (4, 'Example abstract');
        INSERT INTO creators VALUES (8, 'Ada', 'Example');
        INSERT INTO itemCreators VALUES (1, 8, 8, 0);
        INSERT INTO collections VALUES (20, 'Papers');
        INSERT INTO collectionItems VALUES (1, 20);
        INSERT INTO tags VALUES (30, 'Type Theory');
        INSERT INTO itemTags VALUES (1, 30);
        INSERT INTO itemAttachments VALUES (2, 1, 'storage:paper.pdf', 'application/pdf');
        "#,
    )
    .expect("populate fixture");
    (directory, path)
}

#[test]
fn reads_a_zotero_snapshot_and_truncates_it() {
    let (_directory, path) = zotero_fixture();
    let mut library = read_library(&path).expect("read fixture");

    assert_eq!(library.works.len(), 1);
    let work = &library.works[0];
    assert_eq!(work.id, 1);
    assert_eq!(work.key, "ABCD2345");
    assert_eq!(work.item_type, "journalArticle");
    assert_eq!(work.title, "Example title");
    assert_eq!(work.date.as_deref(), Some("2023-00-00 2023"));
    assert_eq!(work.venue.as_deref(), Some("Example venue"));
    assert_eq!(work.abstract_note, "Example abstract");
    assert_eq!(work.year(), Some(2023));
    assert_eq!(work.authors.len(), 1);
    assert_eq!(work.authors[0].name, "Ada Example");
    assert_eq!(work.authors[0].order_index, 0);
    assert_eq!(work.collections[0].name, "Papers");
    assert_eq!(work.tags[0].name, "Type Theory");
    assert_eq!(
        work.search_text(),
        "Example title\nAda Example\nExample venue\nExample abstract\nType Theory"
    );

    assert_eq!(library.attachments.len(), 1);
    assert_eq!(library.attachments[0].id, 2);
    assert_eq!(library.attachments[0].parent_item_id, 1);
    assert_eq!(library.attachments[0].key, "ATTACH01");
    assert_eq!(
        library.attachments[0].path.as_deref(),
        Some("storage:paper.pdf")
    );

    library.truncate(0);
    assert!(library.works.is_empty());
    assert!(library.attachments.is_empty());
}

#[test]
fn missing_zotero_database_is_reported() {
    let error = read_library(std::path::Path::new("/nonexistent/zotero.sqlite"))
        .expect_err("missing database must fail");
    assert!(error.to_string().contains("Zotero database not found"));
}
