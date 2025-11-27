use std::net::SocketAddr;
use std::str::FromStr;
use clap::Parser;
use crate::common::args::Args;
use crate::config::error::ConfigError;

pub struct Config {
  pub tunnel_host: SocketAddr,

  pub tls_cert_path: String,
  pub tls_private_key_path: String
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

  //  check required field
  if config.tls_cert_path.is_empty() {
    Err(ConfigError::RequiredFieldEmpty(("tls_cert".to_string(), "AQUEDUCT_TLS_CERT".to_string())))
  } else if config.tls_private_key_path.is_empty() {
    Err(ConfigError::RequiredFieldEmpty(("tls_private_key".to_string(), "AQUEDUCT_TLS_PRIVATE_KEY".to_string())))
  } else {
    Ok(config)
  }
}