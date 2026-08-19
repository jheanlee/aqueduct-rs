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
use tokio::select;
use tokio::signal::unix::{SignalKind, signal};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

pub async fn signal_handler(cancellation_token: CancellationToken) {
    let mut sighup = signal(SignalKind::hangup()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigquit = signal(SignalKind::quit()).unwrap();
    let mut sigpipe = signal(SignalKind::pipe()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    loop {
        select! {
            biased;
            _ = cancellation_token.cancelled() => {
                break;
            }
            _ = sighup.recv() => {
                warn!("Received SIGHUP");
                cancellation_token.cancel();
                break;
            }
            _ = sigint.recv() => {
                info!("Received SIGINT");
                cancellation_token.cancel();
                break;
            }
            _ = sigquit.recv() => {
                warn!("Received SIGQUIT");
                cancellation_token.cancel();
                break;
            }
            _ = sigpipe.recv() => {}
            _ = sigterm.recv() => {
                info!("Received SIGTERM");
                cancellation_token.cancel();
                break;
            }
        }
    }
}
