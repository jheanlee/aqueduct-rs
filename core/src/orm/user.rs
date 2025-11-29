use openssl::base64;
use openssl::sha::{Sha256};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use entity::entities::user::{Column, Entity};
use crate::orm::error::DbError;

fn password_handler(encoded_salt: String, password: &String) -> String {
  let salt = base64::decode_block(encoded_salt.as_str()).unwrap_or_default();
  let mut hasher = Sha256::new();
  hasher.update(salt.as_slice());
  hasher.update(password.as_bytes());
  let hash = hasher.finish();
  base64::encode_block(&hash)
}

pub async fn authenticate_user(db_connection: &DatabaseConnection, username: &String, password: &String) -> Result<bool, DbError> {
  let user = Entity::find()
    .filter(Column::Username.eq(username))
    .one(db_connection).await?.ok_or(DbError::NotFound)?;
  
  Ok(password_handler(user.salt, password) == user.hashed_password)
}