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
use crate::common::model::Shared;
use crate::orm::error::Error;
use entity::entities::{tunnel_sessions, tunnel_users};
use sea_orm::{ColumnTrait, EntityTrait};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TunnelUserData {
    pub id: String,
    pub username: String,
    pub inbound: i64,
    pub outbound: i64,
}

pub async fn get_tunnel_user_data(shared: Shared, id: &str) -> Result<TunnelUserData, Error> {
    let user = tunnel_users::Entity::find_by_id(id)
        .one(&shared.db_connection)
        .await?
        .ok_or(Error::NotFound)?;
    let sessions = tunnel_sessions::Entity::find()
        .has_related(tunnel_users::Entity, tunnel_users::Column::Id.eq(id))
        .all(&shared.db_connection)
        .await?;

    let mut inbound_usage = 0i64;
    let mut outbound_usage = 0i64;
    for model in sessions {
        inbound_usage += model.inbound;
        outbound_usage += model.outbound;
    }

    Ok(TunnelUserData {
        id: user.id,
        username: user.username,
        inbound: inbound_usage,
        outbound: outbound_usage,
    })
}
