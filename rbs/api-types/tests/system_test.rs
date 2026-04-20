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

//! Integration tests for system types.

use rbs_api_types::{BuildMetadata, ErrorBody, RbsVersion};

#[test]
fn test_error_body() {
    let eb = ErrorBody::new("invalid_request");
    assert_eq!(eb.error, "invalid_request");
}

#[test]
fn test_error_body_from_str() {
    let eb = ErrorBody::from("RBS-AUTHN-001");
    assert_eq!(eb.error, "RBS-AUTHN-001");
}

#[test]
fn test_rbs_version() {
    let json = serde_json::json!({
        "service_name": "rbs",
        "api_version": "0.1.0",
        "build": {
            "version": "0.1.0",
            "git_hash": "abc123",
            "build_date": "2026-04-17"
        }
    });
    let resp: RbsVersion = serde_json::from_value(json).unwrap();
    assert_eq!(resp.service_name, "rbs");
    assert_eq!(resp.api_version, "0.1.0");
    assert_eq!(resp.build.version, "0.1.0");
    assert_eq!(resp.build.git_hash, "abc123");
}

#[test]
fn test_build_metadata() {
    let json = serde_json::json!({
        "version": "1.0.0",
        "git_hash": "def456",
        "build_date": "2026-04-17T10:00:00Z"
    });
    let meta: BuildMetadata = serde_json::from_value(json).unwrap();
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(meta.git_hash, "def456");
}

