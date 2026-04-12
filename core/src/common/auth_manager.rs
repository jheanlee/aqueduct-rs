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
use crate::common::error::Error;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{Salt, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use tokio::sync::Semaphore;
use tokio::task::spawn_blocking;

pub struct AuthManager {
    semaphore: Semaphore,
}

impl AuthManager {
    pub fn new() -> Self {
        AuthManager {
            semaphore: Semaphore::new(8),
        }
    }

    pub async fn verify_password(
        &self,
        password: String,
        encoded_hash: String,
    ) -> Result<bool, Error> {
        let _permit = self.semaphore.acquire().await?;
        spawn_blocking(move || {
            PasswordUtils::verify_password(password.as_bytes(), encoded_hash.as_str())
        })
        .await?
    }

    pub async fn hash_password(&self, password: String) -> Result<(String, String), Error> {
        let _permit = self.semaphore.acquire().await?;
        let salt = SaltString::generate(OsRng);
        let (hash, salt) = spawn_blocking(move || {
            (
                PasswordUtils::hash_password(password.as_bytes(), salt.as_str()),
                salt.to_string(),
            )
        })
        .await?;
        Ok((hash?, salt))
    }
}

struct PasswordUtils;

impl PasswordUtils {
    fn hash_password(password: &[u8], encoded_salt: &str) -> Result<String, Error> {
        Ok(Argon2::default()
            .hash_password(password, Salt::from_b64(encoded_salt)?)?
            .to_string())
    }

    fn verify_password(password: &[u8], hash: &str) -> Result<bool, Error> {
        Ok(Argon2::default()
            .verify_password(password, &PasswordHash::new(hash)?)
            .is_ok())
    }
}
