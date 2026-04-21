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

//! Integration tests for resource types.

use rbs_api_types::{
    ResourceContentResponse, ResourceInfoResponse, ResourceMetadataResponse,
    ResourceRetrieveRequest, ResourceUpsertRequest,
};

#[test]
fn test_resource_content_response() {
    let json = serde_json::json!({
        "uri": "/rbs/v0/provider1/repo1/key/mykey",
        "content": "SGVsbG9Xb3JsZA==",
        "content_type": "text/plain"
    });
    let resp: ResourceContentResponse = serde_json::from_value(json).unwrap();
    assert_eq!(resp.uri, "/rbs/v0/provider1/repo1/key/mykey");
    assert_eq!(resp.content, "SGVsbG9Xb3JsZA==");
    assert_eq!(resp.content_type.as_deref(), Some("text/plain"));
}

#[test]
fn test_resource_upsert_request() {
    let json = serde_json::json!({
        "content": "dGVzdA==",
        "content_type": "application/octet-stream"
    });
    let req: ResourceUpsertRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.content, "dGVzdA==");
}

#[test]
fn test_resource_info_response() {
    let json = serde_json::json!({
        "uri": "/rbs/v0/provider1/repo1/key/mykey",
        "resource_type": "key",
        "created_at": "2026-01-01T00:00:00Z"
    });
    let resp: ResourceInfoResponse = serde_json::from_value(json).unwrap();
    assert_eq!(resp.uri, "/rbs/v0/provider1/repo1/key/mykey");
    assert_eq!(resp.resource_type.as_deref(), Some("key"));
    assert!(resp.created_at.is_some());
}

#[test]
fn test_resource_metadata_response() {
    let json = serde_json::json!({
        "uri": "/rbs/v0/provider1/repo1/key/mykey",
        "res_provider": "provider1",
        "repository_name": "repo1",
        "resource_type": "key",
        "resource_name": "mykey"
    });
    let resp: ResourceMetadataResponse = serde_json::from_value(json).unwrap();
    assert_eq!(
        resp.uri.as_deref(),
        Some("/rbs/v0/provider1/repo1/key/mykey")
    );
    assert_eq!(resp.res_provider.as_deref(), Some("provider1"));
}

#[test]
fn test_resource_retrieve_request_is_attest_request() {
    let json = serde_json::json!({
        "rbc_evidences": {
            "measurements": [{"nonce": "test-nonce"}]
        }
    });
    let req: ResourceRetrieveRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.rbc_evidences.measurements[0].nonce, "test-nonce");
}
