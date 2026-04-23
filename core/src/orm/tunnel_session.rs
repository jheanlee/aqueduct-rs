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
use crate::common::log::{Level, log};
use crate::common::model::Shared;
use entity::entities::tunnel_sessions::{ActiveModel, Column, Entity};
use sea_orm::sea_query::{CaseStatement, Expr, Query};
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set};
use sea_orm::{EntityTrait, ExprTrait};
use std::collections::HashMap;
use std::mem::take;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep_until};
use tokio_util::sync::CancellationToken;

pub enum DatabaseTunnelSessionAction {
    Insert {
        id: String,
        user_id: String,
        tunnel_client: SocketAddr,
        external_client: SocketAddr,
    },
    Update {
        id: String,
        inbound: i64,
        outbound: i64,
        closed: bool,
    },
}

pub async fn database_tunnel_session_batch_thread(
    shared: Shared,
    mut database_tunnel_session_batch_rx: mpsc::Receiver<DatabaseTunnelSessionAction>,
    cancellation_token: CancellationToken,
) {
    const SLEEP_INTERVAL: u64 = 60;
    const BATCH_LIMIT: u16 = 500;

    let mut next_deadline = Instant::now() + Duration::from_secs(SLEEP_INTERVAL);
    let mut insert_list = Vec::new();

    let mut update_map = HashMap::new();

    loop {
        let should_flush = insert_list.len() + update_map.len() >= BATCH_LIMIT as usize;
        select! {
            biased;
            request = database_tunnel_session_batch_rx.recv() => {
                match request {
                    Some(DatabaseTunnelSessionAction::Insert { id, user_id, tunnel_client, external_client }) => {
                        insert_list.push(ActiveModel {
                            id: Set(id),
                            user_id: Set(user_id),
                            tunnel_client: Set(tunnel_client.to_string()),
                            external_client: Set(external_client.to_string()),
                            inbound: Set(0),
                            outbound: Set(0),
                            start_time: Set(chrono::Utc::now()),
                            end_time: Set(None),
                        });
                    }
                    Some(DatabaseTunnelSessionAction::Update { id, inbound, outbound, closed }) => {
                        let entry = update_map.entry(id).or_insert((0, 0, None));
                        entry.0 += inbound;
                        entry.1 += outbound;
                        if closed {
                            entry.2 = Some(chrono::Utc::now());
                        }
                    }
                    None => break,
                }
            }
            _ = cancellation_token.cancelled() => { break; }
            _ = sleep_until(if should_flush { Instant::now() } else { next_deadline }) => {
                //  bulk insert
                if !insert_list.is_empty() {
                    if let Err(error) =
                        Entity::insert_many(take(&mut insert_list)).exec(&shared.db_connection).await {
                        log(
                            Level::Warning,
                            format!("Unable to insert into database: {:?}", error).as_str(),
                            "orm::tunnel_session::database_tunnel_session_batch_thread"
                        )
                        .await;
                    }
                }

                //  bulk update
                if update_map.len() != 0 {
                    let mut ids = Vec::new();
                    let mut inbound_expr = CaseStatement::new();
                    let mut outbound_expr = CaseStatement::new();
                    let mut end_time_expr = CaseStatement::new();
                    let mut has_end_time = false;

                    for (id, (inbound, outbound, end_time)) in update_map {
                        let id_expr = Expr::col(Column::Id).eq(id.clone());
                        ids.push(id);

                        inbound_expr = inbound_expr.case(id_expr.clone(), Expr::col(Column::Inbound).add(inbound));
                        outbound_expr = outbound_expr.case(id_expr.clone(), Expr::col(Column::Outbound).add(outbound));

                        if end_time.is_some() {
                            has_end_time = true;
                            end_time_expr = end_time_expr.case(id_expr, Expr::value(end_time));
                        }
                    }

                    let mut query = Query::update();
                    query.table(Entity);

                    let mut values = vec![
                        (Column::Inbound, inbound_expr.finally(Expr::col(Column::Inbound)).into()),
                        (Column::Outbound, outbound_expr.finally(Expr::col(Column::Outbound)).into())
                    ];

                    if has_end_time {
                        values.push(
                            (Column::EndTime, end_time_expr.finally(Expr::col(Column::EndTime)).into())
                        );
                    }

                    query.values(values).and_where(Expr::col(Column::Id).is_in(ids));

                    if let Err(error) = shared.db_connection.execute(&query).await {
                        log(
                            Level::Warning,
                            format!("Unable to update database: {:?}", error).as_str(),
                            "orm::tunnel_session::database_tunnel_session_batch_thread"
                        )
                        .await;
                    }

                    update_map = HashMap::new();
                }
                next_deadline = Instant::now() + Duration::from_secs(SLEEP_INTERVAL);
            }
        }
    }

    //  clean up
    //  bulk insert
    if !insert_list.is_empty() {
        if let Err(error) = Entity::insert_many(insert_list)
            .exec(&shared.db_connection)
            .await
        {
            log(
                Level::Warning,
                format!("Unable to insert into database: {:?}", error).as_str(),
                "orm::tunnel_session::database_tunnel_session_batch_thread",
            )
            .await;
        }
    }

    //  bulk update
    let mut ids = Vec::new();
    let mut inbound_expr = CaseStatement::new();
    let mut outbound_expr = CaseStatement::new();
    let mut end_time_expr = CaseStatement::new();
    let mut has_end_time = false;

    for (id, (inbound, outbound, end_time)) in update_map {
        let id_expr = Expr::col(Column::Id).eq(id.clone());
        ids.push(id);

        inbound_expr = inbound_expr.case(id_expr.clone(), Expr::col(Column::Inbound).add(inbound));
        outbound_expr =
            outbound_expr.case(id_expr.clone(), Expr::col(Column::Outbound).add(outbound));

        if end_time.is_some() {
            has_end_time = true;
            end_time_expr = end_time_expr.case(id_expr, Expr::value(end_time));
        }
    }

    let mut query = Query::update();
    query.table(Entity);

    let mut values = vec![
        (
            Column::Inbound,
            inbound_expr.finally(Expr::col(Column::Inbound)).into(),
        ),
        (
            Column::Outbound,
            outbound_expr.finally(Expr::col(Column::Outbound)).into(),
        ),
    ];

    if has_end_time {
        values.push((
            Column::EndTime,
            end_time_expr.finally(Expr::col(Column::EndTime)).into(),
        ));
    }

    query
        .values(values)
        .and_where(Expr::col(Column::Id).is_in(ids));

    if let Err(error) = shared.db_connection.execute(&query).await {
        log(
            Level::Warning,
            format!("Unable to update database: {:?}", error).as_str(),
            "orm::tunnel_session::database_tunnel_session_batch_thread",
        )
        .await;
    }
}
