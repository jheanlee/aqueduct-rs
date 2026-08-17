/*
 * Copyright 2026 Jhe-An Lee
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[cfg(feature = "migration")]
use clap::ValueEnum;
use std::net::SocketAddr;

#[derive(clap::Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(long)]
    pub host: Option<SocketAddr>,
    #[arg(long)]
    pub tunnel_allowed_ports: Option<String>,
    #[arg(long)]
    pub tunnel_global_connection_limit: Option<u32>,
    #[arg(long)]
    pub tunnel_client_connection_limit: Option<u32>,
    #[arg(long)]
    pub tls_cert: Option<String>,
    #[arg(long)]
    pub tls_private_key: Option<String>,
    #[arg(long)]
    pub api_host: Option<SocketAddr>,
    #[arg(long)]
    pub jwt_access_private_key: Option<String>,
    #[arg(long)]
    pub jwt_access_public_key: Option<String>,
    #[arg(long)]
    pub jwt_refresh_private_key: Option<String>,
    #[arg(long)]
    pub jwt_refresh_public_key: Option<String>,
    #[arg(long)]
    pub db_name: Option<String>,
    #[arg(long)]
    pub db_host: Option<String>,
    #[arg(long)]
    pub db_port: Option<u16>,
    #[arg(long)]
    pub db_username: Option<String>,
    #[arg(long)]
    pub db_password: Option<String>,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    #[cfg(feature = "migration")]
    Migrate(MigrationArgs),
    Init,
}

#[derive(clap::Args)]
#[cfg(feature = "migration")]
pub struct MigrationArgs {
    #[arg(value_enum)]
    pub mode: MigrationModes,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[cfg(feature = "migration")]
pub enum MigrationModes {
    Up,
}
