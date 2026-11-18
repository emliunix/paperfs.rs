# 4 — Finalization and validation evidence

Completes `changes/2-migrate-paper-graph-v2.md` and
`changes/3-toolchain-pin.md`.

## Moves and preservation

- Backup: `/Users/emliunix/Documents/paper-graph-backup-2026-11-18`
  (278 files; 79 MB including `data/`; sha256 of analysis, experiments and
  `ui.html` matched the source at backup time).
- Moved: `../paper-graph/data` -> `paper-graph/data` (232 files, 79 MB).
- Re-synced the newest `analysis/` (3 files, including
  `sutra-os-research.md`) and verified `analysis`, `experiments` and
  `ui.html` are byte-identical to the source before deletion.
- Deleted the Python tree only after the backup and after all above checks.
- Legacy path `/Users/emliunix/Documents/paper-graph` is now a symlink to
  `paperfs.rs/paper-graph`; `cd` there still resolves the workspace and
  `cargo run -p paper-graph -- stats` works.

## Validation performed

- `cargo build --workspace`: success (pre-existing root warnings only).
- `cargo test --workspace`: paper-graph 9 passed, 1 ignored opt-in; root test
  binary builds; doc tests pass.
- `cargo clippy -p paper-graph --all-targets -- -D warnings`: clean.
- `cargo fmt -p paper-graph`: applied.
- Opt-in real-DB test against the pre-move copy: passed with
  324 works / 171 unfiled / 347 persons / 5 collections / 24 tags /
  10 venues / 161 attachments.
- Default path on the moved database: `stats` returns the same counts from
  both the workspace root and the legacy symlink path.
- `probe` against real Zotero + llama.cpp: 324 works, 161 attachments,
  1024 dimensions, `bge-m3`, 10 tokens.
- Limited `sync --limit 4 --batch-size 2` into a temp database: success;
  report included all stats and 479 tokens.
- All search modes (BM25, vector, hybrid) and author lookup on the temp DB.
- UI smoke test: index HTML, stats, BM25/hybrid search, work detail, 404 work,
  history JSONL all succeeded from the axum server.
- HTTP parity: missing `q`, invalid `top_k`, invalid `id`, unknown mode and
  unknown path all return 500/404 JSON with
  `Cache-Control: no-store` and `Content-Type: application/json; charset=utf-8`.

## Configuration changes

- `flake.nix`: source filter now keeps `.rs`, `.toml`, `.html` and
  `Cargo.lock` (needed by both crates' `include_str!`) while excluding
  `paper-graph/data`, `test_onedrive` and `target`.
- `.dockerignore`: excludes `test_onedrive`, `paper-graph/data`,
  `paper-graph/.venv`, `.git`.
- `Dockerfile`: `rust:1.100-trixie` (Docker Hub tag verified).
- CI: `cargo build --workspace` and `cargo test --workspace`.
- `Cargo.toml`: workspace member plus `rust-version = "1.97.1"`;
  `paper-graph/Cargo.toml`: `publish = false`, `rust-version = "1.97.1"`.

## Not executed locally

- Nix and Docker are not installed in this environment, so `nix build` and
  `docker build` were not run. The Nix source filter was reviewed against
  crane's `filterCargoSources` and the two `include_str!` call sites; the
  Docker base tag was checked through the Docker Hub API.
- A full sync against the real moved database was intentionally not run; the
  limited sync into a temp database and the real-DB count test cover the
  rebuild path without replacing the user's production data.
