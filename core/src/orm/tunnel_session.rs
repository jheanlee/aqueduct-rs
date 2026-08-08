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
use chrono::{NaiveDateTime, Timelike, Utc};
use entity::entities::tunnel_sessions::{ActiveModel, Column, Entity};
use sea_orm::prelude::Expr;
use sea_orm::sea_query::OnConflict;
use sea_orm::{DatabaseConnection, EntityTrait, ExprTrait, Iden, Set};
use std::collections::HashMap;
use std::mem::take;
use std::net::IpAddr;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep_until};
use tokio_util::sync::CancellationToken;
use tracing::{instrument, warn};

pub enum DatabaseTunnelSessionAction {
    Update {
        user_id: String,
        tunnel_client: IpAddr,
        inbound: i64,
        outbound: i64,
        external_connection_count_update: bool,
    },
}

#[instrument(skip(database_tunnel_session_batch_rx, cancellation_token))]
pub async fn database_tunnel_session_batch_task(
    db_connection: DatabaseConnection,
    mut database_tunnel_session_batch_rx: mpsc::Receiver<DatabaseTunnelSessionAction>,
    cancellation_token: CancellationToken,
) {
    const SLEEP_INTERVAL_SEC: u64 = 60;
    const BUCKET_LIMIT_MIN: u32 = 10;
    const BATCH_LIMIT: u16 = 500;

    assert!(SLEEP_INTERVAL_SEC < BUCKET_LIMIT_MIN as u64 * 60);

    let mut next_deadline = Instant::now() + Duration::from_secs(SLEEP_INTERVAL_SEC);
    let mut update_map = HashMap::new();

    let mut database_update_threads = JoinSet::new();

    loop {
        let should_flush = update_map.len() >= BATCH_LIMIT as usize;
        select! {
            biased;
            _ = database_update_threads.join_next(), if !database_update_threads.is_empty() => {}
            _ = sleep_until(if should_flush { Instant::now() } else { next_deadline }) => {
                {
                    let update_map = take(&mut update_map);
                    database_update_threads.spawn(
                        flush_to_database(db_connection.clone(), update_map)
                    );
                }

                next_deadline = Instant::now() + Duration::from_secs(SLEEP_INTERVAL_SEC);
            }
            request = database_tunnel_session_batch_rx.recv() => {
                match request {
                    Some(DatabaseTunnelSessionAction::Update { user_id, tunnel_client, inbound, outbound, external_connection_count_update }) => {
                        update_map
                            .entry(user_id.clone() + tunnel_client.to_string().as_str())
                            .and_modify(|v| {
                                v.inbound = Set(v.inbound.clone().unwrap() + inbound);
                                v.outbound = Set(v.outbound.clone().unwrap() + outbound);
                                v.external_connection_count = Set(v.external_connection_count.clone().unwrap() + external_connection_count_update as i64)
                            })
                            .or_insert(ActiveModel {
                                id: Default::default(),
                                user_id: Set(Some(user_id)),
                                bucket_start: Set(bucket_time(BUCKET_LIMIT_MIN)),
                                tunnel_client: Set(sea_orm::prelude::IpNetwork::new(tunnel_client, 32).unwrap()),
                                inbound: Set(inbound),
                                outbound: Set(outbound),
                                external_connection_count: Set(external_connection_count_update as i64),
                            });
                    }
                    None => break,
                }
            }
            _ = cancellation_token.cancelled() => { break; }
        }
    }

    //  clean up
    flush_to_database(db_connection.clone(), update_map).await;
}

async fn flush_to_database(
    db_connection: DatabaseConnection,
    update_map: HashMap<String, ActiveModel>,
) {
    if !update_map.is_empty() {
        let table_name = Entity.to_string();
        let models: Vec<ActiveModel> = update_map.into_values().collect();

        let res = Entity::insert_many(models)
            .on_conflict(
                OnConflict::columns([Column::UserId, Column::TunnelClient, Column::BucketStart])
                    .values([
                        (
                            Column::Inbound,
                            Expr::cust(format!("\"{table_name}\".\"inbound\""))
                                .add(Expr::cust("EXCLUDED.\"inbound\"")),
                        ),
                        (
                            Column::Outbound,
                            Expr::cust(format!("\"{table_name}\".\"outbound\""))
                                .add(Expr::cust("EXCLUDED.\"outbound\"")),
                        ),
                        (
                            Column::ExternalConnectionCount,
                            Expr::cust(format!("\"{table_name}\".\"external_connection_count\""))
                                .add(Expr::cust("EXCLUDED.\"external_connection_count\"")),
                        ),
                    ])
                    .clone(),
            )
            .exec(&db_connection)
            .await;

        if let Err(error) = res {
            warn!("Unable to update database: {:?}", error);
        }
    }
}

fn bucket_time(bucket_limit_min: u32) -> NaiveDateTime {
    let now = Utc::now();
    let bucket_minute = now.minute() / bucket_limit_min * bucket_limit_min;
    let bucket_utc = now
        .with_minute(bucket_minute)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    bucket_utc.naive_utc()
}
