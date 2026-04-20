#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
#
# Smoke tests for repository shell scripts: syntax check and --help exits 0.
# Run from repository root: ./tests/scripts_smoke.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SCRIPT_FILES=(
    scripts/build.sh
    scripts/build-rpm.sh
    scripts/build-docker.sh
    scripts/generate-api-docs.sh
    scripts/lib/os-pkg.sh
    scripts/lib/build-deps.sh
)

for f in "${SCRIPT_FILES[@]}"; do
    bash -n "$f"
done

./scripts/build.sh help >/dev/null
./scripts/build-rpm.sh --help >/dev/null
./scripts/build-docker.sh --help >/dev/null
./scripts/generate-api-docs.sh help >/dev/null

echo "scripts_smoke: OK (${#SCRIPT_FILES[@]} files, bash -n + help)"
