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
use jsonwebtoken::{DecodingKey, EncodingKey};

#[derive(Clone)]
pub struct JwtKeyPair {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

pub async fn init_jwt_keys(
    private_key_path: String,
    public_key_path: String,
) -> Result<JwtKeyPair, Error> {
    let priv_bytes = tokio::fs::read(private_key_path).await?;

    let encoding_key = EncodingKey::from_rsa_pem(priv_bytes.as_slice())
        .or_else(|_| EncodingKey::from_ec_pem(priv_bytes.as_slice()))
        .or_else(|_| EncodingKey::from_ed_pem(priv_bytes.as_slice()))?;

    let pub_bytes = tokio::fs::read(public_key_path).await?;

    let decoding_key = DecodingKey::from_rsa_pem(pub_bytes.as_slice())
        .or_else(|_| DecodingKey::from_ec_pem(pub_bytes.as_slice()))
        .or_else(|_| DecodingKey::from_ed_pem(pub_bytes.as_slice()))?;

    Ok(JwtKeyPair {
        encoding_key,
        decoding_key,
    })
}
