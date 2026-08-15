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
import { isAxiosError } from "axios";
import type { TunnelChartResolution } from "@/services/tunnel/usage.ts";

export const getRealtimeSystemStatus = async () => {
  try {
    const res = await cronFetcher.get<{
      cpu_usage: number;
      used_memory: number;
      total_memory: number;
      process_cpu_usage: number;
      process_memory: number;
      process_fd_count: number | null;
    }>("/api/status/realtime/system");

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

export const getRealtimeTunnelStatus = async () => {
  try {
    const res = await cronFetcher.get<{
      uptime: number;
      active_service_count: number;
      active_external_connection_count: number;
    }>("/api/status/realtime/tunnel");

    return res.data;
  } catch {
    return {
      uptime: null,
      active_service_count: null,
      active_external_connection_count: null,
    };
  }
};

export interface StatusDataPoint {
  timestamp: Date;
  activeServiceAvg: number;
  activeServiceMax: number;
  externalConnectionAvg: number;
  externalConnectionMax: number;
}

export const getTunnelStatus = async (
  start: Date,
  end: Date,
  resolution: TunnelChartResolution,
) => {
  //  date handling
  let query_start = start.toISOString();
  query_start = query_start.substring(0, query_start.length - 1);
  let query_end = end.toISOString();
  query_end = query_end.substring(0, query_end.length - 1);

  try {
    const res = await cronFetcher.get<{
      status_data_points: {
        bucket: number;
        active_service_avg: number;
        active_service_max: number;
        external_connection_avg: number;
        external_connection_max: number;
      }[];
    }>("/api/status/tunnel", {
      params: {
        resolution,
        query_start,
        query_end,
      },
    });

    return res.data.status_data_points.map((value) => {
      return {
        timestamp: new Date(value.bucket * 1000),
        activeServiceAvg: value.active_service_avg,
        activeServiceMax: value.active_service_max,
        externalConnectionAvg: value.external_connection_avg,
        externalConnectionMax: value.external_connection_max,
      } as StatusDataPoint;
    });
  } catch (error) {
    console.log(`Failed to get historical tunnel status: ${error}`);
    if (isAxiosError(error)) {
      return error.status || 500;
    } else {
      return 500;
    }
  }
};
