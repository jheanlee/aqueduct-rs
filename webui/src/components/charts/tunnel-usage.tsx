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

import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart.tsx";
import { Line, LineChart, XAxis } from "recharts";
import { type UsageDataPoint } from "@/services/tunnel/usage.ts";
import { formatBytes } from "@/lib/format.ts";

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

export interface UsageDataPointFormatted {
  timestamp: string;
  inboundBytes: number;
  inboundString: string;
  outboundBytes: number;
  outboundString: string;
  connections: number;
}

interface TunnelUsageIOChartProps {
  chartData: Omit<UsageDataPointFormatted, "connections">[];
}

export const TunnelUsageIOChart = ({ chartData }: TunnelUsageIOChartProps) => {
  const chartConfig = {
    inboundBytes: {
      label: "Inbound",
      color: "var(--chart-1)",
    },
    outboundBytes: {
      label: "Outbound",
      color: "var(--chart-2)",
    },
  };

  return (
    <ChartContainer config={chartConfig}>
      <LineChart
        data={chartData}
        margin={{
          left: 12,
          right: 12,
        }}
      >
        <XAxis
          dataKey="timestamp"
          tickLine={false}
          tickMargin={10}
          axisLine={false}
        />

        <ChartTooltip
          cursor={false}
          content={(props) => {
            if (!props.active || !props.payload?.length) {
              return null;
            }

            const modifiedPayload = props.payload.map((item) => {
              return {
                ...item,
                value: (() => {
                  if (item.dataKey === "inboundBytes") {
                    return item.payload.inboundString;
                  }
                  if (item.dataKey === "outboundBytes") {
                    return item.payload.outboundString;
                  }
                  return item.value;
                })(),
              };
            });

            return (
              <ChartTooltipContent
                active={props.active}
                payload={modifiedPayload}
                label={props.label}
              />
            );
          }}
        />

        <Line
          dataKey="inboundBytes"
          type="monotone"
          stroke="var(--color-inboundBytes)"
          strokeWidth={2}
          dot={false}
        />

        <Line
          dataKey="outboundBytes"
          type="monotone"
          stroke="var(--color-outboundBytes)"
          strokeWidth={2}
          dot={false}
        />
      </LineChart>
    </ChartContainer>
  );
};

interface TunnelUsageConnectionsChartProps {
  chartData: Omit<
    UsageDataPointFormatted,
    "inboundBytes" | "inboundString" | "outboundBytes" | "outboundString"
  >[];
}

export const TunnelUsageConnectionsChart = ({
  chartData,
}: TunnelUsageConnectionsChartProps) => {
  const chartConfig = {
    connections: {
      label: "Connections",
      color: "var(--chart-1)",
    },
  };

  return (
    <ChartContainer config={chartConfig}>
      <LineChart
        data={chartData}
        margin={{
          left: 12,
          right: 12,
        }}
      >
        <XAxis
          dataKey="timestamp"
          tickLine={false}
          tickMargin={10}
          axisLine={false}
        />

        <ChartTooltip cursor={false} content={<ChartTooltipContent />} />

        <Line
          dataKey="connections"
          type="monotone"
          stroke="var(--color-connections)"
          strokeWidth={2}
          dot={false}
        />
      </LineChart>
    </ChartContainer>
  );
};
