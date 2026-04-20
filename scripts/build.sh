#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
#
# Unified build entrypoint for this repository.
#
# Subcommands delegate to scripts/build-rpm.sh, scripts/build-docker.sh, and
# scripts/generate-api-docs.sh (the `docs` entry may grow to include CLI or other doc bundles later).
# Optional auto-install of toolchain deps is handled
# inside those scripts (disabled when CI=true or DISABLE_AUTO_INSTALL_DEPS=1).
#
# `help` / `rpm` / `docker` / `docs` do not load scripts/lib/build-deps.sh in this process
# (delegated scripts load what they need; release builds source it below).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

usage() {
    cat <<'EOF'
Usage: scripts/build.sh [<command>] [arguments...]

Commands:
  help, -h, --help     Show this help.

  (default)            Same as "release".
  release              Run `cargo build --release` for the workspace.
                       Any extra arguments are passed to cargo (e.g. `--bin rbs`).

  debug                Run `cargo build` (dev profile, output under target/debug/).
                       Use for local iteration; use release for packaging or production-like runs.

  rpm                  Build RPM packages (see scripts/build-rpm.sh).
  docker               Build the RBS Docker image (see scripts/build-docker.sh).
  docs                 Generated documentation (REST API / OpenAPI + MD/HTML today via
                       scripts/generate-api-docs.sh). CLI or other bundles may be added to this entry later.

Environment (examples):
  VERSION, RELEASE     Used by `rpm` and `docker` wrappers (see those scripts).
  REGISTRY             Docker image registry prefix (docker command).
  CONTAINER_ENGINE     docker or podman for image build (see scripts/build-docker.sh).
  SOURCE_URL           Optional OCI image source label (see scripts/build-docker.sh).
  RPM_BUILD_DIR        Optional absolute path for rpmbuild topdir (default: <repo>/rpm-build).

Examples:
  scripts/build.sh
  scripts/build.sh release --bin rbs
  scripts/build.sh debug --bin rbs
  scripts/build.sh --bin rbs
  scripts/build.sh rpm
  VERSION=1.0.0 RELEASE=2 scripts/build.sh rpm
  RPM_BUILD_DIR=/tmp/rbs-rpm scripts/build.sh rpm
  scripts/build.sh docker
  SKIP_LICENSE_CHECK=1 scripts/build.sh docs
EOF
}

# Fast path: no need to source lib/build-deps.sh for help or delegated commands.
if [[ $# -gt 0 ]]; then
    case "$1" in
        help | -h | --help)
            usage
            exit 0
            ;;
        rpm)
            shift
            exec "$SCRIPT_DIR/build-rpm.sh" "$@"
            ;;
        docker)
            shift
            exec "$SCRIPT_DIR/build-docker.sh" "$@"
            ;;
        docs)
            shift
            exec "$SCRIPT_DIR/generate-api-docs.sh" "$@"
            ;;
        api-docs)
            # Back-compat alias for `docs` (not listed in help; use `docs`).
            shift
            exec "$SCRIPT_DIR/generate-api-docs.sh" "$@"
            ;;
        debug)
            shift
            # shellcheck source=lib/build-deps.sh
            source "$SCRIPT_DIR/lib/build-deps.sh"
            cd "$PROJECT_ROOT"
            echo "Building globaltrustauthority-rbs project (dev profile)..."
            ensure_cargo
            cargo build "$@"
            echo "Build completed successfully!"
            echo "Binaries are located in: $PROJECT_ROOT/target/debug/"
            exit 0
            ;;
    esac
fi

# shellcheck source=lib/build-deps.sh
source "$SCRIPT_DIR/lib/build-deps.sh"

run_release() {
    cd "$PROJECT_ROOT"
    echo "Building globaltrustauthority-rbs project (release)..."
    ensure_cargo
    cargo build --release "$@"
    echo "Build completed successfully!"
    echo "Binaries are located in: $PROJECT_ROOT/target/release/"
}

if [[ $# -eq 0 ]]; then
    run_release
elif [[ "$1" == "release" ]]; then
    shift
    run_release "$@"
else
    run_release "$@"
fi
