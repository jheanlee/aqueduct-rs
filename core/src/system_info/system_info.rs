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
use crate::system_info::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;
use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::time::sleep;
use tokio_util::future::FutureExt;
use tokio_util::sync::CancellationToken;
use tracing::error;

pub struct SystemInfo {
    pub cpu_usage: AtomicU32, // f32
    pub used_memory: AtomicU64,
    pub total_memory: AtomicU64,
    pub process_cpu_usage: AtomicU32, // f32
    pub process_memory: AtomicU64,
    pub process_fd_count: AtomicUsize,
}

pub async fn system_info_hot(
    system_info: Arc<SystemInfo>,
    cancellation_token: CancellationToken,
) -> Result<(), Error> {
    let mut sys = tokio::task::spawn_blocking(|| System::new_all())
        .await
        .inspect_err(|error| error!("System info thread panicked: {:?}", error))?;
    let pid_arr = [Pid::from(std::process::id() as usize)];

    loop {
        sys = tokio::task::spawn_blocking(move || {
            sys.refresh_cpu_usage();
            sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
            sys.refresh_processes_specifics(
                ProcessesToUpdate::Some(&pid_arr),
                false,
                ProcessRefreshKind::nothing().with_cpu().with_memory(),
            );
            sys
        })
        .with_cancellation_token_owned(cancellation_token.clone())
        .await
        .ok_or(Error::Empty)?
        .inspect_err(|error| error!("System info thread panicked: {:?}", error))?;

        if let Some(process) = sys.process(pid_arr[0]) {
            system_info
                .cpu_usage
                .store(sys.global_cpu_usage().to_bits(), Ordering::Relaxed);
            system_info
                .used_memory
                .store(sys.used_memory(), Ordering::Relaxed);
            system_info
                .total_memory
                .store(sys.total_memory(), Ordering::Relaxed);

            system_info
                .process_cpu_usage
                .store(process.cpu_usage().to_bits(), Ordering::Relaxed);
            system_info
                .process_memory
                .store(process.memory(), Ordering::Relaxed);
        }

        sleep(Duration::from_secs(10))
            .with_cancellation_token_owned(cancellation_token.clone())
            .await;
    }
}

pub async fn system_info_cold(
    system_info: Arc<SystemInfo>,
    cancellation_token: CancellationToken,
) -> Result<(), Error> {
    let sys = tokio::task::spawn_blocking(|| Arc::new(std::sync::Mutex::new(System::new_all())))
        .await
        .inspect_err(|error| error!("System info thread panicked: {:?}", error))?;

    loop {
        let sys_clone = sys.clone();
        let fd_count = tokio::task::spawn_blocking(move || get_fd_count(sys_clone))
            .with_cancellation_token_owned(cancellation_token.clone())
            .await
            .ok_or(Error::Empty)?
            .inspect_err(|error| error!("System info thread panicked: {:?}", error))?
            .unwrap_or(0);

        system_info
            .process_fd_count
            .store(fd_count, Ordering::Relaxed);

        sleep(Duration::from_secs(5))
            .with_cancellation_token_owned(cancellation_token.clone())
            .await;
    }
}

pub fn get_fd_count(sys: Arc<std::sync::Mutex<System>>) -> Option<usize> {
    let pid = Pid::from(std::process::id() as usize);
    let mut sys = sys.lock().expect("sys should not be locked");

    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::nothing(),
    );

    sys.process(pid)
        .expect("The current process must exist")
        .open_files()
}
