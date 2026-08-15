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

export interface UsageDataPoint {
  timestamp: Date;
  inbound: number;
  outbound: number;
  connections: number;
}

export type TunnelChartResolution =
  | "ten_minutes"
  | "hourly"
  | "daily"
  | "weekly";

export const getTunnelUsage = async (
  user_id: string | null,
  start: Date,
  end: Date,
  resolution: TunnelChartResolution,
) => {
  const uri =
    user_id === null ? "/api/tunnel/usage" : `/api/tunnel/usage/${user_id}`;

  //  date handling
  let query_start = start.toISOString();
  query_start = query_start.substring(0, query_start.length - 1);
  let query_end = end.toISOString();
  query_end = query_end.substring(0, query_end.length - 1);

  try {
    const res = await cronFetcher.get<{
      usage_data_points: {
        bucket: number;
        external_connection_count: number;
        inbound: number;
        outbound: number;
      }[];
    }>(uri, {
      params: {
        resolution,
        query_start,
        query_end,
      },
    });
    return res.data.usage_data_points.map((value) => {
      return {
        timestamp: new Date(value.bucket * 1000),
        connections: value.external_connection_count,
        inbound: value.inbound,
        outbound: value.outbound,
      } as UsageDataPoint;
    });
  } catch (error) {
    console.log(`Failed to get usage: ${error}`);
    if (isAxiosError(error)) {
      return error.status || 500;
    } else {
      return 500;
    }
  }
};
