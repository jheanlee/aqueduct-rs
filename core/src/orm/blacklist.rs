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
use crate::orm::error::Error;
use entity::entities::ip_blacklist::{ActiveModel, Entity};
use ip_network::IpNetwork;
use sea_orm::{
    ActiveModelTrait, DbConn, DbErr, EntityTrait, IntoActiveModel, RuntimeErr, Set, sqlx,
};
use serde::Deserialize;
use std::net::IpAddr;
use std::str::FromStr;

pub async fn get_blacklist(db_connection: DbConn) -> Result<Vec<IpNetwork>, Error> {
    let res: Vec<IpNetwork> = Entity::find()
        .all(&db_connection)
        .await?
        .into_iter()
        .filter_map(|v| {
            let mask = match v.network.ip() {
                IpAddr::V4(addr) => {
                    let mask: u32 = addr.into();
                    mask.count_ones() as u8
                }
                IpAddr::V6(addr) => {
                    let mask: u128 = addr.into();
                    mask.count_ones() as u8
                }
            };
            IpNetwork::new(v.network.ip(), mask).ok()
        })
        .collect();
    Ok(res)
}

#[derive(Deserialize)]
pub struct BlacklistEntry {
    pub network: String,
    pub notes: String,
}
pub async fn add_blacklist(
    db_connection: DbConn,
    values: Vec<BlacklistEntry>,
) -> Result<(), Error> {
    let models: Vec<ActiveModel> = values
        .iter()
        .map(|value| {
            Ok::<ActiveModel, Error>(ActiveModel {
                id: Default::default(),
                network: Set(
                    sea_orm::prelude::IpNetwork::from_str(value.network.as_str())
                        .map_err(|_| Error::BadRequest)?,
                ),
                notes: Set(value.notes.clone()),
            })
        })
        .collect::<Result<Vec<ActiveModel>, Error>>()?;

    let res = Entity::insert_many(models).exec(&db_connection).await;
    match res {
        Ok(_) => Ok(()),
        Err(DbErr::Query(RuntimeErr::SqlxError(error))) => match error.as_ref() {
            sqlx::Error::Database(db_err) => match db_err.code() {
                Some(code) if code == "23P01" => Err(Error::BadRequest)?,
                Some(code) if code == "23505" => Err(Error::BadRequest)?,
                Some(code) if code == "23503" => Err(Error::BadRequest)?,
                _ => Err(DbErr::Query(RuntimeErr::SqlxError(error)))?,
            },
            _ => Err(DbErr::Query(RuntimeErr::SqlxError(error)))?,
        },
        Err(error) => Err(error)?,
    }
}

pub async fn delete_blacklist(db_connection: DbConn, id: i32) -> Result<(), Error> {
    Entity::find_by_id(id)
        .one(&db_connection)
        .await?
        .ok_or(Error::NotFound)?
        .into_active_model()
        .delete(&db_connection)
        .await?;
    Ok(())
}
