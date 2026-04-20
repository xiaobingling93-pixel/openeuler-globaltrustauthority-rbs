#!/usr/bin/env bash
# SPDX-License-Identifier: MulanPSL-2.0
#
# Shared helpers for detecting distro package family and optional non-interactive installs.
# Intended to be sourced from scripts under scripts/*.sh (not executed directly).

# Caller must check exit status (or rely on set -e) — failure means no root/sudo.
run_priv() {
    if [[ "$(id -u)" -eq 0 ]]; then
        "$@"
    elif command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    else
        echo "error: need root or sudo to install OS packages." >&2
        return 1
    fi
}

# Returns: apt | dnf | none
detect_pkg_family() {
    if [[ ! -f /etc/os-release ]]; then
        echo none
        return
    fi
    # shellcheck source=/dev/null
    . /etc/os-release
    local id_lc like_lc
    id_lc="$(echo "${ID:-unknown}" | tr '[:upper:]' '[:lower:]')"
    like_lc="$(echo "${ID_LIKE:-}" | tr '[:upper:]' '[:lower:]')"

    case "$id_lc" in
        ubuntu | debian | linuxmint | pop | elementary | zorin) echo apt ;;
        raspbian) echo apt ;;
        openeuler) echo dnf ;;
        *)
            if [[ "$like_lc" == *debian* ]] || [[ "$like_lc" == *ubuntu* ]]; then
                echo apt
            elif [[ "$like_lc" == *rhel* ]] || [[ "$like_lc" == *fedora* ]] ||
                [[ "$like_lc" == *centos* ]] || [[ "$like_lc" == *openeuler* ]]; then
                echo dnf
            else
                echo none
            fi
            ;;
    esac
}

# True when automatic OS package install must not run (CI or explicit opt-out).
build_deps_auto_install_disabled() {
    [[ "${CI:-}" == "true" ]] || [[ "${DISABLE_AUTO_INSTALL_DEPS:-}" == "1" ]]
}
