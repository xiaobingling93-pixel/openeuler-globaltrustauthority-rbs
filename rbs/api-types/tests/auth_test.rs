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

//! Integration tests for auth types.

use rbs_api_types::{
    AttestRequest, AttestResponse, AuthChallengeResponse, RbcMeasurement,
};

#[test]
fn test_auth_challenge_response() {
    let json = r#"{"nonce":"test-nonce-value"}"#;
    let resp: AuthChallengeResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.nonce, "test-nonce-value");
}

#[test]
fn test_attest_response() {
    let json = r#"{"token":"jwt-token-here"}"#;
    let resp: AttestResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.token, "jwt-token-here");
}

#[test]
fn test_rbc_measurement() {
    let json = r#"{"nonce":"abc123"}"#;
    let m: RbcMeasurement = serde_json::from_str(json).unwrap();
    assert_eq!(m.nonce, "abc123");
    assert!(m.evidences.is_none());
}

#[test]
fn test_attest_request_full() {
    let json = serde_json::json!({
        "as_provider": "gta",
        "rbc_evidences": {
            "measurements": [
                {
                    "nonce": "challenge-nonce",
                    "evidences": []
                }
            ]
        }
    });
    let req: AttestRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.as_provider.as_deref(), Some("gta"));
    assert_eq!(req.rbc_evidences.measurements.len(), 1);
    assert_eq!(req.rbc_evidences.measurements[0].nonce, "challenge-nonce");
}
