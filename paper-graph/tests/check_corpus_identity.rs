use anyhow::{bail, Result};
use paper_graph::store::{open_database, rebuild, work_detail};
use paper_graph::zotero::{Library, Work};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct CorpusIdentity {
    #[serde(rename = "itemID", alias = "item_id")]
    item_id: i64,
    zotero_key: String,
    doi: Option<String>,
}

fn identity_map(records: &[CorpusIdentity]) -> Result<BTreeMap<i64, (String, Option<String>)>> {
    let mut by_item = BTreeMap::new();
    let mut keys = BTreeSet::new();
    let mut dois = BTreeSet::new();
    for record in records {
        let key = record.zotero_key.trim();
        if key.is_empty() {
            bail!("itemID {} has an empty zotero_key", record.item_id);
        }
        let doi = record
            .doi
            .as_deref()
            .map(str::trim)
            .filter(|doi| !doi.is_empty())
            .map(str::to_ascii_lowercase);
        if !keys.insert(key.to_string()) {
            bail!("zotero_key {key} maps to multiple itemIDs");
        }
        if let Some(doi) = &doi {
            if !dois.insert(doi.clone()) {
                bail!("DOI {doi} maps to multiple itemIDs");
            }
        }
        if by_item
            .insert(record.item_id, (key.to_string(), doi))
            .is_some()
        {
            bail!("itemID {} occurs more than once", record.item_id);
        }
    }
    Ok(by_item)
}

fn library(identities: &[(i64, &str)]) -> (Library, Vec<Vec<f32>>) {
    (
        Library {
            works: identities
                .iter()
                .map(|(id, key)| Work {
                    id: *id,
                    key: (*key).to_string(),
                    item_type: "journalArticle".to_string(),
                    title: format!("Identity fixture {key}"),
                    ..Work::default()
                })
                .collect(),
            attachments: Vec::new(),
        },
        identities.iter().map(|_| vec![0.0; 1024]).collect(),
    )
}

#[test]
fn item_id_zotero_key_and_doi_form_a_bijection() {
    let records: Vec<CorpusIdentity> = serde_json::from_str(
        r#"[
          {"itemID": 7, "zotero_key": "IDENT001", "doi": "10.1000/One"},
          {"item_id": 9, "zotero_key": "IDENT002", "doi": null}
        ]"#,
    )
    .expect("parse identity fixture");
    let map = identity_map(&records).expect("valid identity map");
    assert_eq!(map[&7].0, "IDENT001");
    assert_eq!(map[&7].1.as_deref(), Some("10.1000/one"));
    assert_eq!(map[&9].0, "IDENT002");

    let reverse: BTreeMap<&str, i64> = records
        .iter()
        .map(|record| (record.zotero_key.as_str(), record.item_id))
        .collect();
    assert_eq!(reverse["IDENT001"], 7);
    assert_eq!(reverse["IDENT002"], 9);
}

#[test]
fn public_rebuild_api_preserves_durable_keys_when_item_ids_change() {
    let (before, vectors) = library(&[(7, "IDENT001"), (9, "IDENT002")]);
    let mut first_db = open_database(None).expect("open first database");
    rebuild(&mut first_db, &before, &vectors).expect("first rebuild");
    assert_eq!(
        work_detail(&mut first_db, 7)
            .expect("query original work")
            .expect("original work exists")
            .zotero_key,
        "IDENT001"
    );

    let (after, vectors) = library(&[(700, "IDENT001"), (900, "IDENT002")]);
    let mut second_db = open_database(None).expect("open re-import database");
    rebuild(&mut second_db, &after, &vectors).expect("re-import with new itemIDs");
    for (old_id, new_id, key) in [(7, 700, "IDENT001"), (9, 900, "IDENT002")] {
        let reimported = work_detail(&mut second_db, new_id)
            .expect("query re-imported work")
            .expect("re-imported work exists");
        assert_eq!(reimported.zotero_key, key);
        assert_ne!(old_id, reimported.item_id);

        // A derived assertion bound to the durable key still resolves after
        // local itemID renumbering; binding it to old_id would not.
        let derived_work_key = "IDENT001";
        if derived_work_key == key {
            assert_eq!(reimported.zotero_key, derived_work_key);
        }
    }
}

#[test]
fn seeded_duplicate_zotero_key_is_rejected() {
    let records = vec![
        CorpusIdentity {
            item_id: 1,
            zotero_key: "DUPKEY01".to_string(),
            doi: None,
        },
        CorpusIdentity {
            item_id: 2,
            zotero_key: "DUPKEY01".to_string(),
            doi: None,
        },
    ];
    let error = identity_map(&records).expect_err("seeded duplicate key must fail");
    assert!(error.to_string().contains("multiple itemIDs"), "{error:#}");
}

#[test]
fn seeded_case_variant_duplicate_doi_is_rejected() {
    let records = vec![
        CorpusIdentity {
            item_id: 1,
            zotero_key: "DOIKEY01".to_string(),
            doi: Some("10.1000/Shared".to_string()),
        },
        CorpusIdentity {
            item_id: 2,
            zotero_key: "DOIKEY02".to_string(),
            doi: Some("10.1000/shared".to_string()),
        },
    ];
    let error = identity_map(&records).expect_err("seeded duplicate DOI must fail");
    assert!(error.to_string().contains("DOI"), "{error:#}");
}
