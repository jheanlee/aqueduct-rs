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
use crate::api::error::Error;
use jsonwebtoken::Algorithm::RS256;
use jsonwebtoken::{EncodingKey, Header, get_current_timestamp};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub sid: String,
}
pub fn generate_access_token(
    sub: String,
    sid: String,
    encoding_key: &EncodingKey,
) -> Result<String, Error> {
    let claims = AccessTokenClaims {
        sub,
        iat: get_current_timestamp(),
        exp: get_current_timestamp() + 300,
        sid,
    };
    Ok(jsonwebtoken::encode(
        &Header::new(RS256),
        &claims,
        encoding_key,
    )?)
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub jti: String,
    pub sid: String,
}
pub fn generate_refresh_token(
    sub: String,
    encoding_key: &EncodingKey,
) -> Result<(String, String, String), Error> {
    let claims = RefreshTokenClaims {
        sub,
        iat: get_current_timestamp(),
        exp: get_current_timestamp() + 24 * 60 * 60,
        jti: nanoid!(),
        sid: nanoid!(),
    };
    Ok((
        claims.jti.clone(),
        claims.sid.clone(),
        jsonwebtoken::encode(&Header::new(RS256), &claims, encoding_key)?,
    ))
}
