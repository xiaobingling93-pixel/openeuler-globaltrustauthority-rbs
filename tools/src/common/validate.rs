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

use crate::error::CliError;
use std::path::Path;

pub trait HasLen {
    fn len(&self) -> usize;
}

impl HasLen for str {
    fn len(&self) -> usize {
        str::len(self)
    }
}

impl HasLen for String {
    fn len(&self) -> usize {
        String::len(self)
    }
}

impl<T> HasLen for [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> HasLen for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

pub fn validate_max_len<T>(value: &T, max: usize) -> crate::error::Result<()>
where
    T: HasLen + ?Sized,
{
    let len = value.len();
    if len <= max {
        Ok(())
    } else {
        Err(CliError::InvalidArgument(format!("value length must not exceed {max} characters; got {len}")))
    }
}

pub fn validate_not_empty(value: &str) -> crate::error::Result<String> {
    if value.is_empty() {
        return Err(CliError::InvalidArgument("value is empty".to_string()));
    }
    Ok(value.into())
}

pub fn validate_file_path(file_path_str: &str) -> crate::error::Result<String> {
    if file_path_str.trim().is_empty() {
        return Err(CliError::InvalidArgument("file path must not be empty".to_string()));
    }

    let path = Path::new(file_path_str);
    if path.exists() && path.is_dir() {
        return Err(CliError::InvalidArgument(format!(
            "output path `{file_path_str}` points to an existing directory"
        )));
    }
    if path.file_name().is_none() {
        return Err(CliError::InvalidArgument(format!("file path `{file_path_str}` does not contain a file name")));
    }

    Ok(file_path_str.into())
}

pub fn validate_url(url: &str) -> crate::error::Result<()> {
    url.parse::<reqwest::Url>()
        .map(|_| ())
        .map_err(|err| CliError::InvalidArgument(format!("invalid url `{url}`: {err}")))
}
