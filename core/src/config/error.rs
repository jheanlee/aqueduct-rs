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

#[derive(Debug)]
pub enum ConfigError {
    AddrParseError,
    ParseError((String, String)),
    LogInitError(crate::common::log::Error),
    RequiredFieldEmpty((String, String)),
    OrmError(crate::orm::error::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::AddrParseError => write!(f, "Invalid address format"),
            ConfigError::ParseError((config_type, value)) => {
                write!(f, "Error while parsing {config_type} value \"{value}\"")
            }
            ConfigError::RequiredFieldEmpty((arg_name, env_name)) => write!(
                f,
                "Required field must be set: `--{arg_name}` or environment variable `{env_name}`"
            ),
            ConfigError::LogInitError(error) => write!(f, "{error}"),
            ConfigError::OrmError(error) => write!(f, "Database error: {error}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::net::AddrParseError> for ConfigError {
    fn from(_value: std::net::AddrParseError) -> Self {
        ConfigError::AddrParseError
    }
}

impl From<crate::common::log::Error> for ConfigError {
    fn from(value: crate::common::log::Error) -> Self {
        ConfigError::LogInitError(value)
    }
}

impl From<crate::orm::error::Error> for ConfigError {
    fn from(value: crate::orm::error::Error) -> Self {
        ConfigError::OrmError(value)
    }
}
