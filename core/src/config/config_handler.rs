use std::net::SocketAddr;
use std::str::FromStr;
use clap::Parser;
use crate::config::args::Args;
use crate::config::error::ConfigError;

pub struct Config {
  pub tunnel_host: SocketAddr,

  pub tls_cert_path: String,
  pub tls_private_key_path: String,

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
    tls_cert_path: "".to_string(),
    tls_private_key_path: "".to_string(),
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
  if let Ok(tls_cert) = std::env::var("AQUEDUCT_TLS_CERT") {
    config.tls_cert_path = tls_cert;
  }
  if let Ok(tls_private_key) = std::env::var("AQUEDUCT_TLS_PRIVATE_KEY") {
    config.tls_private_key_path = tls_private_key;
  }
  if let Ok(db_name) = std::env::var("AQUEDUCT_DB_NAME") {
    config.db_name = db_name;
  }
  if let Ok(db_host) = std::env::var("AQUEDUCT_DB_HOST") {
    config.db_host = db_host;
  }
  if let Ok(db_port) = std::env::var("AQUEDUCT_DB_PORT") {
    config.db_port = db_port.parse()?;
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
  if let Some(tls_cert) = args.tls_cert {
    config.tls_cert_path = tls_cert;
  }
  if let Some(tls_private_key) = args.tls_private_key {
    config.tls_private_key_path = tls_private_key;
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
    Err(ConfigError::RequiredFieldEmpty(("tls_cert".to_string(), "AQUEDUCT_TLS_CERT".to_string())))
  } else if config.tls_private_key_path.is_empty() {
    Err(ConfigError::RequiredFieldEmpty(("tls_private_key".to_string(), "AQUEDUCT_TLS_PRIVATE_KEY".to_string())))
  } else {
    Ok(config)
  }
}