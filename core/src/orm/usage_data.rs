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
use chrono::{DateTime, Duration, NaiveDateTime};
use sea_orm::{DatabaseBackend, DatabaseConnection, FromQueryResult, Statement};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimestampBucketSize {
    TenMinutes,
    Hourly,
    Daily,
    Weekly,
}

impl TimestampBucketSize {
    pub fn as_str(&self) -> &str {
        match self {
            TimestampBucketSize::TenMinutes => "10 minutes",
            TimestampBucketSize::Hourly => "1 hour",
            TimestampBucketSize::Daily => "1 day",
            TimestampBucketSize::Weekly => "7 days",
        }
    }

    pub fn as_duration(&self) -> Duration {
        match self {
            TimestampBucketSize::TenMinutes => Duration::minutes(10),
            TimestampBucketSize::Hourly => Duration::hours(1),
            TimestampBucketSize::Daily => Duration::days(1),
            TimestampBucketSize::Weekly => Duration::days(7),
        }
    }

    pub fn get_timestamp_bucket_size_str(&self) -> &str {
        match self {
            TimestampBucketSize::TenMinutes => "minute",
            TimestampBucketSize::Hourly => "hour",
            TimestampBucketSize::Daily => "day",
            TimestampBucketSize::Weekly => "week",
        }
    }
}

#[derive(Debug, FromQueryResult)]
pub struct TunnelUsagePoint {
    pub bucket: NaiveDateTime,
    pub inbound: i64,
    pub outbound: i64,
    pub external_connection_count: i64,
}

pub async fn get_tunnel_usage_data(
    db_connection: DatabaseConnection,
    user_id: Option<String>,
    timestamp_bucket_size: TimestampBucketSize,
    query_start: NaiveDateTime,
    query_end: NaiveDateTime,
) -> Result<Vec<TunnelUsagePoint>, Error> {
    //  bucket size handling
    let bucket_size_str = timestamp_bucket_size.as_str();

    let bucket_size_duration = timestamp_bucket_size.as_duration();

    let trunc_size = timestamp_bucket_size.get_timestamp_bucket_size_str();

    //  round query timestamps
    //  start time rounded up
    let rounded_query_start_seconds =
        (query_start.and_utc() + bucket_size_duration - Duration::seconds(1)).timestamp()
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
    const STATEMENT_BY_USER: &str = "WITH buckets AS (
    SELECT generate_series(
            date_trunc($4, $2::timestamp),
            date_trunc($4, $3::timestamp),
            $5::interval
        ) AS bucket
),
usage AS (
    SELECT
        date_trunc($4, bucket_start) AS bucket,
        sum(inbound)::bigint AS inbound,
        sum(outbound)::bigint AS outbound,
        sum(external_connection_count)::bigint AS external_connection_count
    FROM tunnel_sessions
    WHERE user_id = $1
        AND bucket_start >= $2
        AND bucket_start <= $3
    GROUP BY bucket
)
SELECT buckets.bucket,
       coalesce(usage.inbound, 0) AS inbound,
       coalesce(usage.outbound, 0) AS outbound,
       coalesce(usage.external_connection_count, 0) AS external_connection_count
FROM buckets
LEFT JOIN usage USING (bucket)
ORDER BY buckets.bucket;";
    const STATEMENT_OVERALL: &str = "WITH buckets AS (
    SELECT generate_series(
            date_trunc($3, $1::timestamp),
            date_trunc($3, $2::timestamp),
            $4::interval
        ) AS bucket
),
usage AS (
    SELECT
        date_trunc($3, bucket_start) AS bucket,
        sum(inbound)::bigint AS inbound,
        sum(outbound)::bigint AS outbound,
        sum(external_connection_count)::bigint AS external_connection_count
    FROM tunnel_sessions
    WHERE bucket_start >= $1
        AND bucket_start <= $2
    GROUP BY bucket
)
SELECT buckets.bucket,
       coalesce(usage.inbound, 0) AS inbound,
       coalesce(usage.outbound, 0) AS outbound,
       coalesce(usage.external_connection_count, 0) AS external_connection_count
FROM buckets
LEFT JOIN usage USING (bucket)
ORDER BY buckets.bucket;";

    if let Some(user_id) = user_id {
        let res = TunnelUsagePoint::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            STATEMENT_BY_USER,
            vec![
                user_id.into(),
                rounded_query_start.into(),
                rounded_query_end.into(),
                trunc_size.into(),
                bucket_size_str.into(),
            ],
        ))
        .all(&db_connection)
        .await?;
        Ok(res)
    } else {
        let res = TunnelUsagePoint::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            STATEMENT_OVERALL,
            vec![
                rounded_query_start.into(),
                rounded_query_end.into(),
                trunc_size.into(),
                bucket_size_str.into(),
            ],
        ))
        .all(&db_connection)
        .await?;
        Ok(res)
    }
}
