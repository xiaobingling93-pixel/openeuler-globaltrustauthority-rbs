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

use std::env;

use crate::cli::GlobalCliArgs;
use crate::common::validate::{validate_file_path, validate_max_len, validate_not_empty, validate_url};
use crate::config::{GlobalOptions, OutputFormat, DEFAULT_BASE_URL, DEFAULT_FORMAT};
use crate::error::Result;

pub fn resolve_global_options(cli: &GlobalCliArgs) -> Result<GlobalOptions> {
    let env_base_url = env::var("RBS_BASE_URL").ok();
    let env_token = env::var("RBS_TOKEN").ok();
    let env_cert = env::var("RBS_CERT").ok();
    let env_format = env::var("RBS_FORMAT").ok();

    if let Some(format) = &env_format {
        format.parse::<OutputFormat>()?;
    }

    let base_url = cli.base_url.clone().unwrap_or_else(|| env_base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()));
    let format = cli.format.clone().unwrap_or_else(|| {
        env_format.unwrap_or_else(|| DEFAULT_FORMAT.to_string()).parse::<OutputFormat>().unwrap_or(OutputFormat::Text)
    });
    let token = cli.token.clone().or_else(|| env_token);
    let cert = cli.cert.clone().or_else(|| env_cert);

    validate_base_url(&base_url)?;
    if let Some(token) = &token {
        validate_token(token)?;
    }
    if let Some(cert) = &cert {
        validate_cert(cert)?;
    }

    Ok(GlobalOptions {
        base_url,
        token,
        cert,
        format,
        output_file: cli.output_file.clone(),
        verbose: cli.verbose,
        quiet: cli.quiet,
        noout: cli.noout,
    })
}

pub fn validate_base_url(base_url: &str) -> Result<String> {
    validate_not_empty(base_url)?;
    validate_url(base_url)?;
    validate_max_len(base_url, 2048)?;
    Ok(base_url.to_string())
}

pub fn validate_token(token: &str) -> Result<String> {
    validate_not_empty(token)?;
    validate_max_len(token, 16384)?;
    Ok(token.to_string())
}

pub fn validate_cert(cert: &str) -> Result<String> {
    validate_not_empty(cert)?;
    validate_max_len(cert, 4096)?;
    validate_file_path(cert)?;
    Ok(cert.to_string())
}

pub fn validate_output_file(output_file: &str) -> Result<String> {
    validate_not_empty(output_file)?;
    validate_max_len(output_file, 4096)?;
    validate_file_path(output_file)?;
    Ok(output_file.to_string())
}

#[cfg(test)]
mod tests {
    use super::{resolve_global_options, validate_base_url};
    use crate::cli::GlobalCliArgs;
    use crate::config::OutputFormat;

    #[test]
    fn resolve_global_options_uses_defaults() {
        let cli = GlobalCliArgs::default();

        let config = resolve_global_options(&cli).unwrap();
        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.format, OutputFormat::Text);
    }

    #[test]
    fn resolve_global_options_uses_cert_arg() {
        let cli = GlobalCliArgs { cert: Some("ca.pem".to_string()), ..Default::default() };

        let config = resolve_global_options(&cli).unwrap();
        assert_eq!(config.cert, Some("ca.pem".to_string()));
    }

    #[test]
    fn validate_base_url_rejects_invalid_url() {
        assert!(validate_base_url("not-a-url").is_err());
    }
}
