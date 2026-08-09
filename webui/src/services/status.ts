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

import { cronFetcher } from "@/services/fetcher.ts";

export const get_system_status = async () => {
  try {
    const res = await cronFetcher.get<{
      cpu_usage: number;
      used_memory: number;
      total_memory: number;
      process_cpu_usage: number;
      process_memory: number;
      process_fd_count: number | null;
    }>("/api/status/system");

    res.data.process_fd_count = res.data.process_fd_count
      ? res.data.process_fd_count
      : null;

    return res.data;
  } catch {
    return {
      cpu_usage: null,
      used_memory: null,
      total_memory: null,
      process_cpu_usage: null,
      process_memory: null,
      process_fd_count: null,
    };
  }
};

export const get_tunnel_status = async () => {
  try {
    const res = await cronFetcher.get<{
      uptime: number;
      active_service_count: number;
      active_external_connection_count: number;
    }>("/api/status/tunnel");

    return res.data;
  } catch {
    return {
      uptime: null,
      active_service_count: null,
      active_external_connection_count: null,
    };
  }
};
