use anyhow::{bail, Context, Result};
use hawdb::{Database, ValueRef};
use paper_graph::store::{open_database, rebuild};
use paper_graph::zotero::{Library, Work};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MigrationCounts {
    works: i64,
    claims: i64,
    evidence: i64,
    reviews: i64,
}

fn lint_no_startup_drops(source: &str) -> Result<()> {
    let tokens: Vec<String> = source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_uppercase)
        .collect();
    for window in tokens.windows(2) {
        if window[0] == "DROP" && (window[1] == "TABLE" || window[1] == "VIEW") {
            bail!("startup SQL contains forbidden DROP {}", window[1]);
        }
    }
    Ok(())
}

fn scalar(output: hawdb::QueryOutput) -> Result<i64> {
    match output.rows.get(0, "count") {
        Some(ValueRef::Int(count)) => Ok(count),
        _ => bail!("count query returned no integer count"),
    }
}

fn table_count(db: &mut Database, table: &str) -> Result<i64> {
    scalar(
        db.query_sql(&format!("SELECT COUNT(*) AS count FROM {table}"))
            .with_context(|| format!("count {table}"))?,
    )
}

fn migration_counts(db: &mut Database) -> Result<MigrationCounts> {
    Ok(MigrationCounts {
        works: table_count(db, "works")?,
        claims: table_count(db, "claims")?,
        evidence: table_count(db, "evidence")?,
        reviews: table_count(db, "reviews")?,
    })
}

fn verify_count_oracle(before: MigrationCounts, after: MigrationCounts) -> Result<()> {
    if before != after {
        bail!("migration count oracle mismatch: before={before:?}, after={after:?}");
    }
    Ok(())
}

fn populate_original(path: &Path) -> Result<MigrationCounts> {
    let mut db = open_database(Some(path))?;
    let library = Library {
        works: vec![
            Work {
                id: 1,
                key: "MIGRATE1".to_string(),
                item_type: "journalArticle".to_string(),
                title: "Migration fixture one".to_string(),
                ..Work::default()
            },
            Work {
                id: 2,
                key: "MIGRATE2".to_string(),
                item_type: "journalArticle".to_string(),
                title: "Migration fixture two".to_string(),
                ..Work::default()
            },
        ],
        attachments: Vec::new(),
    };
    rebuild(&mut db, &library, &[vec![0.0; 1024], vec![0.0; 1024]])?;
    db.query_sql("CREATE TABLE claims (id BIGINT PRIMARY KEY, statement TEXT NOT NULL)")?;
    db.query_sql("CREATE TABLE evidence (id BIGINT PRIMARY KEY, quote TEXT NOT NULL)")?;
    db.query_sql("CREATE TABLE reviews (id BIGINT PRIMARY KEY, verdict TEXT NOT NULL)")?;
    db.query_sql("INSERT INTO claims VALUES (1, 'one'), (2, 'two')")?;
    db.query_sql("INSERT INTO evidence VALUES (1, 'q1'), (2, 'q2'), (3, 'q3')")?;
    db.query_sql("INSERT INTO reviews VALUES (1, 'approved')")?;
    migration_counts(&mut db)
}

fn copy_database_tree(source: &Path, destination: &Path) -> Result<()> {
    let file_type = fs::symlink_metadata(source)?.file_type();
    if file_type.is_symlink() {
        bail!("database copy refuses symlink {}", source.display());
    }
    if file_type.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_database_tree(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else if file_type.is_file() {
        fs::copy(source, destination)?;
    } else {
        bail!("unsupported database file type at {}", source.display());
    }
    Ok(())
}

fn apply_additive_migration(db: &mut Database) -> Result<()> {
    lint_no_startup_drops(
        "CREATE TABLE migration_audit (id BIGINT PRIMARY KEY, note TEXT NOT NULL)",
    )?;
    let mut transaction = db.begin_transaction()?;
    transaction
        .query_sql("CREATE TABLE migration_audit (id BIGINT PRIMARY KEY, note TEXT NOT NULL)")?;
    transaction.commit()?;
    Ok(())
}

#[test]
fn startup_sources_and_additive_migration_have_no_drop_table_or_view() {
    for source in [
        include_str!("../src/main.rs"),
        include_str!("../src/store.rs"),
    ] {
        lint_no_startup_drops(source).expect("startup source must not drop schema objects");
    }
    lint_no_startup_drops("ALTER TABLE claims ADD COLUMN review_note TEXT")
        .expect("additive migration");
}

#[test]
fn seeded_drop_table_and_drop_view_statements_fail_lint() {
    for sql in [
        "DROP TABLE claims",
        "drop\n view if exists approved_claims",
        "DrOp TaBlE reviews",
    ] {
        let error = lint_no_startup_drops(sql).expect_err("seeded startup DROP must fail");
        assert!(error.to_string().contains("forbidden DROP"), "{error:#}");
    }
}

#[test]
fn additive_migration_on_database_copy_preserves_all_counts() {
    let directory = tempfile::tempdir().expect("temporary migration databases");
    let original = directory.path().join("original.hawdb");
    let copy = directory.path().join("copy.hawdb");
    let before = populate_original(&original).expect("populate original");
    copy_database_tree(&original, &copy).expect("copy closed database");

    let after = {
        let mut db = open_database(Some(&copy)).expect("open disposable copy");
        apply_additive_migration(&mut db).expect("apply additive migration to copy");
        migration_counts(&mut db).expect("count migrated copy")
    };
    verify_count_oracle(before, after).expect("work/claim/evidence/review counts survive");

    let mut original_db = open_database(Some(&original)).expect("reopen untouched original");
    assert_eq!(
        migration_counts(&mut original_db).expect("count original"),
        before,
        "migration must never touch the original"
    );
}

#[test]
fn seeded_row_loss_fails_count_oracle_without_touching_original() {
    let directory = tempfile::tempdir().expect("temporary migration databases");
    let original = directory.path().join("original.hawdb");
    let copy = directory.path().join("defective-copy.hawdb");
    let before = populate_original(&original).expect("populate original");
    copy_database_tree(&original, &copy).expect("copy closed database");

    let after = {
        let mut db = open_database(Some(&copy)).expect("open disposable copy");
        db.query_sql("DELETE FROM claims WHERE id = 2")
            .expect("seed row loss on copy");
        migration_counts(&mut db).expect("count defective copy")
    };
    let error = verify_count_oracle(before, after).expect_err("seeded row loss must fail oracle");
    assert!(error.to_string().contains("claims"), "{error:#}");

    let mut original_db = open_database(Some(&original)).expect("reopen untouched original");
    assert_eq!(
        migration_counts(&mut original_db).expect("count original"),
        before
    );
}
