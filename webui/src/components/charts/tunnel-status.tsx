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

import type { StatusDataPointFormatted } from "@/components/charts/tunnel-status-utils.ts";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart.tsx";
import { Line, LineChart, XAxis } from "recharts";

interface TunnelStatusServiceChartProps {
  chartData: Omit<
    StatusDataPointFormatted,
    "externalConnectionAvg" | "externalConnectionMax"
  >[];
}

export const TunnelStatusServiceChart = ({
  chartData,
}: TunnelStatusServiceChartProps) => {
  const chartConfig = {
    activeServiceAvg: {
      label: "Average Services",
      color: "var(--chart-1)",
    },
    activeServiceMax: {
      label: "Maximum Services",
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

        <ChartTooltip cursor={false} content={<ChartTooltipContent />} />

        <Line
          dataKey="activeServiceAvg"
          type="monotone"
          stroke="var(--color-activeServiceAvg)"
          strokeWidth={2}
          dot={false}
        />

        <Line
          dataKey="activeServiceMax"
          type="monotone"
          stroke="var(--color-activeServiceMax)"
          strokeWidth={2}
          dot={false}
        />
      </LineChart>
    </ChartContainer>
  );
};

interface TunnelStatusConnectionChartProps {
  chartData: Omit<
    StatusDataPointFormatted,
    "activeServiceMax" | "activeServiceAvg"
  >[];
}

export const TunnelStatusConnectionChart = ({
  chartData,
}: TunnelStatusConnectionChartProps) => {
  const chartConfig = {
    externalConnectionAvg: {
      label: "Average Connections",
      color: "var(--chart-1)",
    },
    externalConnectionMax: {
      label: "Maximum Connections",
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

        <ChartTooltip cursor={false} content={<ChartTooltipContent />} />

        <Line
          dataKey="externalConnectionAvg"
          type="monotone"
          stroke="var(--color-externalConnectionAvg)"
          strokeWidth={2}
          dot={false}
        />

        <Line
          dataKey="externalConnectionMax"
          type="monotone"
          stroke="var(--color-externalConnectionMax)"
          strokeWidth={2}
          dot={false}
        />
      </LineChart>
    </ChartContainer>
  );
};
