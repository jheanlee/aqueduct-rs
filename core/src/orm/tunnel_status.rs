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
use crate::common::tunnel_info::TunnelInfo;
use crate::orm::error::Error;
use crate::orm::tunnel_session::round_to_bucket_time;
use crate::orm::usage_data::TimestampBucketSize;
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use entity::entities::tunnel_status::Entity;
use entity::entities::tunnel_status::{ActiveModel, Column};
use sea_orm::prelude::Expr;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    DatabaseBackend, DatabaseConnection, EntityTrait, FromQueryResult, Iden, Set, Statement,
};
use std::cmp::max;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::select;
use tokio::time::{Instant, interval, interval_at};
use tokio_util::sync::CancellationToken;
use tracing::{instrument, warn};

#[instrument(skip(tunnel_info, cancellation_token))]
pub async fn database_tunnel_status_task(
    db_connection: DatabaseConnection,
    tunnel_info: Arc<TunnelInfo>,
    cancellation_token: CancellationToken,
) {
    const BUCKET_SIZE_MIN: u32 = 10;

    let mut sample_interval = interval(Duration::from_secs(10));

    let start = Utc::now();
    let next_deadline = round_to_bucket_time(start.naive_utc(), 10)
        .checked_add_signed(TimeDelta::minutes(BUCKET_SIZE_MIN as i64))
        .expect("Timestamp out of range")
        .and_utc();

    let mut db_interval = interval_at(
        Instant::now()
            + Duration::from_secs((next_deadline.timestamp() - start.timestamp()) as u64),
        Duration::from_mins(BUCKET_SIZE_MIN as u64),
    );

    let mut active_service_count_sum = 0u64;
    let mut active_service_count_max = 0u64;
    let mut active_external_connection_count_sum = 0u64;
    let mut active_external_connection_count_max = 0u64;
    let mut sample_count = 0u64;

    loop {
        select! {
            biased;
            _ = cancellation_token.cancelled() => {
                break;
            }
            _ = db_interval.tick() => {
                if sample_count != 0 {
                    let active_model = ActiveModel {
                        id: Default::default(),
                        bucket_start: Set(
                            round_to_bucket_time(
                                Utc::now()
                                    .naive_utc()
                                    .checked_sub_signed(TimeDelta::minutes(BUCKET_SIZE_MIN as i64))
                                    .expect("Timestamp out of range"),
                                BUCKET_SIZE_MIN
                            )
                        ),
                        sample_count: Set(sample_count as i64),
                        active_service_avg: Set(
                            (active_service_count_sum as f64 / sample_count as f64).round() as i64
                        ),
                        active_service_max: Set(active_service_count_max as i64),
                        external_connection_avg: Set(
                            (active_external_connection_count_sum as f64 / sample_count as f64)
                            .round() as i64
                        ),
                        external_connection_max: Set(active_external_connection_count_max as i64)
                    };

                    let table_name = Entity.to_string();

                    let statement = Entity::insert(active_model)
                        .on_conflict(
                            OnConflict::column(Column::BucketStart)
                                .values([
                                    (
                                        Column::SampleCount,
                                        Expr::cust(format!(
                                            "\"{table_name}\".\"sample_count\" + EXCLUDED.\"sample_count\""
                                        )),
                                    ),
                                    (
                                        Column::ActiveServiceMax,
                                        Expr::cust(format!("GREATEST(\"{table_name}\".\"active_service_max\", EXCLUDED.\"active_service_max\")"))
                                    ),
                                    (
                                        Column::ActiveServiceAvg,
                                        Expr::cust(format!(
                                            "ROUND((\"{table_name}\".\"active_service_avg\" * \"{table_name}\".\"sample_count\" + EXCLUDED.\"active_service_avg\" * EXCLUDED.\"sample_count\") / (\"{table_name}\".\"sample_count\" + EXCLUDED.\"sample_count\"))"
                                        )),
                                    ),
                                    (
                                        Column::ExternalConnectionMax,
                                        Expr::cust(format!(
                                            "GREATEST(\"{table_name}\".\"external_connection_max\", EXCLUDED.\"external_connection_max\")"
                                        )),
                                    ),
                                    (
                                        Column::ExternalConnectionAvg,
                                        Expr::cust(format!(
                                            "ROUND((\"{table_name}\".\"external_connection_avg\" * \"{table_name}\".\"sample_count\" + EXCLUDED.\"external_connection_avg\" * EXCLUDED.\"sample_count\") / (\"{table_name}\".\"sample_count\" + EXCLUDED.\"sample_count\"))"
                                        )),
                                    ),
                                ])
                                .clone()
                        );

                    sample_count = 0;
                    active_service_count_sum = 0;
                    active_service_count_max = 0;
                    active_external_connection_count_sum = 0;
                    active_external_connection_count_max = 0;

                    let res = statement.exec(&db_connection).await;

                    if let Err(error) = res {
                        warn!("Unable to update database with tunnel status: {:?}", error);
                    }
                }

            }
            _ = sample_interval.tick() => {
                let current_service_count = tunnel_info.active_service_count.load(Ordering::Relaxed);
                let current_external_connection_count = tunnel_info.active_external_connection_count.load(Ordering::Relaxed);

                sample_count += 1;
                active_service_count_sum += current_service_count;
                active_service_count_max = max(active_service_count_max, current_service_count);
                active_external_connection_count_sum += current_external_connection_count;
                active_external_connection_count_max = max(active_external_connection_count_max, current_external_connection_count);
            }
        }
    }
}

pub async fn get_tunnel_status_data(
    db_connection: DatabaseConnection,
    timestamp_bucket_size: TimestampBucketSize,
    query_start: NaiveDateTime,
    query_end: NaiveDateTime,
) -> Result<Vec<TunnelStatusModel>, Error> {
    //  bucket size handling
    let bucket_size_str = timestamp_bucket_size.as_str();

    let bucket_size_duration = timestamp_bucket_size.as_duration();

    let trunc_size = timestamp_bucket_size.get_timestamp_bucket_size_str();

    //  round query timestamps
    //  start time rounded up
    let rounded_query_start_seconds =
        (query_start.and_utc() + bucket_size_duration - chrono::Duration::seconds(1)).timestamp()
            / (bucket_size_duration.as_seconds_f64() as i64)
            * (bucket_size_duration.as_seconds_f64() as i64);
    let rounded_query_start = DateTime::from_timestamp_secs(rounded_query_start_seconds)
        .ok_or(Error::BadRequest)?
        .naive_utc();

    //  end time rounded down
    let rounded_query_end_seconds = query_end.and_utc().timestamp()
        / (bucket_size_duration.as_seconds_f64() as i64)
        * (bucket_size_duration.as_seconds_f64() as i64);
    let rounded_query_end = DateTime::from_timestamp_secs(rounded_query_end_seconds)
        .ok_or(Error::BadRequest)?
        .naive_utc();

    //  db operation
    const STATEMENT: &str = "WITH buckets AS (
    SELECT generate_series(
            date_trunc($1, $2::timestamp),
            date_trunc($1, $3::timestamp),
            $4::interval
        ) AS bucket
),
status AS (
    SELECT
        date_trunc($1, bucket_start) AS bucket, active_service_avg,
        active_service_max, external_connection_avg, external_connection_max
    FROM tunnel_status
    WHERE bucket_start >= $2
        AND bucket_start <= $3
)
SELECT
    buckets.bucket,
    coalesce(status.active_service_avg, 0) AS active_service_avg,
    coalesce(status.active_service_max, 0) AS active_service_max,
    coalesce(status.external_connection_avg, 0) AS external_connection_avg,
    coalesce(status.external_connection_max, 0) AS external_connection_max
FROM buckets
LEFT JOIN status USING (bucket)
ORDER BY buckets.bucket
";

    let res = TunnelStatusModel::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        STATEMENT,
        vec![
            trunc_size.into(),
            rounded_query_start.into(),
            rounded_query_end.into(),
            bucket_size_str.into(),
        ],
    ))
    .all(&db_connection)
    .await;

    Ok(res?)
}

#[derive(Debug, FromQueryResult)]
pub struct TunnelStatusModel {
    pub bucket: NaiveDateTime,
    pub active_service_avg: i64,
    pub active_service_max: i64,
    pub external_connection_avg: i64,
    pub external_connection_max: i64,
}
