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

//! Integration tests for error types.

use rbs_api_types::{
    error::{ErrorClass, Retryable, RbsError},
};

#[test]
fn test_error_class_mapping() {
    assert_eq!(RbsError::AuthnMissingToken.error_class(), ErrorClass::Authn);
    assert_eq!(RbsError::AuthzDenied.error_class(), ErrorClass::Authz);
    assert_eq!(
        RbsError::ParamMissing { param: "foo" }.error_class(),
        ErrorClass::Param
    );
    assert_eq!(
        RbsError::ResourceNotFound.error_class(),
        ErrorClass::Resource
    );
    assert_eq!(
        RbsError::InternalError.error_class(),
        ErrorClass::Internal
    );
}

#[test]
fn test_http_status() {
    assert_eq!(RbsError::AuthnMissingToken.http_status(), 401);
    assert_eq!(RbsError::AuthzDenied.http_status(), 403);
    assert_eq!(
        RbsError::ParamMissing { param: "foo" }.http_status(),
        400
    );
    assert_eq!(RbsError::ResourceNotFound.http_status(), 404);
    assert_eq!(RbsError::InternalError.http_status(), 500);
}

#[test]
fn test_retryable() {
    assert_eq!(RbsError::ResourceNotFound.retryable(), Retryable::No);
    assert_eq!(
        RbsError::RateLimitExceeded.retryable(),
        Retryable::Yes
    );
    assert_eq!(
        RbsError::InternalError.retryable(),
        Retryable::Idempotent
    );
}
