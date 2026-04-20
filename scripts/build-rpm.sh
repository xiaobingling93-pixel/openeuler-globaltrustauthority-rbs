#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
#
# RPM build script.
#
# Optional auto-install: cargo, rpmbuild, gcc/g++, make (apt or dnf/yum) when missing.
# Disabled when CI=true or DISABLE_AUTO_INSTALL_DEPS=1.
#
# Prefer: scripts/build.sh rpm

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

usage() {
    cat <<'EOF'
Usage: scripts/build-rpm.sh
   or: scripts/build.sh rpm

Builds RBS / RBC / RBS-CLI RPM packages from the workspace.

Environment:
  VERSION        Package version (default: 0.1.0)
  RELEASE        RPM release (default: 1)
  RPM_BUILD_DIR  rpmbuild topdir (default: <repo>/rpm-build). Use an absolute path in CI or
                 when you must avoid writing under the repository tree.

Example:
  VERSION=1.0.0 RELEASE=2 scripts/build-rpm.sh
  RPM_BUILD_DIR=/tmp/rbs-rpmbuild scripts/build-rpm.sh
EOF
}

if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

# shellcheck source=lib/build-deps.sh
source "$SCRIPT_DIR/lib/build-deps.sh"

cd "$PROJECT_ROOT"

ensure_rpmbuild_stack

VERSION=${VERSION:-0.1.0}
RELEASE=${RELEASE:-1}

echo "Building RPM packages for version $VERSION-$RELEASE..."

# rpmbuild topdir: default under repo; override with RPM_BUILD_DIR (absolute or repo-relative).
if [[ -n "${RPM_BUILD_DIR:-}" ]]; then
    if [[ "${RPM_BUILD_DIR}" == /* ]]; then
        BUILD_DIR="$RPM_BUILD_DIR"
    else
        BUILD_DIR="$PROJECT_ROOT/$RPM_BUILD_DIR"
    fi
else
    BUILD_DIR="$PROJECT_ROOT/rpm-build"
fi

# Safety: refuse obviously unsafe rpmbuild topdirs (this script wipes BUILD_DIR before use).
case "$BUILD_DIR" in
    / | /bin | /boot | /dev | /etc | /lib | /lib64 | /proc | /sys | /usr | /var | "$HOME" | "$HOME"/)
        echo "error: refusing RPM_BUILD_DIR/BUILD_DIR=$BUILD_DIR (too dangerous to clean and reuse)." >&2
        exit 1
        ;;
esac

# Create build directory
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Build Rust project
echo "Building Rust binaries..."
cargo build --release

# Build RBS RPM
echo "Building RBS RPM..."
cd "$PROJECT_ROOT"
rpmbuild -bb rpm/rbs.spec \
    --define "_topdir $BUILD_DIR" \
    --define "_project_root $PROJECT_ROOT" \
    --define "version $VERSION" \
    --define "release $RELEASE" \
    --buildroot "$BUILD_DIR/BUILDROOT"

# Build RBC RPM
echo "Building RBC RPM..."
rpmbuild -bb rpm/rbc.spec \
    --define "_topdir $BUILD_DIR" \
    --define "_project_root $PROJECT_ROOT" \
    --define "version $VERSION" \
    --define "release $RELEASE" \
    --buildroot "$BUILD_DIR/BUILDROOT"

# Build RBS-CLI RPM
echo "Building RBS-CLI RPM..."
rpmbuild -bb rpm/rbs-cli.spec \
    --define "_topdir $BUILD_DIR" \
    --define "_project_root $PROJECT_ROOT" \
    --define "version $VERSION" \
    --define "release $RELEASE" \
    --buildroot "$BUILD_DIR/BUILDROOT"

echo "RPM packages built successfully!"
echo "RPM files are located in: $BUILD_DIR/RPMS/$(rpm --eval %_arch)/"
