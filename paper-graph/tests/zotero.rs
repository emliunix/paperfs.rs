use paper_graph::zotero::{year_from_date, Person, Work};

#[test]
fn year_from_zotero_date_variants() {
    assert_eq!(year_from_date(Some("2023-00-00 2023")), Some(2023));
    assert_eq!(year_from_date(Some("05/2024")), Some(2024));
    assert_eq!(year_from_date(Some("1992")), Some(1992));
    assert_eq!(year_from_date(Some("not dated")), None);
    assert_eq!(year_from_date(None), None);
}

#[test]
fn search_text_contains_metadata_not_only_title() {
    let work = Work {
        id: 7,
        key: "ABCD2345".to_string(),
        item_type: "journalArticle".to_string(),
        title: "Example title".to_string(),
        venue: Some("Example venue".to_string()),
        abstract_note: "Example abstract".to_string(),
        authors: vec![Person {
            id: 8,
            name: "Ada Example".to_string(),
            order_index: 0,
        }],
        ..Work::default()
    };
    assert_eq!(
        work.search_text(),
        "Example title\nAda Example\nExample venue\nExample abstract"
    );
}
