use paper_graph::store::{open_database, stats};
use std::path::Path;

/// Opt-in check against a real HawDB database copy.
///
/// Run with:
/// `PAPER_GRAPH_REAL_DB=/path/to/library.hawdb cargo test -p paper-graph existing_database -- --ignored`
#[test]
#[ignore = "requires PAPER_GRAPH_REAL_DB pointing at a HawDB database"]
fn existing_database_has_the_expected_counts() {
    let path = std::env::var("PAPER_GRAPH_REAL_DB")
        .expect("set PAPER_GRAPH_REAL_DB to a HawDB database directory");
    let mut db = open_database(Some(Path::new(&path))).expect("open existing database");
    let counts = stats(&mut db).expect("read stats");
    assert_eq!(counts.works, 324);
    assert_eq!(counts.unfiled, 171);
    assert_eq!(counts.persons, 347);
    assert_eq!(counts.collections, 5);
    assert_eq!(counts.tags, 24);
    assert_eq!(counts.venues, 10);
    assert_eq!(counts.attachments, 161);
}
