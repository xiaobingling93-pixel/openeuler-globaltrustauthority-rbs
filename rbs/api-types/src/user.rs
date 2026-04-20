/*
 * Copyright (c) Huawei Technologies Co., Ltd. 2026. All rights reserved.
 * Global Trust Authority Resource Broker Service is licensed under the Mulan PSL v2.
 * You can use this software according to the terms and conditions of the Mulan PSL v2.
 * You may obtain a copy of Mulan PSL v2 at:
 *     http://license.coscl.org.cn/MulanPSL2
 * THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR
 * PURPOSE.
 * See the Mulan PSL v2 for more details.
 */

//! User management types.
//!
//! Covers user creation, update, retrieval, and listing.

use serde::{Deserialize, Serialize};
use serde_json::Map;

/// Request body for POST /rbs/v0/users (create user).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserCreateRequest {
    /// Login or unique handle (encoding per deployment).
    pub username: String,

    /// Optional role names (e.g. administrator); default roles are deployment-specific.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

/// Request body for PUT /rbs/v0/users/{user_id} (update user).
///
/// Only supplied fields are changed (semantics per implementation).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserUpdateRequest {
    /// New login name (if supported by implementation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Replace or merge roles (semantics per implementation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,

    /// Whether the account can authenticate, if supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Optional JWT verification material (JWKS URI, public key, etc.) when supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt_verification: Option<Map<String, serde_json::Value>>,
}

/// Response for user retrieval and creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserResponse {
    /// Stable user id (same as path `user_id` where applicable).
    pub id: String,
    /// Human-facing login or handle.
    pub username: String,
}

/// Paginated response for GET /rbs/v0/users.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserListResponse {
    /// Page of users.
    pub items: Vec<UserResponse>,
    /// Total matching users (not only this page).
    pub total_count: i64,
    /// Effective page size (may mirror request `limit`).
    pub limit: i64,
    /// Effective skip count (may mirror request `offset`).
    pub offset: i64,
}
