//! Read works from Zotero without modifying its database.

use anyhow::{bail, Context, Result};
use rusqlite::{backup::Backup, Connection, OpenFlags};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Default location of Zotero's SQLite database.
pub fn default_zotero_db() -> PathBuf {
    match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home).join("Zotero").join("zotero.sqlite"),
        None => PathBuf::from("Zotero").join("zotero.sqlite"),
    }
}

/// Expand a leading `~` the way Python's `Path.expanduser` does.
pub fn expand_tilde(path: &Path) -> PathBuf {
    let Some(text) = path.to_str() else {
        return path.to_path_buf();
    };
    let Some(home) = std::env::var_os("HOME") else {
        return path.to_path_buf();
    };
    if text == "~" {
        return PathBuf::from(home);
    }
    if let Some(rest) = text.strip_prefix("~/") {
        return PathBuf::from(home).join(rest);
    }
    path.to_path_buf()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    pub id: i64,
    pub name: String,
    pub order_index: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedId {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub id: i64,
    pub key: String,
    pub parent_item_id: i64,
    pub path: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Work {
    pub id: i64,
    pub key: String,
    pub item_type: String,
    pub title: String,
    pub date: Option<String>,
    pub venue: Option<String>,
    pub abstract_note: String,
    pub authors: Vec<Person>,
    pub collections: Vec<NamedId>,
    pub tags: Vec<NamedId>,
}

impl Work {
    pub fn year(&self) -> Option<i64> {
        year_from_date(self.date.as_deref())
    }

    pub fn author_names(&self) -> String {
        self.authors
            .iter()
            .map(|person| person.name.as_str())
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn collection_names(&self) -> String {
        self.collections
            .iter()
            .map(|collection| collection.name.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    pub fn tag_names(&self) -> String {
        self.tags
            .iter()
            .map(|tag| tag.name.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    pub fn search_text(&self) -> String {
        let parts = [
            self.title.clone(),
            self.author_names(),
            self.venue.clone().unwrap_or_default(),
            self.abstract_note.clone(),
            self.tag_names(),
        ];
        parts
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Library {
    pub works: Vec<Work>,
    pub attachments: Vec<Attachment>,
}

impl Library {
    /// Keep the first `limit` works and only attachments that belong to them.
    pub fn truncate(&mut self, limit: usize) {
        self.works.truncate(limit);
        let work_ids: std::collections::HashSet<i64> =
            self.works.iter().map(|work| work.id).collect();
        self.attachments
            .retain(|item| work_ids.contains(&item.parent_item_id));
    }
}

/// Extract the first four-digit `19xx`/`20xx` year from a Zotero date string.
pub fn year_from_date(value: Option<&str>) -> Option<i64> {
    let bytes = value?.as_bytes();
    for start in 0..bytes.len().saturating_sub(3) {
        let window = &bytes[start..start + 4];
        if !window.iter().all(u8::is_ascii_digit) {
            continue;
        }
        if (window[0] == b'1' && window[1] == b'9') || (window[0] == b'2' && window[1] == b'0') {
            return std::str::from_utf8(window).ok()?.parse().ok();
        }
    }
    None
}

/// Read one consistent Zotero snapshot through SQLite's backup API.
pub fn read_library(path: &Path) -> Result<Library> {
    let path = expand_tilde(path);
    let path = path.canonicalize().unwrap_or(path);
    if !path.is_file() {
        bail!("Zotero database not found: {}", path.display());
    }

    let temp_dir = tempfile::Builder::new()
        .prefix("paper-graph-zotero-")
        .tempdir()
        .context("create temporary directory for the Zotero snapshot")?;
    let snapshot_path = temp_dir.path().join("zotero.sqlite");

    let source = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("open Zotero database {}", path.display()))?;
    let mut snapshot = Connection::open(&snapshot_path)
        .with_context(|| format!("create Zotero snapshot at {}", snapshot_path.display()))?;
    {
        let backup = Backup::new(&source, &mut snapshot)?;
        backup.run_to_completion(100, Duration::from_millis(10), None)?;
    }
    drop(snapshot);
    drop(source);

    read_snapshot(&snapshot_path)
}

fn read_snapshot(path: &Path) -> Result<Library> {
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let mut works: BTreeMap<i64, Work> = BTreeMap::new();
    {
        let mut statement = db.prepare(
            "SELECT i.itemID, i.key, t.typeName
             FROM items i
             JOIN itemTypes t ON i.itemTypeID = t.itemTypeID
             WHERE i.itemID NOT IN (SELECT itemID FROM deletedItems)
               AND EXISTS (
                   SELECT 1 FROM itemData id
                   JOIN fields f ON id.fieldID = f.fieldID
                   WHERE id.itemID = i.itemID AND f.fieldName = 'title'
               )
             ORDER BY i.itemID",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(Work {
                id: row.get(0)?,
                key: row.get(1)?,
                item_type: row.get(2)?,
                title: String::new(),
                ..Work::default()
            })
        })?;
        for work in rows {
            let work = work?;
            works.insert(work.id, work);
        }
    }

    {
        let mut statement = db.prepare(
            "SELECT id.itemID, f.fieldName, v.value
             FROM itemData id
             JOIN fields f ON id.fieldID = f.fieldID
             JOIN itemDataValues v ON id.valueID = v.valueID
             WHERE f.fieldName IN ('title', 'date', 'publicationTitle', 'abstractNote')",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        for row in rows {
            let (item_id, field, value) = row?;
            let Some(work) = works.get_mut(&item_id) else {
                continue;
            };
            let value = value.unwrap_or_default();
            match field.as_str() {
                "title" => work.title = value,
                "date" => work.date = Some(value),
                "publicationTitle" => work.venue = Some(value),
                "abstractNote" => work.abstract_note = value,
                _ => {}
            }
        }
    }

    {
        let mut statement = db.prepare(
            "SELECT ic.itemID, c.creatorID, c.firstName, c.lastName, ic.orderIndex
             FROM itemCreators ic
             JOIN creators c ON ic.creatorID = c.creatorID
             WHERE ic.creatorTypeID = 8
             ORDER BY ic.itemID, ic.orderIndex",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?;
        for row in rows {
            let (item_id, creator_id, first, last, order_index) = row?;
            let Some(work) = works.get_mut(&item_id) else {
                continue;
            };
            let first = first.unwrap_or_default().trim().to_string();
            let last = last.unwrap_or_default().trim().to_string();
            let name = if first.is_empty() {
                last
            } else {
                format!("{first} {last}").trim().to_string()
            };
            work.authors.push(Person {
                id: creator_id,
                name,
                order_index,
            });
        }
    }

    {
        let mut statement = db.prepare(
            "SELECT ci.itemID, c.collectionID, c.collectionName
             FROM collectionItems ci
             JOIN collections c ON ci.collectionID = c.collectionID
             ORDER BY ci.itemID, c.collectionName",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        for row in rows {
            let (item_id, collection_id, name) = row?;
            if let Some(work) = works.get_mut(&item_id) {
                work.collections.push(NamedId {
                    id: collection_id,
                    name: name.unwrap_or_default(),
                });
            }
        }
    }

    {
        let mut statement = db.prepare(
            "SELECT it.itemID, t.tagID, t.name
             FROM itemTags it
             JOIN tags t ON it.tagID = t.tagID
             ORDER BY it.itemID, t.name",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        for row in rows {
            let (item_id, tag_id, name) = row?;
            if let Some(work) = works.get_mut(&item_id) {
                work.tags.push(NamedId {
                    id: tag_id,
                    name: name.unwrap_or_default(),
                });
            }
        }
    }

    let mut attachments = Vec::new();
    {
        let mut statement = db.prepare(
            "SELECT i.itemID, i.key, ia.parentItemID, ia.path, ia.contentType
             FROM itemAttachments ia
             JOIN items i ON ia.itemID = i.itemID
             WHERE ia.parentItemID IS NOT NULL
               AND i.itemID NOT IN (SELECT itemID FROM deletedItems)
             ORDER BY i.itemID",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(Attachment {
                id: row.get(0)?,
                key: row.get(1)?,
                parent_item_id: row.get(2)?,
                path: row.get(3)?,
                content_type: row.get(4)?,
            })
        })?;
        for attachment in rows {
            let attachment = attachment?;
            if works.contains_key(&attachment.parent_item_id) {
                attachments.push(attachment);
            }
        }
    }

    Ok(Library {
        works: works.into_values().collect(),
        attachments,
    })
}
