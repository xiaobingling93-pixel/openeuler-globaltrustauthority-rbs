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

//! RBS main library.
//!
//! Composes rbs-api-types, rbs-core, and rbs-rest to provide a unified entry point.

use std::path::Path;

use anyhow::Context;
use rbs_api_types::config::RbsConfig;

/// Load `RbsConfig` from a YAML file. Requires a non-null **`rest:`** section (YAML `rest: null` is
/// rejected). This is stricter than `serde` defaults alone: programmatic `RbsConfig::default()` may
/// still include `Some(RestConfig::default())` for tests.
pub fn load_config(path: impl AsRef<Path>) -> anyhow::Result<RbsConfig> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path).with_context(|| format!("read config file {}", path.display()))?;
    let config: RbsConfig =
        serde_yaml::from_str(&raw).with_context(|| format!("parse YAML config {}", path.display()))?;
    if config.rest.is_none() {
        anyhow::bail!("config must contain a non-null `rest` section");
    }
    Ok(config)
}
