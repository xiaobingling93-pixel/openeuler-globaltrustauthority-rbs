# globaltrustauthority-rbs

**Resource Broker Service (RBS)** — distributes keys, certificates, and related resources using remote attestation with the Global Trust Authority.

## Overview

This repository is a **Rust workspace** that implements policy-driven resource brokering for attested workloads. It ships:

- **`rbs`** — HTTP service; REST APIs under **`/rbs`** (for example **`GET /rbs/version`**).
- **`rbc`** — Client library for attestation and resource flows.
- **`rbs-cli`** — Command-line interface for administration and operations against a running broker.

Runtime behaviour is configured with **YAML** (`rest`, TLS, storage, and optional features such as per-IP rate limiting when enabled at build time). The **OpenAPI** contract and rendered API documentation are checked in under **`docs/proto/`** and **`docs/api/`**; regenerate them with **`./scripts/build.sh docs`** (paths and tooling are listed under **Documentation** below).

**RPM** packages (with **systemd** and `rbs.service`), **Docker / Compose** files under **`deployment/docker/`**, and the **`./scripts/build.sh`** entry point cover release builds, container images, and generated docs.

For **build, install, RPM, container, and test** procedures, see [**docs/build/build_and_install.md**](docs/build/build_and_install.md). Its quick start matches **Quick start** in this file.

## Prerequisites

- **Rust** — a recent **stable** toolchain ([rustup](https://rustup.rs/) recommended).
- **OS** — **Linux** is the primary target; RPM-based packaging and paths are documented for openEuler-style systems.
- **Node.js** (required for REST API documentation generation via **`./scripts/build.sh docs`** or **`./scripts/generate-api-docs.sh`**) — **≥ 22.12**; **[`.nvmrc`](.nvmrc)** specifies **24**. Use [nvm](https://github.com/nvm-sh/nvm) or [fnm](https://github.com/Schniz/fnm) at the repository root (`nvm install` / `nvm use`, or `fnm use`) so the active Node.js version matches **`.nvmrc`**, rather than a system-wide default that may be older.

## Quick start

Execute the steps below in order from a POSIX shell. From step 2 onward, run commands from the **repository root** (the directory that contains `Cargo.toml`). This project does not provide **`make install`**; release binaries are produced under **`target/release/`**. Host installation using RPM packages is described in **[docs/build/rpm.md](docs/build/rpm.md)**.

```bash
# 1) Install build and runtime dependencies (choose one distribution block)
# openEuler / Fedora / RHEL
sudo dnf install -y git cargo rust rpm-build rpmdevtools gcc gcc-c++ make docker nodejs npm \
  && sudo systemctl enable --now docker
# Debian / Ubuntu
sudo apt-get update && sudo apt-get install -y git build-essential pkg-config libssl-dev \
  rpm docker.io docker-compose-v2 nodejs npm && sudo systemctl enable --now docker
# For ./scripts/build.sh docs: distro nodejs may be < 22.12 — check `node -v`; use nvm/fnm + .nvmrc if needed (see Prerequisites).
# Rust too old for Cargo.lock? Use rustup instead of the distro Rust:
# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && source "$HOME/.cargo/env"

# 2) Obtain source code (choose one method)
git clone https://gitcode.com/openeuler/globaltrustauthority-rbs.git
cd globaltrustauthority-rbs
# Alternative: download a source tarball or zip from a release or mirror, unpack, and enter the tree:
# tar xf globaltrustauthority-rbs-*.tar.gz && cd globaltrustauthority-rbs-*
# unzip globaltrustauthority-rbs-*.zip && cd globaltrustauthority-rbs-*

# 3) Build (release) — binaries land under target/release/
./scripts/build.sh

# 4) Run the test suite
./tests/test_all.sh

# 5) Start the RBS service (foreground; interrupt with Ctrl-C).
# The sample configuration listens on 127.0.0.1:6666 and uses paths under /var/log/rbs and /root;
# run with elevated privileges or adjust rbs/conf/rbs.yaml (logging.file_path, storage.url) first.
sudo ./target/release/rbs -c rbs/conf/rbs.yaml

# 6) From a second terminal — verify the running service
curl -sS http://127.0.0.1:6666/rbs/version                        # REST: version JSON
./target/release/rbs-cli -b http://127.0.0.1:6666 version         # CLI: version subcommand
./target/release/rbs-cli -b http://127.0.0.1:6666 --help          # CLI: subcommand list
```

**Additional references**

- **Build script** — Run `./scripts/build.sh help` from the repository root for subcommands (`rpm`, `docker`, `docs`, `debug`, and `cargo` passthrough).
- **Container workflow** — `./scripts/build.sh docker` or `docker compose -f deployment/docker/docker-compose.yml up --build` (maps to steps 3 and 5 above for image + runtime). Port mapping and config: [Container: build, run, and test](docs/build/build_and_install.md#5-container-build-run-and-test-step-by-step).
- **Tests** — Layout, skips, and e2e driver: [tests/README.md](tests/README.md).
- **Tooling and docs** — E2e scripts, OpenAPI artefacts, Compose paths, optional `cargo deny`: [Further reading and tooling](docs/build/build_and_install.md#7-further-reading-and-tooling).
- **CI / no `sudo` installs** — Set **`CI=true`** or **`DISABLE_AUTO_INSTALL_DEPS=1`** so helper scripts do not install OS packages. See [RPM flow in build_and_install.md](docs/build/build_and_install.md#4-rpm-build-install-run-and-test-step-by-step) and the [container](docs/build/build_and_install.md#5-container-build-run-and-test-step-by-step) section of the same file.

## Documentation

| Topic | Location |
|--------|----------|
| End-to-end build, run, and test (source, RPM, container, generated docs) | [**docs/build/build_and_install.md**](docs/build/build_and_install.md) |
| RPM install, upgrade, systemd, packaging details | [**docs/build/rpm.md**](docs/build/rpm.md) |
| Tests (Cargo, e2e, script smoke) | [**tests/README.md**](tests/README.md) |
| Checked-in REST OpenAPI and rendered API (regenerate via `./scripts/build.sh docs`) | [`docs/proto/rbs_rest_api.yaml`](docs/proto/rbs_rest_api.yaml), [`docs/api/rbs/md/rbs_rest_api.md`](docs/api/rbs/md/rbs_rest_api.md) |
| Compose / image files | [`deployment/docker/`](deployment/docker/) |
| Optional: workspace clippy / `cargo-deny` | [`.cargo/config.toml`](.cargo/config.toml), [`deny.toml`](deny.toml) |

## License

Licensed under the **Mulan Public License, version 2** — see [LICENSE](LICENSE).
