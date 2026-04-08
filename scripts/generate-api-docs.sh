#!/usr/bin/env bash
# Generate API documentation: OpenAPI YAML (Cargo), then Markdown and HTML (npm).
#
# In CI (CI=true) the script also verifies that the committed docs/api/rbs/ tree is
# up-to-date and exits non-zero when there is a drift.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo build -p rbs --features rest

# OpenAPI YAML is emitted by rbs/build.rs; ensure the checked-in file matches the build.
if ! git diff --quiet -- docs/proto/rbs_rest_api.yaml; then
    if [[ "${CI:-}" == "true" ]]; then
        echo "error: docs/proto/rbs_rest_api.yaml differs from build output; rebuild and commit." >&2
        exit 1
    else
        echo "notice: docs/proto/rbs_rest_api.yaml updated; commit it before regenerating companion docs." >&2
    fi
fi

if ! command -v npm >/dev/null 2>&1; then
    echo "error: npm is required to generate docs/api/rbs/md/rbs_rest_api.md and docs/api/rbs/html/rbs_rest_api.html (install Node.js or use nvm)." >&2
    exit 1
fi

mkdir -p docs/api/rbs/md docs/api/rbs/html

NPM_DIR="$ROOT/scripts/conf/openapi-docs"
if [[ "${CI:-}" == "true" ]] || [[ ! -d "$NPM_DIR/node_modules" ]]; then
    npm ci --prefix "$NPM_DIR"
fi

npm --prefix "$NPM_DIR" run api:docs

# Drift detection: warn locally; fail in CI so stale docs are caught before merge.
if ! git diff --quiet -- docs/api/rbs/; then
    if [[ "${CI:-}" == "true" ]]; then
        echo "error: docs/api/rbs/ differs from committed files; regenerate and commit the updated docs." >&2
        exit 1
    else
        echo "notice: docs/api/rbs/ updated; review and commit the changes." >&2
    fi
fi
