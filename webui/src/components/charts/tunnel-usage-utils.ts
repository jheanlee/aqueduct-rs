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

import { type UsageDataPoint } from "@/services/tunnel/usage.ts";
import { formatBytes } from "@/lib/format.ts";
import type { UsageDataPointFormatted } from "@/components/charts/tunnel-usage.tsx";

export const handleUsageChartDataPoints = (
  data: UsageDataPoint[],
  zoom: string,
) => {
  const formatTime = (timestamp: Date) => {
    const pad = (value: number) => {
      return String(value).padStart(2, "0");
    };

    switch (zoom) {
      case "daily":
        return timestamp.toISOString().substring(11, 16);
      case "weekly":
        return `${pad(timestamp.getUTCMonth())}/${pad(timestamp.getUTCDate())} ${pad(timestamp.getUTCHours())}:${pad(timestamp.getUTCMinutes())}`;
      case "monthly":
        return `${pad(timestamp.getUTCMonth())}/${pad(timestamp.getUTCDate())}`;
      case "yearly":
        return `${pad(timestamp.getUTCMonth())}/${pad(timestamp.getUTCDate())}`;
    }
  };

  return data.map((value) => {
    return {
      timestamp: formatTime(value.timestamp),
      inboundBytes: value.inbound,
      inboundString: formatBytes(value.inbound),
      outboundBytes: value.outbound,
      outboundString: formatBytes(value.outbound),
      connections: value.connections,
    } as UsageDataPointFormatted;
  });
};
