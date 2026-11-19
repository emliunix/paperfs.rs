#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
CRATE_ROOT="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(CDPATH= cd -- "$CRATE_ROOT/.." && pwd)"

node "$SCRIPT_DIR/docs_lint.js" "$REPO_ROOT" "$CRATE_ROOT"
