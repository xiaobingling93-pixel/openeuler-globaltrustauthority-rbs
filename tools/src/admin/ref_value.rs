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

use clap::{Args, Subcommand};

use crate::common::formatter::{Formatter, TextOutput};
use crate::config::GlobalOptions;
use crate::error::Result;

#[derive(Args, Debug, Clone)]
pub struct RefValueCli {
    #[command(subcommand)]
    pub command: RefValueCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RefValueCommand {
    List,
    Get,
    Create,
    Update,
    Delete,
}

pub fn run(cli: &RefValueCli, _global: &GlobalOptions) -> Result<Box<dyn Formatter>> {
    let message = match cli.command {
        RefValueCommand::List => "ref-value list",
        RefValueCommand::Get => "ref-value get",
        RefValueCommand::Create => "ref-value create",
        RefValueCommand::Update => "ref-value update",
        RefValueCommand::Delete => "ref-value delete",
    };
    Ok(Box::new(TextOutput::new(message)))
}
