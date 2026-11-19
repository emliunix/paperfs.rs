use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AdmissionRecord {
    item_id: i64,
    zotero_key: String,
    attachment_id: i64,
    path: PathBuf,
    bytes: u64,
    signature_ok: bool,
    sha256: String,
    status: String,
}

/// Test-side executable admission specification; no production admission hook
/// exists yet. Hashing is streamed and compared with the stat'd byte length.
fn admit(
    item_id: i64,
    zotero_key: &str,
    attachment_id: i64,
    path: &Path,
    expected_sha256: &str,
) -> Result<AdmissionRecord> {
    let metadata = path
        .metadata()
        .with_context(|| format!("stat admission candidate {}", path.display()))?;
    if !metadata.is_file() || metadata.len() == 0 {
        bail!("admission candidate is not a non-empty regular file");
    }
    if zotero_key.trim().is_empty() {
        bail!("admission candidate has no zotero_key");
    }

    let mut file = File::open(path).context("open admission candidate")?;
    let mut signature = [0_u8; 5];
    file.read_exact(&mut signature)
        .context("read five-byte PDF signature")?;
    if &signature != b"%PDF-" {
        bail!("admission candidate lacks literal %PDF- signature");
    }

    let mut digest = Sha256::new();
    digest.update(signature);
    let mut hashed_bytes = signature.len() as u64;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer).context("hash admission candidate")?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        hashed_bytes += read as u64;
    }
    if hashed_bytes != metadata.len() {
        bail!("file changed during admission: stat/hash byte count mismatch");
    }
    let sha256 = hex::encode(digest.finalize());
    if sha256 != expected_sha256.to_ascii_lowercase() {
        bail!("SHA-256 mismatch: expected {expected_sha256}, got {sha256}");
    }

    Ok(AdmissionRecord {
        item_id,
        zotero_key: zotero_key.to_string(),
        attachment_id,
        path: path.to_path_buf(),
        bytes: metadata.len(),
        signature_ok: true,
        sha256,
        status: "admitted".to_string(),
    })
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[test]
fn executable_admission_records_stat_signature_and_sha256() {
    let directory = tempfile::tempdir().expect("temporary corpus");
    let path = directory.path().join("paper.pdf");
    let bytes = b"%PDF-1.7
1 0 obj
<<>>
endobj
%%EOF
";
    std::fs::write(&path, bytes).expect("write PDF fixture");

    let record = admit(42, "PDFKEY01", 43, &path, &digest(bytes)).expect("admit PDF");
    assert_eq!(record.item_id, 42);
    assert_eq!(record.attachment_id, 43);
    assert_eq!(record.bytes, bytes.len() as u64);
    assert!(record.signature_ok);
    assert_eq!(record.sha256, digest(bytes));
    assert_eq!(record.sha256.len(), 64);
    assert_eq!(record.status, "admitted");

    let json = serde_json::to_value(&record).expect("serialize admission record");
    for field in [
        "item_id",
        "zotero_key",
        "attachment_id",
        "path",
        "bytes",
        "signature_ok",
        "sha256",
        "status",
    ] {
        assert!(json.get(field).is_some(), "missing admission field {field}");
    }
}

#[test]
fn seeded_non_pdf_signature_is_rejected() {
    let directory = tempfile::tempdir().expect("temporary corpus");
    let path = directory.path().join("pretend.pdf");
    let bytes = b"not a pdf despite its extension";
    std::fs::write(&path, bytes).expect("write seeded defect");

    let error = admit(1, "BADPDF01", 2, &path, &digest(bytes))
        .expect_err("seeded non-PDF must fail admission");
    assert!(error.to_string().contains("%PDF-"), "{error:#}");
}

#[test]
fn seeded_sha256_mismatch_is_rejected() {
    let directory = tempfile::tempdir().expect("temporary corpus");
    let path = directory.path().join("tampered.pdf");
    std::fs::write(
        &path,
        b"%PDF-1.7
tampered
",
    )
    .expect("write seeded defect");

    let error = admit(
        1,
        "BADHASH1",
        2,
        &path,
        &digest(
            b"%PDF-1.7
original
",
        ),
    )
    .expect_err("seeded digest mismatch must fail admission");
    assert!(error.to_string().contains("SHA-256 mismatch"), "{error:#}");
}

#[test]
fn seeded_missing_file_is_rejected_by_stat() {
    let directory = tempfile::tempdir().expect("temporary corpus");
    let missing = directory.path().join("missing.pdf");
    let error = admit(1, "MISSING1", 2, &missing, &digest(b""))
        .expect_err("seeded missing file must fail admission");
    assert!(
        error.to_string().contains("stat admission candidate"),
        "{error:#}"
    );
}
