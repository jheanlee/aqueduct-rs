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
use crate::core::tunnel::model::TunnelStatus;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::time::{Instant, sleep};
use tokio_util::sync::CancellationToken;

pub async fn pending_client_cleaner(
    cancellation_token: CancellationToken,
    tunnel_status: Arc<TunnelStatus>,
) {
    loop {
        select! {
            biased;
            _ = cancellation_token.cancelled() => {
                return;
            }
            _ = sleep(Duration::from_secs(60)) => {
                let deadline = Instant::now() - Duration::from_secs(60);
                tunnel_status.pending_external_clients.retain(|_, value| value.timestamp > deadline);
            }
        }
    }
}
