use paper_graph::store::{
    author_works, bm25_search, hybrid_search, open_database, rebuild, stats, vector_search,
    work_detail,
};
use paper_graph::zotero::{Library, NamedId, Person, Work};

fn sample_library() -> (Library, Vec<Vec<f32>>) {
    let works = vec![
        Work {
            id: 1,
            key: "AAAA1111".to_string(),
            item_type: "journalArticle".to_string(),
            title: "Bidirectional Typechecking".to_string(),
            abstract_note: "Higher-rank polymorphism".to_string(),
            authors: vec![Person {
                id: 10,
                name: "Ada Example".to_string(),
                order_index: 0,
            }],
            collections: vec![NamedId {
                id: 20,
                name: "Papers".to_string(),
            }],
            ..Work::default()
        },
        Work {
            id: 2,
            key: "BBBB2222".to_string(),
            item_type: "preprint".to_string(),
            title: "Recursion Schemes".to_string(),
            abstract_note: "Fixed points for functional programs".to_string(),
            authors: vec![Person {
                id: 11,
                name: "Grace Example".to_string(),
                order_index: 0,
            }],
            ..Work::default()
        },
    ];
    let mut first = vec![0.0_f32; 1024];
    first[0] = 1.0;
    let mut second = vec![0.0_f32; 1024];
    second[1] = 1.0;
    (
        Library {
            works,
            attachments: Vec::new(),
        },
        vec![first, second],
    )
}

#[test]
fn hawdb_graph_and_all_search_modes() {
    let (library, vectors) = sample_library();
    let mut db = open_database(None).expect("open in-memory database");
    rebuild(&mut db, &library, &vectors).expect("rebuild");
    let counts = stats(&mut db).expect("stats");
    assert_eq!(counts.works, 2);
    assert_eq!(counts.persons, 2);
    assert_eq!(counts.unfiled, 1);

    let lexical = bm25_search(&mut db, "polymorphism", 5).expect("bm25 search");
    assert_eq!(
        lexical.iter().map(|row| row.item_id).collect::<Vec<_>>(),
        vec![1]
    );

    let nearest = vector_search(&mut db, &vectors[0], 2).expect("vector search");
    assert_eq!(nearest[0].item_id, 1);
    assert_eq!(nearest[0].distance, Some(0.0));

    let hybrid =
        hybrid_search(&mut db, "fixed points", &vectors[1], 2, 1.0, 1.0).expect("hybrid search");
    assert_eq!(hybrid[0].item_id, 2);

    assert_eq!(
        author_works(&mut db, "Ada").expect("author search")[0].title,
        "Bidirectional Typechecking"
    );
}

#[test]
fn rebuild_is_idempotent() {
    let (library, vectors) = sample_library();
    let mut db = open_database(None).expect("open in-memory database");
    rebuild(&mut db, &library, &vectors).expect("first rebuild");
    rebuild(&mut db, &library, &vectors).expect("second rebuild");
    assert_eq!(stats(&mut db).expect("stats").works, 2);
    assert_eq!(
        author_works(&mut db, "Ada").expect("author search").len(),
        1
    );
}

#[test]
fn search_rows_omit_absent_score_keys() {
    let (library, vectors) = sample_library();
    let mut db = open_database(None).expect("open in-memory database");
    rebuild(&mut db, &library, &vectors).expect("rebuild");

    let lexical = bm25_search(&mut db, "polymorphism", 5).expect("bm25 search");
    let lexical_json = serde_json::to_value(&lexical[0]).expect("serialize bm25 row");
    assert!(lexical_json.get("score").is_some());
    assert!(lexical_json.get("distance").is_none());
    assert!(lexical_json.get("rrf_score").is_none());

    let nearest = vector_search(&mut db, &vectors[0], 2).expect("vector search");
    let vector_json = serde_json::to_value(&nearest[0]).expect("serialize vector row");
    assert!(vector_json.get("distance").is_some());
    assert!(vector_json.get("score").is_none());
}

#[test]
fn work_detail_exposes_sql_fields_and_graph_shape() {
    let (library, vectors) = sample_library();
    let mut db = open_database(None).expect("open in-memory database");
    rebuild(&mut db, &library, &vectors).expect("rebuild");

    let detail = work_detail(&mut db, 1)
        .expect("work detail query")
        .expect("work exists");
    assert_eq!(detail.title, "Bidirectional Typechecking");
    assert_eq!(detail.abstract_note, "Higher-rank polymorphism");
    assert_eq!(detail.graph.people.len(), 1);
    assert_eq!(detail.graph.people[0].name, "Ada Example");
    assert_eq!(detail.graph.people[0].author_order, Some(0));
    assert_eq!(detail.graph.collections.len(), 1);
    assert_eq!(detail.graph.collections[0].name, "Papers");
    assert!(detail.graph.tags.is_empty());
    assert!(detail.graph.attachments.is_empty());

    let serialized = serde_json::to_value(&detail).expect("serialize detail");
    assert_eq!(serialized["abstract"], "Higher-rank polymorphism");
    assert_eq!(serialized["zotero_key"], "AAAA1111");
    assert_eq!(serialized["graph"]["people"][0]["authorOrder"], 0);
    assert!(work_detail(&mut db, 999)
        .expect("missing work query")
        .is_none());
}
