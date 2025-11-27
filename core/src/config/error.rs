#[derive(Debug)]
pub enum ConfigError {
  AddrParseError(std::net::AddrParseError),
  RequiredFieldEmpty((String, String))
}

impl std::fmt::Display for ConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConfigError::AddrParseError(_) => write!(f, "invalid address format"),
      ConfigError::RequiredFieldEmpty((arg_name, env_name)) => write!(f, "required field must be set: `--{arg_name}` or environment variable `{env_name}`")
    }
  }
}

impl std::error::Error for ConfigError {}

impl From<std::net::AddrParseError> for ConfigError {
  fn from(value: std::net::AddrParseError) -> Self {
    ConfigError::AddrParseError(value)
  }
}