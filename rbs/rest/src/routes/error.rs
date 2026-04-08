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

//! JSON error responses (`ErrorBody`) for framework-level HTTP errors (404, 414, 429).

use actix_web::HttpResponse;
use rbs_api_types::ErrorBody;

/// No matching route under `/rbs` or `/rbs/v0`.
pub async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(ErrorBody {
        error: "Not Found".to_string(),
    })
}
