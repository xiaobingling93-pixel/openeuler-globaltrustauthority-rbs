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

use clap::{Args, Parser, Subcommand};

use crate::admin::cert::CertCli;
use crate::admin::policy::PolicyCli;
use crate::admin::ref_value::RefValueCli;
use crate::admin::res::ResCli;
use crate::admin::res_policy::ResPolicyCli;
use crate::admin::user::UserCli;
use crate::client::cmd::ClientCli;
use crate::config::cmd::{validate_base_url, validate_cert, validate_output_file, validate_token};
use crate::config::OutputFormat;
use crate::token::cmd::TokenCli;
use crate::version::cmd::VersionCli;

#[derive(Parser, Debug)]
#[command(name = "rbs-cli")]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalCliArgs,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct GlobalCliArgs {
    #[arg(short = 'b', long, global = true, value_parser = validate_base_url)]
    pub base_url: Option<String>,

    #[arg(short, long, global = true, value_parser = validate_token)]
    pub token: Option<String>,

    #[arg(long, global = true, value_parser = validate_cert)]
    pub cert: Option<String>,

    #[arg(short, long, global = true, value_enum)]
    pub format: Option<OutputFormat>,

    #[arg(short, long, global = true, value_parser = validate_output_file)]
    pub output_file: Option<String>,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    #[arg(long, global = true)]
    pub noout: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Cert(CertCli),
    Client(ClientCli),
    Policy(PolicyCli),
    RefValue(RefValueCli),
    Res(ResCli),
    ResPolicy(ResPolicyCli),
    Token(TokenCli),
    User(UserCli),
    Version(VersionCli),
}
