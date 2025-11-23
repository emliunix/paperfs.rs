# paperfs.rs

1. paper because I use it with zotero
2. fs because it's a webdav server to work with zotero webdav sync
3. rs because it's written in Rust

And I use it to connect to my personal onedrive.

Still working towards MVP.

## TODO

* [ ] rename odrive.rs to msauth.rs,
* [ ] on access token refresh, only reconstruct opendal operator. Current approach may corrupt the WebDAV fslock semantic.
* [ ] add workflow and automated deployment
* [ ] UI for login, though curl works

## GPT Review (2025-11-23T06:42:16.837Z)
High-level
- Purpose: WebDAV server (Zotero sync) backed by OneDrive with dynamic init post OAuth.
- Architecture: Axum + OpenDAL (buffer + mux + logging layers) + custom DavHandler wrapper + token refresh thread + callback hooks.

Strengths
- Clear modular separation (auth, layers, DAV wrapper, init gating).
- PKCE + state tracking; graceful shutdown integrated with refresh thread.
- Git revision embedded for observability.
- Flexible layering (Mux to route special files to memory, Buf aggregation, Logging).

Key Issues / Risks
1. Security: Plain-text refresh/id tokens in app_data.json; verbose logs may leak tokens (id_token, OAuth payload). No permission hardening.
2. Error handling: Frequent unwrap/expect causing potential panics; silent suppression via log_and_go hides persistent failures.
3. Memory usage: DavHandlerWrapper fully buffers request bodies; BufLayer buffers entire file before write -> large uploads can exhaust RAM.
4. Refresh logic: Immediate refresh on start even if valid; tight loop if expiry near; no backoff/jitter on failures.
5. Mux semantics: list concatenates without dedup; delete decision based on empty path; create_dir always delegated to OneDrive; inconsistent behavior.
6. DAV MKCOL patch: URI rebuild may mishandle query components.
7. Logging hygiene: Excessive info/debug (per frame, per list entry); mix of log/tracing; noisy in production.
8. API design: POST /login redirect (GET more conventional); inconsistent response formats; no health endpoint.
9. Resilience: No retry strategy or exponential backoff; assumes writable app_data.json.
10. Observability: Lacks metrics; tracing only behind feature flag; missing structured fields.
11. Tests: None for auth, token refresh, DAV routing, mux logic.
12. Code style: is_fn helper unclear; AsyncHook could be simplified; planned file rename pending.
13. Potential bugs: delete() path logic; MKCOL path_and_query formatting; BufLayer delete signature unusual; ConcatList very chatty logs.

Recommendations (priority)
1. Remove sensitive token/id logging; secure token storage (permissions, env-configurable path, consider encryption).
2. Replace unwrap/expect with error handling (anyhow + context); propagate errors.
3. Stream request bodies instead of full buffering; reconsider/remodel BufLayer with size limits or remove if not required.
4. Refine refresh scheduling (only when within threshold, add jitter/backoff).
5. Fix MuxLayer semantics (path-based decisions for delete/create_dir/stat/list; dedup collisions).
6. Correct MKCOL URI patch preserving query safely.
7. Reduce log verbosity; use tracing spans/fields; reserve trace for high-frequency events.
8. Normalize API (GET /login, JSON envelopes, health/status endpoint).
9. Add minimal tests (auth flow, token refresh timing, MKCOL patch, mux filtering).
10. Introduce CI workflow (fmt, clippy, test, build).
11. Config improvements (env var for token file path; clarify ONEDRIVE_ROOT semantics).
12. Cleanup (rename odrive.rs; simplify AsyncHook with Arc<dyn Fn> pattern).
13. Metrics (Prometheus exporter) and user-facing login status page.

Minor improvements
- Use SystemTime/Duration for expiry; rename expires_in -> expires_at for clarity.
- Remove unused fields (access_token in OneDriveArgs); unify naming.
- Add clippy lints (deny unwrap_used) early.

Overall
Solid prototype with clear direction; prioritize security (token/logging) and memory safety (streaming) before production. Strengthen error handling and add basic tests + CI.
