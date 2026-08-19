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

use crate::config::args::{Args, Commands};
use crate::config::error::ConfigError;
use clap::Parser;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::num::ParseIntError;
use std::str::FromStr;

pub struct Config {
    pub subcommand: Option<Commands>,

    pub tunnel_bind_address: SocketAddr,
    pub tunnel_allowed_ports: VecDeque<u16>,
    pub tunnel_global_connection_limit: u32,
    pub tunnel_client_connection_limit: u32,

    pub tls_cert_path: String,
    pub tls_private_key_path: String,

    #[cfg(feature = "api")]
    pub api_bind_address: SocketAddr,

    #[cfg(feature = "api")]
    pub jwt_access_private_key_path: String,
    #[cfg(feature = "api")]
    pub jwt_access_public_key_path: String,
    #[cfg(feature = "api")]
    pub jwt_refresh_private_key_path: String,
    #[cfg(feature = "api")]
    pub jwt_refresh_public_key_path: String,

    pub db_name: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
}

///   Reads config from
///     1. command line args
///     2. environment variables
///     3. default value
pub fn read_config() -> Result<Config, ConfigError> {
    let mut config = Config {
        subcommand: None,

        tunnel_bind_address: SocketAddr::from_str("0.0.0.0:30330")?,
        tunnel_allowed_ports: (51000..=51999).collect(),
        tunnel_global_connection_limit: 16384,
        tunnel_client_connection_limit: 256,

        tls_cert_path: "".to_string(),
        tls_private_key_path: "".to_string(),

        #[cfg(feature = "api")]
        api_bind_address: SocketAddr::from_str("0.0.0.0:30331")?,
        #[cfg(feature = "api")]
        jwt_access_private_key_path: "".to_string(),
        #[cfg(feature = "api")]
        jwt_access_public_key_path: "".to_string(),
        #[cfg(feature = "api")]
        jwt_refresh_private_key_path: "".to_string(),
        #[cfg(feature = "api")]
        jwt_refresh_public_key_path: "".to_string(),

        db_name: "aqueduct".to_string(),
        db_host: "127.0.0.1".to_string(),
        db_port: 5432,
        db_user: "postgres".to_string(),
        db_password: "".to_string(),
    };

    //  environment variable
    if let Ok(tunnel_bind_address) = std::env::var("AQUEDUCT_BIND_ADDRESS") {
        config.tunnel_bind_address = tunnel_bind_address.parse()?;
    }
    #[cfg(feature = "api")]
    if let Ok(api_bind_address) = std::env::var("AQUEDUCT_API_BIND_ADDRESS") {
        config.api_bind_address = api_bind_address.parse()?;
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
    if let Ok(tls_cert) = std::env::var("AQUEDUCT_TLS_CERTIFICATE_FILE") {
        config.tls_cert_path = tls_cert;
    }
    if let Ok(tls_private_key) = std::env::var("AQUEDUCT_TLS_PRIVATE_KEY_FILE") {
        config.tls_private_key_path = tls_private_key;
    }
    #[cfg(feature = "api")]
    if let Ok(jwt_access_private_key_path) = std::env::var("AQUEDUCT_JWT_ACCESS_PRIVATE_KEY_FILE") {
        config.jwt_access_private_key_path = jwt_access_private_key_path;
    }
    #[cfg(feature = "api")]
    if let Ok(jwt_access_public_key_path) = std::env::var("AQUEDUCT_JWT_ACCESS_PUBLIC_KEY_FILE") {
        config.jwt_access_public_key_path = jwt_access_public_key_path;
    }
    #[cfg(feature = "api")]
    if let Ok(jwt_refresh_private_key_path) = std::env::var("AQUEDUCT_JWT_REFRESH_PRIVATE_KEY_FILE")
    {
        config.jwt_refresh_private_key_path = jwt_refresh_private_key_path;
    }
    #[cfg(feature = "api")]
    if let Ok(jwt_refresh_public_key_path) = std::env::var("AQUEDUCT_JWT_REFRESH_PUBLIC_KEY_FILE") {
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
    if let Ok(db_user) = std::env::var("AQUEDUCT_DB_USER") {
        config.db_user = db_user;
    }
    if let Ok(db_password) = std::env::var("AQUEDUCT_DB_PASSWORD") {
        config.db_password = db_password;
    }

    //  args
    let args = Args::parse();
    config.subcommand = args.command;
    if let Some(tunnel_bind_address) = args.bind_address {
        config.tunnel_bind_address = tunnel_bind_address;
    }
    #[cfg(feature = "api")]
    if let Some(api_bind_address) = args.api_bind_address {
        config.api_bind_address = api_bind_address;
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
    if let Some(tls_cert) = args.tls_certificate_file {
        config.tls_cert_path = tls_cert;
    }
    if let Some(tls_private_key) = args.tls_private_key_file {
        config.tls_private_key_path = tls_private_key;
    }
    #[cfg(feature = "api")]
    if let Some(jwt_access_private_key_path) = args.jwt_access_private_key_file {
        config.jwt_access_private_key_path = jwt_access_private_key_path;
    }
    #[cfg(feature = "api")]
    if let Some(jwt_access_public_key_path) = args.jwt_access_public_key_file {
        config.jwt_access_public_key_path = jwt_access_public_key_path;
    }
    #[cfg(feature = "api")]
    if let Some(jwt_refresh_private_key_path) = args.jwt_refresh_private_key_file {
        config.jwt_refresh_private_key_path = jwt_refresh_private_key_path;
    }
    #[cfg(feature = "api")]
    if let Some(jwt_refresh_public_key_path) = args.jwt_refresh_public_key_file {
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
    if let Some(db_user) = args.db_user {
        config.db_user = db_user;
    }
    if let Some(db_password) = args.db_password {
        config.db_password = db_password;
    }

    //  check required field
    if config.subcommand.is_none() && config.tls_cert_path.is_empty() {
        Err(ConfigError::RequiredFieldEmpty((
            "tls-certificate-file".to_string(),
            "AQUEDUCT_TLS_CERTIFICATE_FILE".to_string(),
        )))?
    }
    if config.subcommand.is_none() && config.tls_private_key_path.is_empty() {
        Err(ConfigError::RequiredFieldEmpty((
            "tls-private-key-file".to_string(),
            "AQUEDUCT_TLS_PRIVATE_KEY_FILE".to_string(),
        )))?
    }
    #[cfg(feature = "api")]
    {
        if config.subcommand.is_none() && config.jwt_refresh_private_key_path.is_empty() {
            Err(ConfigError::RequiredFieldEmpty((
                "jwt-refresh-private-key-file".to_string(),
                "AQUEDUCT_JWT_REFRESH_PRIVATE_KEY_FILE".to_string(),
            )))?
        }
        if config.subcommand.is_none() && config.jwt_refresh_public_key_path.is_empty() {
            Err(ConfigError::RequiredFieldEmpty((
                "jwt-refresh-public-key-file".to_string(),
                "AQUEDUCT_JWT_REFRESH_PUBLIC_KEY_FILE".to_string(),
            )))?
        }
        if config.subcommand.is_none() && config.jwt_access_private_key_path.is_empty() {
            Err(ConfigError::RequiredFieldEmpty((
                "jwt-access-private-key-file".to_string(),
                "AQUEDUCT_JWT_ACCESS_PRIVATE_KEY_FILE".to_string(),
            )))?
        }
        if config.subcommand.is_none() && config.jwt_access_public_key_path.is_empty() {
            Err(ConfigError::RequiredFieldEmpty((
                "jwt-access-public-key-file".to_string(),
                "AQUEDUCT_JWT_ACCESS_PUBLIC_KEY_FILE".to_string(),
            )))?
        }
    }

    Ok(config)
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
