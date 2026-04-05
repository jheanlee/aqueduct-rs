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
    AddrParseError(std::net::AddrParseError),
    ParseIntError(std::num::ParseIntError),
    ParseBoolError(std::str::ParseBoolError),
    LogInitError(crate::common::log::Error),
    RequiredFieldEmpty((String, String)),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::AddrParseError(_) => write!(f, "invalid address format"),
            ConfigError::ParseIntError(error) => write!(f, "{error}"),
            ConfigError::ParseBoolError(error) => write!(f, "{error}"),
            ConfigError::RequiredFieldEmpty((arg_name, env_name)) => write!(
                f,
                "required field must be set: `--{arg_name}` or environment variable `{env_name}`"
            ),
            ConfigError::LogInitError(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::net::AddrParseError> for ConfigError {
    fn from(value: std::net::AddrParseError) -> Self {
        ConfigError::AddrParseError(value)
    }
}

impl From<std::num::ParseIntError> for ConfigError {
    fn from(value: std::num::ParseIntError) -> Self {
        ConfigError::ParseIntError(value)
    }
}

impl From<std::str::ParseBoolError> for ConfigError {
    fn from(value: std::str::ParseBoolError) -> Self {
        ConfigError::ParseBoolError(value)
    }
}

impl From<crate::common::log::Error> for ConfigError {
    fn from(value: crate::common::log::Error) -> Self {
        ConfigError::LogInitError(value)
    }
}
