# 5 — Make the patched opendal source fetchable

## Facts

- The root `Cargo.toml` patched `opendal` to `./opendal/core`, an untracked
  local clone (271 MB, 2068 source files) of the public fork
  `emliunix/opendal` at commit `fbfb7658a`.
- `opendal/` is not tracked and was never pushed to `origin/main`. A clean
  checkout — GitHub Actions, Docker, Nix, or a new clone — therefore cannot
  resolve the patch path and fails before compiling.
- The fork is public and the fix commit is reachable through the GitHub API.

## Change

- Replace the local path patch with a pinned git source:

  ```toml
  [patch.crates-io]
  opendal = { git = "https://github.com/emliunix/opendal", rev = "fbfb7658a0c34d1aee5acf8f7bf27124cc729fa3" }
  ```

- Add `/opendal/` to `.gitignore`, `.dockerignore`, and the Nix source
  filter. The local clone stays available for development but is no longer a
  build input.
- Regenerate `Cargo.lock` and run `cargo check --workspace`.

## Trade-off

Nix builds now fetch a git dependency, which crane normally pins with
`cargoLock.outputHashes`. Nix is not installed in this environment, so that
hash could not be generated; the flake was already unable to build from a
clean checkout because the patch path was untracked. The PR documents this
rather than inventing a hash. CI and Docker use Cargo directly and are fixed.
