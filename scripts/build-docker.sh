#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
#
# Docker image build script (only builds RBS service).
#
# Optional auto-install: docker CLI (e.g. docker.io on Debian/Ubuntu, docker on dnf/yum) when missing.
# Disabled when CI=true or DISABLE_AUTO_INSTALL_DEPS=1.
# The Docker daemon must be running; you may need the docker group for rootless access.
# When engine is docker, builds use BuildKit via `docker buildx build --load` (not legacy `docker build`).
#
# Prefer: scripts/build.sh docker

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

usage() {
    cat <<'EOF'
Usage: scripts/build-docker.sh
   or: scripts/build.sh docker

Builds the RBS image from deployment/docker/dockerfile.

With Docker: requires Buildx (`docker buildx version`). With Podman: uses `podman build`.

Environment:
  VERSION              Image tag (default: latest)
  REGISTRY             Image prefix (default: globaltrustauthority-rbs), image: REGISTRY/rbs:VERSION
  CONTAINER_ENGINE     CLI to use: docker or podman (default: docker if on PATH, else podman)
  SOURCE_URL           Optional OCI label org.opencontainers.image.source (e.g. repository URL)

Example:
  VERSION=1.2.3 REGISTRY=myrepo scripts/build-docker.sh
  CONTAINER_ENGINE=podman scripts/build-docker.sh
EOF
}

if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

# shellcheck source=lib/build-deps.sh
source "$SCRIPT_DIR/lib/build-deps.sh"

cd "$PROJECT_ROOT"

VERSION=${VERSION:-latest}
REGISTRY=${REGISTRY:-globaltrustauthority-rbs}

# Prefer explicit engine; else docker, then podman; else try to install docker via build-deps.
CE="${CONTAINER_ENGINE:-}"
if [[ -z "$CE" ]]; then
    if command -v docker >/dev/null 2>&1; then
        CE=docker
    elif command -v podman >/dev/null 2>&1; then
        CE=podman
    else
        ensure_docker_cli
        CE=docker
    fi
elif ! command -v "$CE" >/dev/null 2>&1; then
    echo "error: CONTAINER_ENGINE=$CE is not on PATH." >&2
    exit 1
fi

echo "Building RBS OCI image for version $VERSION (engine=$CE)..."

label_args=()
if [[ -n "${SOURCE_URL:-}" ]]; then
    label_args+=(--label "org.opencontainers.image.source=${SOURCE_URL}")
fi
label_args+=(--label "org.opencontainers.image.version=${VERSION}")
# Best-effort: repo URL for provenance (override with SOURCE_URL).
if [[ -z "${SOURCE_URL:-}" ]] && command -v git >/dev/null 2>&1; then
    _origin="$(git -C "$PROJECT_ROOT" config --get remote.origin.url 2>/dev/null || true)"
    if [[ -n "$_origin" ]]; then
        label_args+=(--label "org.opencontainers.image.source=${_origin}")
    fi
fi
label_args+=(--label "org.opencontainers.image.title=rbs")
label_args+=(--label "org.opencontainers.image.description=Global Trust Authority Resource Broker Service")

# Only build RBS image (RBC and tools do not need containerization)
if [[ "$CE" == "docker" ]]; then
    if ! docker buildx version >/dev/null 2>&1; then
        echo "error: docker buildx is required (legacy \`docker build\` is not used). Install Buildx: https://docs.docker.com/build/buildx/install/" >&2
        exit 1
    fi
    docker buildx build --load -f deployment/docker/dockerfile "${label_args[@]}" -t "$REGISTRY/rbs:$VERSION" .
else
    "$CE" build -f deployment/docker/dockerfile "${label_args[@]}" -t "$REGISTRY/rbs:$VERSION" .
fi

echo "OCI image built successfully!"
echo "Image: $REGISTRY/rbs:$VERSION"
echo ""
echo "Note: RBC and tools are not containerized, use RPM deployment instead."
