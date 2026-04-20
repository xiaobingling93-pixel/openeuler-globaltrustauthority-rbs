#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
#
# Build-related dependency detection and optional distro package installs.
# Source after setting SCRIPT_DIR to the scripts/ directory:
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   # shellcheck source=lib/build-deps.sh
#   source "$SCRIPT_DIR/lib/build-deps.sh"

_LIB_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/os-pkg.sh
source "$_LIB_ROOT/os-pkg.sh"

_ensure_cmds_after_install() {
    local missing=()
    local c
    for c in "$@"; do
        if ! command -v "$c" >/dev/null 2>&1; then
            missing+=("$c")
        fi
    done
    if [[ ${#missing[@]} -gt 0 ]]; then
        echo "error: after package install, still missing on PATH: ${missing[*]}" >&2
        return 1
    fi
    return 0
}

# Rust cargo (workspace build). Uses distro packages; for latest toolchains use rustup instead.
ensure_cargo() {
    if command -v cargo >/dev/null 2>&1; then
        return 0
    fi
    if build_deps_auto_install_disabled; then
        echo "error: cargo is not installed. Install Rust (https://rustup.rs) or set up distro packages." >&2
        exit 1
    fi

    local family
    family="$(detect_pkg_family)"
    echo "notice: cargo not found; attempting install (pkg_family=${family}) ..." >&2

    case "$family" in
        apt)
            if ! command -v apt-get >/dev/null 2>&1; then
                echo "error: apt-get not found; install cargo manually." >&2
                exit 1
            fi
            run_priv apt-get update -qq
            run_priv env DEBIAN_FRONTEND=noninteractive apt-get install -y cargo ||
                run_priv env DEBIAN_FRONTEND=noninteractive apt-get install -y rustc cargo
            ;;
        dnf)
            if command -v dnf >/dev/null 2>&1; then
                run_priv dnf install -y cargo rust || run_priv dnf install -y cargo
            elif command -v yum >/dev/null 2>&1; then
                run_priv yum install -y cargo rust || run_priv yum install -y cargo
            else
                echo "error: neither dnf nor yum found; install cargo manually." >&2
                exit 1
            fi
            ;;
        *)
            echo "error: unsupported distro for automatic cargo install (pkg_family=${family})." >&2
            echo "Install Rust via https://rustup.rs or your distribution packages." >&2
            exit 1
            ;;
    esac

    _ensure_cmds_after_install cargo || exit 1
    echo "notice: distro-installed cargo may be older than this workspace needs; if cargo build fails on Cargo.lock or edition, use https://rustup.rs/ and verify cargo --version." >&2
}

# RPM packaging: cargo, rpmbuild, and a C toolchain (matches rpm/*.spec %build using cargo).
ensure_rpmbuild_stack() {
    ensure_cargo

    local need=()
    local c
    for c in rpmbuild gcc g++ make; do
        command -v "$c" >/dev/null 2>&1 || need+=("$c")
    done
    [[ ${#need[@]} -eq 0 ]] && return 0

    if build_deps_auto_install_disabled; then
        echo "error: missing build tools: ${need[*]}. Install rpm-build / gcc / make for your OS." >&2
        exit 1
    fi

    local family
    family="$(detect_pkg_family)"
    echo "notice: installing RPM build dependencies (pkg_family=${family}) ..." >&2

    case "$family" in
        apt)
            if ! command -v apt-get >/dev/null 2>&1; then
                echo "error: apt-get not found." >&2
                exit 1
            fi
            run_priv apt-get update -qq
            # Debian/Ubuntu: 'rpm' provides rpmbuild; build-essential pulls gcc/g++/make.
            run_priv env DEBIAN_FRONTEND=noninteractive apt-get install -y rpm build-essential
            ;;
        dnf)
            if command -v dnf >/dev/null 2>&1; then
                run_priv dnf install -y rpm-build rpmdevtools gcc gcc-c++ make
            elif command -v yum >/dev/null 2>&1; then
                run_priv yum install -y rpm-build rpmdevtools gcc gcc-c++ make
            else
                echo "error: neither dnf nor yum found." >&2
                exit 1
            fi
            ;;
        *)
            echo "error: unsupported distro for automatic RPM tooling (pkg_family=${family})." >&2
            echo "On openEuler-style systems: dnf install -y rpm-build rpmdevtools gcc gcc-c++ make" >&2
            exit 1
            ;;
    esac

    _ensure_cmds_after_install rpmbuild gcc g++ make || exit 1
}

# Docker CLI for image builds (daemon must be running separately); build-docker.sh uses buildx when engine=docker.
ensure_docker_cli() {
    if command -v docker >/dev/null 2>&1; then
        return 0
    fi
    if build_deps_auto_install_disabled; then
        echo "error: docker is not installed. Install Docker Engine / CLI for your OS." >&2
        exit 1
    fi

    local family
    family="$(detect_pkg_family)"
    echo "notice: docker not found; attempting install (pkg_family=${family}) ..." >&2

    case "$family" in
        apt)
            if ! command -v apt-get >/dev/null 2>&1; then
                echo "error: apt-get not found; install Docker manually." >&2
                exit 1
            fi
            run_priv apt-get update -qq
            run_priv env DEBIAN_FRONTEND=noninteractive apt-get install -y docker.io
            ;;
        dnf)
            if command -v dnf >/dev/null 2>&1; then
                run_priv dnf install -y docker ||
                    run_priv dnf install -y moby-engine ||
                    run_priv dnf install -y podman-docker
            elif command -v yum >/dev/null 2>&1; then
                run_priv yum install -y docker ||
                    run_priv yum install -y podman-docker
            else
                echo "error: neither dnf nor yum found; install docker manually." >&2
                exit 1
            fi
            ;;
        *)
            echo "error: unsupported distro for automatic docker install (pkg_family=${family})." >&2
            exit 1
            ;;
    esac

    if ! command -v docker >/dev/null 2>&1; then
        echo "error: docker CLI still not on PATH after package install." >&2
        exit 1
    fi
    echo "notice: ensure the Docker daemon is running and your user can access it (e.g. docker group)." >&2
}
