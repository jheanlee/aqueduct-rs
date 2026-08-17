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
use crate::orm::blacklist::get_blacklist;
use crate::orm::whitelist::get_whitelist;
use ip_network_table::IpNetworkTable;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn update_access_ip_tables(
    db_connection: &DatabaseConnection,
    whitelist_table: Arc<RwLock<IpNetworkTable<()>>>,
    blacklist_table: Arc<RwLock<IpNetworkTable<()>>>,
) -> Result<(), crate::orm::error::Error> {
    {
        let mut whitelist_table = whitelist_table.write().await;
        *whitelist_table = IpNetworkTable::new();
        for entry in get_whitelist(db_connection).await? {
            whitelist_table.insert(entry, ());
        }
    }
    {
        let mut blacklist_table = blacklist_table.write().await;
        *blacklist_table = IpNetworkTable::new();
        for entry in get_blacklist(db_connection).await? {
            blacklist_table.insert(entry, ());
        }
    }
    Ok(())
}
