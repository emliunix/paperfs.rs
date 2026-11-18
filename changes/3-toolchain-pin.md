# 3 — Toolchain pin fallback

The review asked for an exact `1.100.0` pin in `rust-toolchain.toml`.

Attempting that pin failed in this environment:

```text
error: could not download nonexistent rust version `1.100.0-aarch64-apple-darwin`
       ... 404 from mirrors.tuna.tsinghua.edu.cn/rustup ...
```

The configured rustup mirror does not carry the exact artifact, so an exact
pin would break every build on this machine. The workspace instead:

- tracks `channel = "stable"` (currently rustc 1.100.0, above hawdb's 1.97.1
  MSRV), with the MSRV recorded as a comment in `rust-toolchain.toml`;
- sets `rust-version = "1.97.1"` in both the root package and
  `paper-graph/Cargo.toml`;
- builds `Dockerfile` from `rust:1.100-trixie` (tag verified on Docker Hub);
- runs `cargo build/test --workspace` in CI, so the member is not skipped.

This is a deliberate deviation from the reviewed v2 text, recorded here
because change plans are append-only.
