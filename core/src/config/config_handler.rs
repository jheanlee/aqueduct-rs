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

use crate::config::args::Args;
use crate::config::error::ConfigError;
use clap::Parser;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::num::ParseIntError;
use std::str::FromStr;

pub struct Config {
    pub tunnel_host: SocketAddr,
    pub tunnel_allowed_ports: VecDeque<u16>,
    pub tunnel_global_connection_limit: u32,
    pub tunnel_client_connection_limit: u32,

    pub tls_cert_path: String,
    pub tls_private_key_path: String,

    pub api_host: SocketAddr,

    pub jwt_access_private_key_path: String,
    pub jwt_access_public_key_path: String,
    pub jwt_refresh_private_key_path: String,
    pub jwt_refresh_public_key_path: String,

    pub db_name: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_username: String,
    pub db_password: String,
}

///   Reads config from
///     1. command line args
///     2. environment variables
///     3. default value
pub fn read_config() -> Result<Config, ConfigError> {
    let mut config = Config {
        tunnel_host: SocketAddr::from_str("0.0.0.0:30330")?,
        tunnel_allowed_ports: (51000..=51999).collect(),
        tunnel_global_connection_limit: 10000,
        tunnel_client_connection_limit: 2048,

        tls_cert_path: "".to_string(),
        tls_private_key_path: "".to_string(),

        api_host: SocketAddr::from_str("0.0.0.0:30331")?,
        jwt_access_private_key_path: "".to_string(),
        jwt_access_public_key_path: "".to_string(),
        jwt_refresh_private_key_path: "".to_string(),
        jwt_refresh_public_key_path: "".to_string(),

        db_name: "aqueduct-rs".to_string(),
        db_host: "127.0.0.1".to_string(),
        db_port: 5432,
        db_username: "postgres".to_string(),
        db_password: "".to_string(),
    };

    //  environment variable
    if let Ok(tunnel_host) = std::env::var("AQUEDUCT_HOST") {
        config.tunnel_host = tunnel_host.parse()?;
    }
    if let Ok(tunnel_allowed_ports) = std::env::var("AQUEDUCT_TUNNEL_ALLOWED_PORTS") {
        config.tunnel_allowed_ports = parse_port_list(tunnel_allowed_ports.as_str())
            .map_err(|_| {
                ConfigError::ParseError((
                    "config".to_string(),
                    "AQUEDUCT_TUNNEL_ALLOWED_PORTS".to_string(),
                ))
            })?
            .into()
    }
    if let Ok(tunnel_global_connection_limit) =
        std::env::var("AQUEDUCT_TUNNEL_GLOBAL_CONNECTION_LIMIT")
    {
        config.tunnel_global_connection_limit =
            tunnel_global_connection_limit.parse().map_err(|_| {
                ConfigError::ParseError((
                    "config".to_string(),
                    "AQUEDUCT_TUNNEL_GLOBAL_CONNECTION_LIMIT".to_string(),
                ))
            })?;
    }
    if let Ok(tunnel_client_connection_limit) =
        std::env::var("AQUEDUCT_TUNNEL_CLIENT_CONNECTION_LIMIT")
    {
        config.tunnel_client_connection_limit =
            tunnel_client_connection_limit.parse().map_err(|_| {
                ConfigError::ParseError((
                    "config".to_string(),
                    "AQUEDUCT_TUNNEL_CLIENT_CONNECTION_LIMIT".to_string(),
                ))
            })?;
    }
    if let Ok(tls_cert) = std::env::var("AQUEDUCT_TLS_CERT") {
        config.tls_cert_path = tls_cert;
    }
    if let Ok(tls_private_key) = std::env::var("AQUEDUCT_TLS_PRIVATE_KEY") {
        config.tls_private_key_path = tls_private_key;
    }
    if let Ok(api_host) = std::env::var("AQUEDUCT_API_HOST") {
        config.api_host = api_host.parse()?;
    }
    if let Ok(jwt_access_private_key_path) = std::env::var("AQUEDUCT_JWT_ACCESS_PRIVATE_KEY") {
        config.jwt_access_private_key_path = jwt_access_private_key_path;
    }
    if let Ok(jwt_access_public_key_path) = std::env::var("AQUEDUCT_JWT_ACCESS_PUBLIC_KEY") {
        config.jwt_access_public_key_path = jwt_access_public_key_path;
    }
    if let Ok(jwt_refresh_private_key_path) = std::env::var("AQUEDUCT_JWT_REFRESH_PRIVATE_KEY") {
        config.jwt_refresh_private_key_path = jwt_refresh_private_key_path;
    }
    if let Ok(jwt_refresh_public_key_path) = std::env::var("AQUEDUCT_JWT_REFRESH_PUBLIC_KEY") {
        config.jwt_refresh_public_key_path = jwt_refresh_public_key_path;
    }
    if let Ok(db_name) = std::env::var("AQUEDUCT_DB_NAME") {
        config.db_name = db_name;
    }
    if let Ok(db_host) = std::env::var("AQUEDUCT_DB_HOST") {
        config.db_host = db_host;
    }
    if let Ok(db_port) = std::env::var("AQUEDUCT_DB_PORT") {
        config.db_port = db_port.parse().map_err(|_| {
            ConfigError::ParseError(("config".to_string(), "AQUEDUCT_DB_PORT".to_string()))
        })?;
    }
    if let Ok(db_username) = std::env::var("AQUEDUCT_DB_USERNAME") {
        config.db_username = db_username;
    }
    if let Ok(db_password) = std::env::var("AQUEDUCT_DB_PASSWORD") {
        config.db_password = db_password;
    }

    //  args
    let args = Args::parse();
    if let Some(tunnel_host) = args.host {
        config.tunnel_host = tunnel_host;
    }
    if let Some(tunnel_allowed_ports) = args.tunnel_allowed_ports {
        config.tunnel_allowed_ports = parse_port_list(tunnel_allowed_ports.as_str())
            .map_err(|_| {
                ConfigError::ParseError((
                    "config".to_string(),
                    "--tunnel-allowed-ports".to_string(),
                ))
            })?
            .into();
    }
    if let Some(tunnel_global_connection_limit) = args.tunnel_global_connection_limit {
        config.tunnel_global_connection_limit = tunnel_global_connection_limit;
    }
    if let Some(tunnel_client_connection_limit) = args.tunnel_client_connection_limit {
        config.tunnel_client_connection_limit = tunnel_client_connection_limit;
    }
    if let Some(tls_cert) = args.tls_cert {
        config.tls_cert_path = tls_cert;
    }
    if let Some(tls_private_key) = args.tls_private_key {
        config.tls_private_key_path = tls_private_key;
    }
    if let Some(api_host) = args.api_host {
        config.api_host = api_host;
    }
    if let Some(jwt_access_private_key_path) = args.jwt_access_private_key {
        config.jwt_access_private_key_path = jwt_access_private_key_path;
    }
    if let Some(jwt_access_public_key_path) = args.jwt_access_public_key {
        config.jwt_access_public_key_path = jwt_access_public_key_path;
    }
    if let Some(jwt_refresh_private_key_path) = args.jwt_refresh_private_key {
        config.jwt_refresh_private_key_path = jwt_refresh_private_key_path;
    }
    if let Some(jwt_refresh_public_key_path) = args.jwt_refresh_public_key {
        config.jwt_refresh_public_key_path = jwt_refresh_public_key_path;
    }
    if let Some(db_name) = args.db_name {
        config.db_name = db_name;
    }
    if let Some(db_host) = args.db_host {
        config.db_host = db_host;
    }
    if let Some(db_port) = args.db_port {
        config.db_port = db_port;
    }
    if let Some(db_username) = args.db_username {
        config.db_username = db_username;
    }
    if let Some(db_password) = args.db_password {
        config.db_password = db_password;
    }

    //  check required field
    if config.tls_cert_path.is_empty() {
        Err(ConfigError::RequiredFieldEmpty((
            "tls_cert".to_string(),
            "AQUEDUCT_TLS_CERT".to_string(),
        )))
    } else if config.tls_private_key_path.is_empty() {
        Err(ConfigError::RequiredFieldEmpty((
            "tls_private_key".to_string(),
            "AQUEDUCT_TLS_PRIVATE_KEY".to_string(),
        )))
    } else {
        Ok(config)
    }
}

fn parse_port_list(arg_string: &str) -> Result<Vec<u16>, ParseIntError> {
    let mut allowed_ports = Vec::new();

    for entry in arg_string.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }

        if entry.contains('-') {
            let bounds: Vec<&str> = entry.split('-').collect();
            let start: u16 = bounds[0].parse()?;
            let end: u16 = bounds[1].parse()?;
            allowed_ports.extend(start..=end);
        } else {
            let port: u16 = entry.parse()?;
            allowed_ports.push(port);
        }
    }

    Ok(allowed_ports)
}
