//! Stage 1 corpus admission and Stage 2 system-tool PDF page extraction.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::zotero::{expand_tilde, Attachment, Library};

pub const ADMISSION_SCHEMA_VERSION: &str = "stage-1-admission-v1";
pub const PAGE_SCHEMA_VERSION: &str = "stage-2-pages-v1";
pub const PDF_MAGIC: &[u8; 5] = b"%PDF-";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusManifest {
    pub schema_version: String,
    pub status: String,
    pub verified_on: String,
    pub verification: Vec<String>,
    pub papers: Vec<CorpusPaper>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusPaper {
    pub item_id: i64,
    pub split: String,
    pub domain: String,
    pub title: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionStatus {
    Admitted,
    MissingWork,
    MissingAttachment,
    UnresolvedAttachmentPath,
    MissingFile,
    IoError,
    InvalidSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AttachmentCandidateCheck {
    pub attachment_id: i64,
    pub attachment_key: String,
    pub raw_path: Option<String>,
    pub resolved_path: Option<String>,
    pub content_type: Option<String>,
    pub status: AdmissionStatus,
    pub bytes: Option<u64>,
    pub signature_ok: bool,
    pub sha256: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdmissionRecord {
    pub item_id: i64,
    pub zotero_key: Option<String>,
    pub split: String,
    pub domain: String,
    pub title: String,
    pub attachment_id: Option<i64>,
    pub attachment_key: Option<String>,
    pub path: Option<String>,
    pub bytes: Option<u64>,
    pub signature_ok: bool,
    pub sha256: Option<String>,
    pub status: AdmissionStatus,
    pub error: Option<String>,
    pub candidates_checked: Vec<AttachmentCandidateCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdmissionReport {
    pub schema_version: String,
    pub corpus_schema_version: String,
    pub storage_root: String,
    pub total_papers: usize,
    pub admitted_count: usize,
    pub failed_count: usize,
    pub records: Vec<AdmissionRecord>,
}

impl AdmissionReport {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != ADMISSION_SCHEMA_VERSION {
            bail!(
                "unexpected admission schema_version {}, expected {}",
                self.schema_version,
                ADMISSION_SCHEMA_VERSION
            );
        }
        if self.records.len() != self.total_papers {
            bail!(
                "admission record count {} != total_papers {}",
                self.records.len(),
                self.total_papers
            );
        }
        let mut seen_items = HashSet::new();
        let mut admitted = 0usize;
        let mut failed = 0usize;
        for record in &self.records {
            if !seen_items.insert(record.item_id) {
                bail!("duplicate item_id {} in admission report", record.item_id);
            }
            match record.status {
                AdmissionStatus::Admitted => {
                    admitted += 1;
                    if record.attachment_id.is_none() {
                        bail!("admitted item {} missing attachment_id", record.item_id);
                    }
                    let Some(path) = &record.path else {
                        bail!("admitted item {} missing path", record.item_id);
                    };
                    if path.is_empty() {
                        bail!("admitted item {} has empty path", record.item_id);
                    }
                    if record.bytes.unwrap_or(0) < PDF_MAGIC.len() as u64 {
                        bail!("admitted item {} has invalid byte length", record.item_id);
                    }
                    if !record.signature_ok {
                        bail!("admitted item {} has signature_ok=false", record.item_id);
                    }
                    let Some(digest) = &record.sha256 else {
                        bail!("admitted item {} missing sha256", record.item_id);
                    };
                    if !is_valid_sha256_hex(digest) {
                        bail!("admitted item {} has invalid sha256 {}", record.item_id, digest);
                    }
                }
                _ => {
                    failed += 1;
                    if record.error.as_deref().unwrap_or_default().is_empty() {
                        bail!(
                            "failed admission item {} ({:?}) missing explicit error message",
                            record.item_id,
                            record.status
                        );
                    }
                }
            }
        }
        if admitted != self.admitted_count || failed != self.failed_count {
            bail!(
                "admission counts mismatch: header admitted={} failed={}, actual admitted={} failed={}",
                self.admitted_count,
                self.failed_count,
                admitted,
                failed
            );
        }
        Ok(())
    }
}

pub fn is_valid_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

pub fn load_corpus_manifest(path: &Path) -> Result<CorpusManifest> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("read corpus manifest {}", path.display()))?;
    let manifest: CorpusManifest = serde_json::from_str(&text)
        .with_context(|| format!("parse corpus manifest {}", path.display()))?;
    if manifest.papers.is_empty() {
        bail!("corpus manifest {} has zero papers", path.display());
    }
    Ok(manifest)
}

pub fn zotero_storage_root(zotero_db_path: &Path) -> PathBuf {
    let expanded = expand_tilde(zotero_db_path);
    expanded
        .parent()
        .map(|parent| parent.join("storage"))
        .unwrap_or_else(|| PathBuf::from("storage"))
}

pub fn resolve_attachment_path(storage_root: &Path, attachment: &Attachment) -> Option<PathBuf> {
    let raw = attachment.path.as_deref()?.trim();
    if raw.is_empty() {
        return None;
    }
    if let Some(rel) = raw.strip_prefix("storage:") {
        if rel.is_empty() {
            return None;
        }
        return Some(storage_root.join(&attachment.key).join(rel));
    }
    if let Some(rel) = raw.strip_prefix("attachments:") {
        if rel.is_empty() {
            return None;
        }
        return Some(expand_tilde(Path::new(rel)));
    }
    let candidate = expand_tilde(Path::new(raw));
    if candidate.is_absolute() {
        Some(candidate)
    } else {
        Some(storage_root.join(&attachment.key).join(candidate))
    }
}

fn is_pdf_candidate(attachment: &Attachment) -> bool {
    let content_type_pdf = attachment
        .content_type
        .as_deref()
        .map(|ct| ct.eq_ignore_ascii_case("application/pdf"))
        .unwrap_or(false);
    let path_pdf = attachment
        .path
        .as_deref()
        .map(|p| p.to_ascii_lowercase().ends_with(".pdf"))
        .unwrap_or(false);
    content_type_pdf || path_pdf
}

pub fn inspect_pdf_file(path: &Path) -> (AdmissionStatus, Option<u64>, bool, Option<String>, Option<String>) {
    let metadata = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return (
                AdmissionStatus::MissingFile,
                None,
                false,
                None,
                Some(format!("file not found: {}", path.display())),
            );
        }
        Err(err) => {
            return (
                AdmissionStatus::IoError,
                None,
                false,
                None,
                Some(format!("metadata error for {}: {}", path.display(), err)),
            );
        }
    };
    if !metadata.is_file() {
        return (
            AdmissionStatus::MissingFile,
            None,
            false,
            None,
            Some(format!("not a regular file: {}", path.display())),
        );
    }
    let byte_len = metadata.len();
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(err) => {
            return (
                AdmissionStatus::IoError,
                Some(byte_len),
                false,
                None,
                Some(format!("open error for {}: {}", path.display(), err)),
            );
        }
    };

    let mut hasher = Sha256::new();
    let mut header = Vec::with_capacity(PDF_MAGIC.len());
    let mut buf = [0u8; 65536];
    loop {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if header.len() < PDF_MAGIC.len() {
                    let needed = (PDF_MAGIC.len() - header.len()).min(n);
                    header.extend_from_slice(&buf[..needed]);
                }
                hasher.update(&buf[..n]);
            }
            Err(err) => {
                return (
                    AdmissionStatus::IoError,
                    Some(byte_len),
                    false,
                    None,
                    Some(format!("read error for {}: {}", path.display(), err)),
                );
            }
        }
    }

    let sha256 = hex::encode(hasher.finalize());
    let sig_ok = header.as_slice() == PDF_MAGIC;
    if sig_ok {
        (AdmissionStatus::Admitted, Some(byte_len), true, Some(sha256), None)
    } else {
        (
            AdmissionStatus::InvalidSignature,
            Some(byte_len),
            false,
            Some(sha256),
            Some(format!(
                "file {} does not start with literal %PDF- header",
                path.display()
            )),
        )
    }
}

/// Run Stage 1 corpus admission on every paper in `corpus`.
///
/// Every paper produces exactly one `AdmissionRecord`; missing works, missing
/// attachments, missing files, and invalid signatures emit typed failure records
/// and are never silently skipped.
pub fn admit_corpus(
    corpus: &CorpusManifest,
    library: &Library,
    storage_root: &Path,
) -> Result<AdmissionReport> {
    let works_by_id: BTreeMap<i64, &crate::zotero::Work> =
        library.works.iter().map(|w| (w.id, w)).collect();
    let mut attachments_by_parent: BTreeMap<i64, Vec<&Attachment>> = BTreeMap::new();
    for att in &library.attachments {
        attachments_by_parent
            .entry(att.parent_item_id)
            .or_default()
            .push(att);
    }

    let mut records = Vec::with_capacity(corpus.papers.len());
    for paper in &corpus.papers {
        let Some(work) = works_by_id.get(&paper.item_id) else {
            records.push(AdmissionRecord {
                item_id: paper.item_id,
                zotero_key: None,
                split: paper.split.clone(),
                domain: paper.domain.clone(),
                title: paper.title.clone(),
                attachment_id: None,
                attachment_key: None,
                path: None,
                bytes: None,
                signature_ok: false,
                sha256: None,
                status: AdmissionStatus::MissingWork,
                error: Some(format!(
                    "work item_id {} not found in Zotero library snapshot",
                    paper.item_id
                )),
                candidates_checked: Vec::new(),
            });
            continue;
        };

        let all_atts = attachments_by_parent
            .get(&paper.item_id)
            .cloned()
            .unwrap_or_default();
        let pdf_atts: Vec<&Attachment> = all_atts
            .iter()
            .copied()
            .filter(|att| is_pdf_candidate(att))
            .collect();

        if pdf_atts.is_empty() {
            records.push(AdmissionRecord {
                item_id: paper.item_id,
                zotero_key: Some(work.key.clone()),
                split: paper.split.clone(),
                domain: paper.domain.clone(),
                title: paper.title.clone(),
                attachment_id: None,
                attachment_key: None,
                path: None,
                bytes: None,
                signature_ok: false,
                sha256: None,
                status: AdmissionStatus::MissingAttachment,
                error: Some(format!(
                    "work item_id {} has {} attachment(s), none PDF",
                    paper.item_id,
                    all_atts.len()
                )),
                candidates_checked: Vec::new(),
            });
            continue;
        }

        let mut candidates_checked = Vec::with_capacity(pdf_atts.len());
        let mut chosen: Option<AttachmentCandidateCheck> = None;

        for att in pdf_atts {
            let check = match resolve_attachment_path(storage_root, att) {
                None => AttachmentCandidateCheck {
                    attachment_id: att.id,
                    attachment_key: att.key.clone(),
                    raw_path: att.path.clone(),
                    resolved_path: None,
                    content_type: att.content_type.clone(),
                    status: AdmissionStatus::UnresolvedAttachmentPath,
                    bytes: None,
                    signature_ok: false,
                    sha256: None,
                    error: Some(format!(
                        "attachment {} has empty or unresolvable path {:?}",
                        att.id, att.path
                    )),
                },
                Some(resolved) => {
                    let (status, bytes, signature_ok, sha256, error) = inspect_pdf_file(&resolved);
                    AttachmentCandidateCheck {
                        attachment_id: att.id,
                        attachment_key: att.key.clone(),
                        raw_path: att.path.clone(),
                        resolved_path: Some(resolved.display().to_string()),
                        content_type: att.content_type.clone(),
                        status,
                        bytes,
                        signature_ok,
                        sha256,
                        error,
                    }
                }
            };
            if chosen.is_none() && check.status == AdmissionStatus::Admitted {
                chosen = Some(check.clone());
            }
            candidates_checked.push(check);
        }

        if let Some(winner) = chosen {
            records.push(AdmissionRecord {
                item_id: paper.item_id,
                zotero_key: Some(work.key.clone()),
                split: paper.split.clone(),
                domain: paper.domain.clone(),
                title: paper.title.clone(),
                attachment_id: Some(winner.attachment_id),
                attachment_key: Some(winner.attachment_key),
                path: winner.resolved_path,
                bytes: winner.bytes,
                signature_ok: winner.signature_ok,
                sha256: winner.sha256,
                status: AdmissionStatus::Admitted,
                error: None,
                candidates_checked,
            });
        } else {
            let primary = candidates_checked
                .last()
                .expect("at least one pdf candidate checked")
                .clone();
            let combined_errors = candidates_checked
                .iter()
                .filter_map(|c| c.error.as_deref())
                .collect::<Vec<_>>()
                .join("; ");
            records.push(AdmissionRecord {
                item_id: paper.item_id,
                zotero_key: Some(work.key.clone()),
                split: paper.split.clone(),
                domain: paper.domain.clone(),
                title: paper.title.clone(),
                attachment_id: Some(primary.attachment_id),
                attachment_key: Some(primary.attachment_key),
                path: primary.resolved_path,
                bytes: primary.bytes,
                signature_ok: primary.signature_ok,
                sha256: primary.sha256,
                status: primary.status,
                error: Some(combined_errors),
                candidates_checked,
            });
        }
    }

    let admitted_count = records
        .iter()
        .filter(|r| r.status == AdmissionStatus::Admitted)
        .count();
    let failed_count = records.len() - admitted_count;

    let report = AdmissionReport {
        schema_version: ADMISSION_SCHEMA_VERSION.to_string(),
        corpus_schema_version: corpus.schema_version.clone(),
        storage_root: storage_root.display().to_string(),
        total_papers: corpus.papers.len(),
        admitted_count,
        failed_count,
        records,
    };
    report.validate()?;
    Ok(report)
}

pub fn write_admission_report(path: &Path, report: &AdmissionReport) -> Result<()> {
    report.validate()?;
    let json = serde_json::to_string_pretty(report)?;
    let roundtrip: AdmissionReport =
        serde_json::from_str(&json).context("serde roundtrip validation for AdmissionReport")?;
    roundtrip.validate()?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create directory {}", parent.display()))?;
        }
    }
    fs::write(path, format!("{json}\n"))
        .with_context(|| format!("write admission report {}", path.display()))
}

pub fn read_admission_report(path: &Path) -> Result<AdmissionReport> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let report: AdmissionReport =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    report.validate()?;
    Ok(report)
}

// ============================================================================
// Stage 2: System-tool PDF page extraction
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PdfToolKind {
    Pdftotext,
    Mutool,
    MacosPdfkitSwift,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SystemPdfTool {
    pub kind: PdfToolKind,
    pub executable: String,
    pub parser_version: String,
    pub reading_order_mode: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PageParseStatus {
    Ok,
    EmptyPage,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PageOrderMetadata {
    pub page_index_zero_based: usize,
    pub total_pages_in_document: usize,
    pub reading_order_mode: String,
    pub char_count: usize,
    pub line_count: usize,
    pub replacement_char_count: usize,
}

/// Typed Stage 2 record written to `pages.jsonl`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PageRecord {
    pub schema_version: String,
    pub paper_id: i64,
    pub attachment_id: i64,
    pub attachment_sha256: String,
    pub page: usize,
    pub text: String,
    pub order_metadata: PageOrderMetadata,
    pub parser_version: String,
    pub status: PageParseStatus,
    pub error: Option<String>,
}

impl PageRecord {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != PAGE_SCHEMA_VERSION {
            bail!(
                "unexpected page schema_version {}, expected {}",
                self.schema_version,
                PAGE_SCHEMA_VERSION
            );
        }
        if !is_valid_sha256_hex(&self.attachment_sha256) {
            bail!(
                "invalid attachment_sha256 {} on paper_id {}",
                self.attachment_sha256,
                self.paper_id
            );
        }
        if self.parser_version.trim().is_empty() {
            bail!("empty parser_version on paper_id {}", self.paper_id);
        }
        match self.status {
            PageParseStatus::Ok => {
                if self.page == 0 {
                    bail!("page must be >= 1 for status=ok on paper_id {}", self.paper_id);
                }
                if self.text.trim().is_empty() {
                    bail!(
                        "status=ok page {} on paper_id {} has empty text; must be empty_page",
                        self.page,
                        self.paper_id
                    );
                }
                if self.order_metadata.page_index_zero_based + 1 != self.page {
                    bail!(
                        "page index mismatch on paper_id {} page {}",
                        self.paper_id,
                        self.page
                    );
                }
                if self.page > self.order_metadata.total_pages_in_document {
                    bail!(
                        "page {} exceeds total_pages_in_document {} on paper_id {}",
                        self.page,
                        self.order_metadata.total_pages_in_document,
                        self.paper_id
                    );
                }
                if self.order_metadata.char_count != self.text.chars().count() {
                    bail!(
                        "char_count mismatch on paper_id {} page {}",
                        self.paper_id,
                        self.page
                    );
                }
            }
            PageParseStatus::EmptyPage => {
                if self.page == 0 {
                    bail!(
                        "page must be >= 1 for status=empty_page on paper_id {}",
                        self.paper_id
                    );
                }
                if self.error.as_deref().unwrap_or_default().is_empty() {
                    bail!(
                        "empty_page on paper_id {} page {} must record diagnostic note in error",
                        self.paper_id,
                        self.page
                    );
                }
            }
            PageParseStatus::Failed => {
                if self.error.as_deref().unwrap_or_default().is_empty() {
                    bail!(
                        "failed page record on paper_id {} missing error message",
                        self.paper_id
                    );
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PaperParseSummary {
    pub paper_id: i64,
    pub attachment_id: Option<i64>,
    pub split: String,
    pub parsed_ok: bool,
    pub total_pages: usize,
    pub empty_pages: usize,
    pub total_chars: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ParseRunReport {
    pub schema_version: String,
    pub parser_tool: SystemPdfTool,
    pub split_filter: Option<String>,
    pub target_papers: usize,
    pub succeeded_papers: usize,
    pub failed_papers: usize,
    pub total_page_records: usize,
    pub papers: Vec<PaperParseSummary>,
}

fn find_executable_in_path(binary: &str) -> Option<PathBuf> {
    let mut search_dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect())
        .unwrap_or_default();
    for extra in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"] {
        let p = PathBuf::from(extra);
        if !search_dirs.contains(&p) {
            search_dirs.push(p);
        }
    }
    for dir in search_dirs {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Detect an available system PDF extraction tool.
///
/// Checks `pdftotext` first, then `mutool`, then macOS `/usr/bin/swift` + `PDFKit`.
pub fn detect_system_pdf_tool() -> Result<SystemPdfTool> {
    if let Some(exe) = find_executable_in_path("pdftotext") {
        let out = Command::new(&exe).arg("-v").output();
        let ver_raw = match out {
            Ok(o) => {
                let combined = format!(
                    "{}\n{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                );
                combined
                    .lines()
                    .find(|l| l.to_ascii_lowercase().contains("pdftotext"))
                    .unwrap_or("unknown")
                    .trim()
                    .to_string()
            }
            Err(_) => "unknown".to_string(),
        };
        return Ok(SystemPdfTool {
            kind: PdfToolKind::Pdftotext,
            executable: exe.display().to_string(),
            parser_version: format!("pdftotext:{ver_raw}"),
            reading_order_mode: "poppler_stream_order".to_string(),
        });
    }

    if let Some(exe) = find_executable_in_path("mutool") {
        let out = Command::new(&exe).arg("-v").output();
        let ver_raw = match out {
            Ok(o) => {
                let combined = format!(
                    "{}\n{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                );
                combined
                    .lines()
                    .next()
                    .unwrap_or("unknown")
                    .trim()
                    .to_string()
            }
            Err(_) => "unknown".to_string(),
        };
        return Ok(SystemPdfTool {
            kind: PdfToolKind::Mutool,
            executable: exe.display().to_string(),
            parser_version: format!("mutool:{ver_raw}"),
            reading_order_mode: "mupdf_stext_blocks".to_string(),
        });
    }

    if cfg!(target_os = "macos") {
        if let Some(exe) = find_executable_in_path("swift") {
            let probe_expr = r#"import Foundation; import PDFKit; let v = Bundle(for: PDFDocument.self).infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0"; print("pdfkit-\(v)")"#;
            let out = Command::new(&exe)
                .args(["-e", probe_expr])
                .output();
            if let Ok(o) = out {
                if o.status.success() {
                    let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    let label = if ver.is_empty() {
                        "macos-pdfkit-swift-v1".to_string()
                    } else {
                        format!("macos-{ver}-swift-v1")
                    };
                    return Ok(SystemPdfTool {
                        kind: PdfToolKind::MacosPdfkitSwift,
                        executable: exe.display().to_string(),
                        parser_version: label,
                        reading_order_mode: "pdfkit_column_flow".to_string(),
                    });
                }
            }
        }
    }

    bail!("no supported system PDF text extractor found (checked pdftotext, mutool, and macOS swift PDFKit)")
}

const SWIFT_PDFKIT_SCRIPT: &str = r#"
import Foundation
import PDFKit

struct ExtractedPage: Codable {
    let page: Int
    let text: String
}

struct ExtractedDoc: Codable {
    let total_pages: Int
    let pages: [ExtractedPage]
}

guard CommandLine.arguments.count >= 2 else {
    fputs("missing PDF path argument\n", stderr)
    exit(2)
}
let path = CommandLine.arguments[1]
let url = URL(fileURLWithPath: path)
guard let doc = PDFDocument(url: url) else {
    fputs("PDFKit failed to open PDF at \(path)\n", stderr)
    exit(3)
}
let count = doc.pageCount
var pages: [ExtractedPage] = []
pages.reserveCapacity(count)
for idx in 0..<count {
    let raw = doc.page(at: idx)?.string ?? ""
    pages.append(ExtractedPage(page: idx + 1, text: raw))
}
let payload = ExtractedDoc(total_pages: count, pages: pages)
let encoder = JSONEncoder()
let data = try encoder.encode(payload)
FileHandle.standardOutput.write(data)
"#;
#[derive(Debug, Deserialize)]
struct SwiftExtractedDoc {
    total_pages: usize,
    pages: Vec<SwiftExtractedPage>,
}

#[derive(Debug, Deserialize)]
struct SwiftExtractedPage {
    page: usize,
    text: String,
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

/// Extract raw text per 1-based page from a single PDF using `tool`.
pub fn extract_pdf_pages_with_tool(
    pdf_path: &Path,
    tool: &SystemPdfTool,
) -> Result<Vec<(usize, String)>> {
    let (status, _, sig_ok, _, err) = inspect_pdf_file(pdf_path);
    if status != AdmissionStatus::Admitted || !sig_ok {
        bail!(
            "cannot parse non-admitted PDF {}: {}",
            pdf_path.display(),
            err.unwrap_or_else(|| format!("{status:?}"))
        );
    }

    match tool.kind {
        PdfToolKind::Pdftotext => {
            let output = Command::new(&tool.executable)
                .arg("-enc")
                .arg("UTF-8")
                .arg(pdf_path)
                .arg("-")
                .output()
                .with_context(|| format!("invoke pdftotext on {}", pdf_path.display()))?;
            if !output.status.success() {
                bail!(
                    "pdftotext exited with {}: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                );
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut parts: Vec<&str> = stdout.split('\x0c').collect();
            if parts.len() > 1 && parts.last().map(|s| s.is_empty()).unwrap_or(false) {
                parts.pop();
            }
            if parts.is_empty() {
                bail!("pdftotext produced 0 pages for {}", pdf_path.display());
            }
            Ok(parts
                .into_iter()
                .enumerate()
                .map(|(i, s)| (i + 1, normalize_newlines(s)))
                .collect())
        }
        PdfToolKind::Mutool => {
            let output = Command::new(&tool.executable)
                .args(["draw", "-F", "txt", "-o", "-"])
                .arg(pdf_path)
                .output()
                .with_context(|| format!("invoke mutool on {}", pdf_path.display()))?;
            if !output.status.success() {
                bail!(
                    "mutool exited with {}: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                );
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut parts: Vec<&str> = stdout.split('\x0c').collect();
            if parts.len() > 1 && parts.last().map(|s| s.is_empty()).unwrap_or(false) {
                parts.pop();
            }
            if parts.is_empty() {
                bail!("mutool produced 0 pages for {}", pdf_path.display());
            }
            Ok(parts
                .into_iter()
                .enumerate()
                .map(|(i, s)| (i + 1, normalize_newlines(s)))
                .collect())
        }
        PdfToolKind::MacosPdfkitSwift => {
            let mut child = Command::new(&tool.executable)
                .arg("-")
                .arg(pdf_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .with_context(|| format!("spawn swift PDFKit extractor for {}", pdf_path.display()))?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(SWIFT_PDFKIT_SCRIPT.as_bytes())
                    .context("write swift PDFKit script to stdin")?;
            }
            let output = child
                .wait_with_output()
                .with_context(|| format!("wait on swift PDFKit extractor for {}", pdf_path.display()))?;
            if !output.status.success() {
                bail!(
                    "swift PDFKit extractor failed ({}): {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                );
            }
            let extracted: SwiftExtractedDoc = serde_json::from_slice(&output.stdout)
                .context("decode JSON from swift PDFKit extractor")?;
            if extracted.total_pages == 0 || extracted.pages.is_empty() {
                bail!("PDFKit reported 0 pages for {}", pdf_path.display());
            }
            Ok(extracted
                .pages
                .into_iter()
                .map(|p| (p.page, normalize_newlines(&p.text)))
                .collect())
        }
    }
}

/// Run Stage 2 page extraction on admitted papers (optionally filtered by `split_filter`,
/// e.g. `Some("development")` for the 4 development PDFs).
///
/// Failed admission records or PDF parse errors are recorded as explicit `PageParseStatus::Failed`
/// records in `pages.jsonl` and in `ParseRunReport` — never silently dropped.
pub fn parse_admitted_corpus(
    admission: &AdmissionReport,
    split_filter: Option<&str>,
    tool: &SystemPdfTool,
) -> Result<(Vec<PageRecord>, ParseRunReport)> {
    admission
        .validate()
        .context("validate admission report before stage 2 parse")?;

    let selected: Vec<&AdmissionRecord> = admission
        .records
        .iter()
        .filter(|r| match split_filter {
            Some(split) => r.split == split,
            None => true,
        })
        .collect();

    let mut page_records = Vec::new();
    let mut paper_summaries = Vec::with_capacity(selected.len());
    let mut succeeded_papers = 0usize;
    let mut failed_papers = 0usize;

    for record in selected {
        if record.status != AdmissionStatus::Admitted {
            failed_papers += 1;
            let err_msg = format!(
                "stage 1 admission failed ({:?}): {}",
                record.status,
                record.error.as_deref().unwrap_or("unknown admission failure")
            );
            let fail_rec = PageRecord {
                schema_version: PAGE_SCHEMA_VERSION.to_string(),
                paper_id: record.item_id,
                attachment_id: record.attachment_id.unwrap_or(-1),
                attachment_sha256: record
                    .sha256
                    .clone()
                    .filter(|d| is_valid_sha256_hex(d))
                    .unwrap_or_else(|| "0".repeat(64)),
                page: 0,
                text: String::new(),
                order_metadata: PageOrderMetadata {
                    page_index_zero_based: 0,
                    total_pages_in_document: 0,
                    reading_order_mode: tool.reading_order_mode.clone(),
                    char_count: 0,
                    line_count: 0,
                    replacement_char_count: 0,
                },
                parser_version: tool.parser_version.clone(),
                status: PageParseStatus::Failed,
                error: Some(err_msg.clone()),
            };
            fail_rec.validate()?;
            page_records.push(fail_rec);
            paper_summaries.push(PaperParseSummary {
                paper_id: record.item_id,
                attachment_id: record.attachment_id,
                split: record.split.clone(),
                parsed_ok: false,
                total_pages: 0,
                empty_pages: 0,
                total_chars: 0,
                error: Some(err_msg),
            });
            continue;
        }

        let attachment_id = record.attachment_id.expect("validated admitted attachment_id");
        let sha256 = record.sha256.clone().expect("validated admitted sha256");
        let pdf_path = PathBuf::from(record.path.as_ref().expect("validated admitted path"));

        match extract_pdf_pages_with_tool(&pdf_path, tool) {
            Ok(pages) => {
                let total_pages = pages.len();
                let mut empty_pages = 0usize;
                let mut total_chars = 0usize;
                for (idx, (page_num, text)) in pages.into_iter().enumerate() {
                    let char_count = text.chars().count();
                    let line_count = if text.is_empty() { 0 } else { text.lines().count() };
                    let replacement_char_count =
                        text.chars().filter(|&c| c == '\u{FFFD}').count();
                    let is_empty = text.trim().is_empty();
                    if is_empty {
                        empty_pages += 1;
                    }
                    total_chars += char_count;

                    let (status, error) = if is_empty {
                        (
                            PageParseStatus::EmptyPage,
                            Some(format!(
                                "page {page_num} of paper {} extracted 0 non-whitespace characters",
                                record.item_id
                            )),
                        )
                    } else {
                        (PageParseStatus::Ok, None)
                    };

                    let page_rec = PageRecord {
                        schema_version: PAGE_SCHEMA_VERSION.to_string(),
                        paper_id: record.item_id,
                        attachment_id,
                        attachment_sha256: sha256.clone(),
                        page: page_num,
                        text,
                        order_metadata: PageOrderMetadata {
                            page_index_zero_based: idx,
                            total_pages_in_document: total_pages,
                            reading_order_mode: tool.reading_order_mode.clone(),
                            char_count,
                            line_count,
                            replacement_char_count,
                        },
                        parser_version: tool.parser_version.clone(),
                        status,
                        error,
                    };
                    page_rec.validate()?;
                    page_records.push(page_rec);
                }
                succeeded_papers += 1;
                paper_summaries.push(PaperParseSummary {
                    paper_id: record.item_id,
                    attachment_id: Some(attachment_id),
                    split: record.split.clone(),
                    parsed_ok: true,
                    total_pages,
                    empty_pages,
                    total_chars,
                    error: None,
                });
            }
            Err(err) => {
                failed_papers += 1;
                let err_msg = format!("{err:#}");
                let fail_rec = PageRecord {
                    schema_version: PAGE_SCHEMA_VERSION.to_string(),
                    paper_id: record.item_id,
                    attachment_id,
                    attachment_sha256: sha256,
                    page: 0,
                    text: String::new(),
                    order_metadata: PageOrderMetadata {
                        page_index_zero_based: 0,
                        total_pages_in_document: 0,
                        reading_order_mode: tool.reading_order_mode.clone(),
                        char_count: 0,
                        line_count: 0,
                        replacement_char_count: 0,
                    },
                    parser_version: tool.parser_version.clone(),
                    status: PageParseStatus::Failed,
                    error: Some(err_msg.clone()),
                };
                fail_rec.validate()?;
                page_records.push(fail_rec);
                paper_summaries.push(PaperParseSummary {
                    paper_id: record.item_id,
                    attachment_id: Some(attachment_id),
                    split: record.split.clone(),
                    parsed_ok: false,
                    total_pages: 0,
                    empty_pages: 0,
                    total_chars: 0,
                    error: Some(err_msg),
                });
            }
        }
    }

    let report = ParseRunReport {
        schema_version: PAGE_SCHEMA_VERSION.to_string(),
        parser_tool: tool.clone(),
        split_filter: split_filter.map(str::to_string),
        target_papers: paper_summaries.len(),
        succeeded_papers,
        failed_papers,
        total_page_records: page_records.len(),
        papers: paper_summaries,
    };

    Ok((page_records, report))
}

pub fn write_pages_jsonl(path: &Path, pages: &[PageRecord]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create directory {}", parent.display()))?;
        }
    }
    let mut lines = String::new();
    for (idx, record) in pages.iter().enumerate() {
        record
            .validate()
            .with_context(|| format!("validate PageRecord at index {idx} before write"))?;
        let line = serde_json::to_string(record)
            .with_context(|| format!("serialize PageRecord at index {idx}"))?;
        let roundtrip: PageRecord = serde_json::from_str(&line)
            .with_context(|| format!("roundtrip validate PageRecord JSON at index {idx}"))?;
        roundtrip.validate()?;
        lines.push_str(&line);
        lines.push('\n');
    }
    fs::write(path, lines).with_context(|| format!("write pages jsonl {}", path.display()))
}

pub fn read_pages_jsonl(path: &Path) -> Result<Vec<PageRecord>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for (line_idx, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("read line {} of {}", line_idx + 1, path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let rec: PageRecord = serde_json::from_str(&line).with_context(|| {
            format!("deserialize PageRecord at {}:{}", path.display(), line_idx + 1)
        })?;
        rec.validate().with_context(|| {
            format!("validate PageRecord at {}:{}", path.display(), line_idx + 1)
        })?;
        records.push(rec);
    }
    Ok(records)
}
