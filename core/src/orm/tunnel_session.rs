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
use chrono::{DateTime, NaiveDateTime};
use entity::entities::tunnel_sessions::{ActiveModel, Column, Entity};
use sea_orm::prelude::Expr;
use sea_orm::sea_query::OnConflict;
use sea_orm::{DatabaseConnection, DbBackend, EntityTrait, ExprTrait, Iden, QueryTrait, Set};
use std::collections::HashMap;
use std::mem::take;
use std::net::IpAddr;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep_until, timeout};
use tokio::{join, select};
use tokio_util::sync::CancellationToken;
use tracing::{debug, instrument, warn};

pub enum DatabaseTunnelSessionAction {
    Update {
        timestamp: NaiveDateTime,
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
    const BUCKET_SIZE_MIN: u32 = 10;
    const BATCH_LIMIT: u16 = 500;

    assert!(SLEEP_INTERVAL_SEC < BUCKET_SIZE_MIN as u64 * 60);

    let mut next_deadline = Instant::now() + Duration::from_secs(SLEEP_INTERVAL_SEC);
    let mut update_map = HashMap::new();

    let mut database_update_tasks = JoinSet::new();

    loop {
        let should_flush = update_map.len() >= BATCH_LIMIT as usize;
        select! {
            biased;
            _ = database_update_tasks.join_next(), if !database_update_tasks.is_empty() => {}
            _ = sleep_until(if should_flush { Instant::now() } else { next_deadline }) => {
                {
                    let update_map = take(&mut update_map);
                    database_update_tasks.spawn(
                        flush_to_database(db_connection.clone(), update_map)
                    );
                }

                next_deadline = Instant::now() + Duration::from_secs(SLEEP_INTERVAL_SEC);
            }
            request = database_tunnel_session_batch_rx.recv() => {
                match request {
                    Some(DatabaseTunnelSessionAction::Update { timestamp, user_id, tunnel_client, inbound, outbound, external_connection_count_update }) => {
                        update_map
                            .entry((round_to_bucket_time(timestamp, BUCKET_SIZE_MIN as i64), user_id.clone(), tunnel_client))
                            .and_modify(|v| {
                                v.inbound = Set(v.inbound.try_as_ref().unwrap() + inbound);
                                v.outbound = Set(v.outbound.try_as_ref().unwrap() + outbound);
                                v.external_connection_count = Set(v.external_connection_count.try_as_ref().unwrap() + external_connection_count_update as i64)
                            })
                            .or_insert(ActiveModel {
                                id: Default::default(),
                                user_id: Set(Some(user_id)),
                                bucket_start: Set(round_to_bucket_time(timestamp, BUCKET_SIZE_MIN as i64)),
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
    let (tasks_res, final_res) = join!(
        timeout(Duration::from_secs(60), database_update_tasks.join_all()),
        timeout(
            Duration::from_secs(60),
            flush_to_database(db_connection.clone(), update_map)
        )
    );

    if tasks_res.is_err() {
        warn!("Database update task(s) cancelled");
    }

    if final_res.is_err() {
        warn!("Database clean up update cancelled");
    }
}

async fn flush_to_database(
    db_connection: DatabaseConnection,
    update_map: HashMap<(NaiveDateTime, String, IpAddr), ActiveModel>,
) {
    if !update_map.is_empty() {
        let table_name = Entity.to_string();
        let models: Vec<ActiveModel> = update_map.into_values().collect();

        debug!("Flushing session records to database");

        let statement = Entity::insert_many(models).on_conflict(
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
        );

        let res = statement.exec(&db_connection).await;

        if let Err(error) = res {
            warn!("Unable to update database: {:?}", error);
        }
    }
}

fn round_to_bucket_time(time: NaiveDateTime, bucket_size_minutes: i64) -> NaiveDateTime {
    //  round down
    let rounded_time_seconds =
        time.and_utc().timestamp() / (bucket_size_minutes * 60) * (bucket_size_minutes * 60);
    DateTime::from_timestamp_secs(rounded_time_seconds)
        .expect("Timestamp out of range")
        .naive_utc()
}
