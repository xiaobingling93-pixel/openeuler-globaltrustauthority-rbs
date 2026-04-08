/*
 * Copyright (c) Huawei Technologies Co., Ltd. 2026. All rights reserved.
 * Global Trust Authority is licensed under the Mulan PSL v2.
 * You can use this software according to the terms and conditions of the Mulan PSL v2.
 * You may obtain a copy of Mulan PSL v2 at:
 *     http://license.coscl.org.cn/MulanPSL2
 * THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR
 * PURPOSE.
 * See the Mulan PSL v2 for more details.
 */

//! JSON types for HTTP responses; [`utoipa::ToSchema`] drives `OpenAPI` `components.schemas` export.
//!
//! `OpenAPI` 3.x labels one optional field on each schema property as `example`: the documented
//! representative value in the machine-readable contract (`openapi.yaml`). That keyword is fixed
//! by the specification and by [`utoipa`]; it denotes **published API documentation**, not informal
//! sample or throwaway code. Helpers below supply the same canonical strings as the running service.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Public service name for APIs, logs, and the exported `OpenAPI` document.
pub const SERVICE_NAME: &str = "globaltrustauthority-rbs";

/// Published REST API contract version string.
pub const API_VERSION: &str = "0.1.0";

/// Value written to the exported `OpenAPI` document for `service_name` (same as [`SERVICE_NAME`]).
/// Required by [`utoipa`] as the argument to `#[schema(example = …)]` (`OpenAPI` spec keyword).
fn open_api_schema_service_name() -> &'static str {
    SERVICE_NAME
}

/// Value written to the exported `OpenAPI` document for `api_version` (same as [`API_VERSION`]).
fn open_api_schema_api_version() -> &'static str {
    API_VERSION
}

/// Value written to the exported `OpenAPI` document for `build.version` (compile-time
/// `CARGO_PKG_VERSION` for this crate).
///
/// # Invariant
///
/// All workspace members share one version (`version.workspace = true` in each
/// `Cargo.toml`). If per-crate versioning is ever introduced, this example and the
/// runtime value in `rbs-core` (`env!("CARGO_PKG_VERSION")`) will diverge silently —
/// add a compile-time or CI check before splitting versions.
fn open_api_schema_build_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Value written to the exported `OpenAPI` document for `git_hash` when no VCS info is available.
fn open_api_schema_git_hash() -> &'static str {
    "unknown"
}

/// Value written to the exported `OpenAPI` document for `build_date` when no build timestamp is available.
fn open_api_schema_build_date() -> &'static str {
    "unknown"
}

/// Error payload for HTTP error responses (e.g. 500).
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct ErrorBody {
    /// Error string for the caller: may be a stable code, a short machine-oriented label,
    /// or a concise human-readable message. Must not include stack traces or secrets.
    pub error: String,
}

/// Build-time identity for the running binary.
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct BuildMetadata {
    /// Cargo package / release version (semver).
    #[schema(example = open_api_schema_build_version)]
    pub version: String,
    /// Git commit hash at build time (or placeholder when unknown).
    #[schema(example = open_api_schema_git_hash)]
    pub git_hash: String,
    /// Build timestamp (UTC), implementation-defined format.
    #[schema(example = open_api_schema_build_date)]
    pub build_date: String,
}

/// JSON emitted by `GET /rbs/version` (`service_name`, `api_version`, structured `build`).
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct RbsVersion {
    /// Logical service display name.
    #[schema(example = open_api_schema_service_name)]
    pub service_name: String,
    /// Published API contract version string.
    #[schema(example = open_api_schema_api_version)]
    pub api_version: String,
    pub build: BuildMetadata,
}
