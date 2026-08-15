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

import { useNavigate, useParams } from "react-router";
import { useEffect, useState } from "react";
import { paths } from "@/config/paths.ts";
import { getTunnelUsage } from "@/services/tunnel/usage.ts";
import { toast } from "sonner";
import {
  TunnelUsageConnectionsChart,
  TunnelUsageIOChart,
} from "@/components/charts/tunnel-usage.tsx";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card.tsx";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select.tsx";
import {
  handleUsageChartDataPoints,
  type UsageDataPointFormatted,
} from "@/components/charts/tunnel-usage-utils.ts";

export const UserUsage = () => {
  const navigate = useNavigate();
  const params = useParams();

  const [usageChartData, setUsageChartData] = useState<
    UsageDataPointFormatted[]
  >([]);
  const [usageChartZoom, setUsageChartZoom] = useState<string>("daily");

  const usageChartZoomSelectItems = [
    { label: "Last 24 hours", value: "daily" },
    { label: "Last 7 days", value: "weekly" },
    { label: "Last 30 days", value: "monthly" },
    { label: "Last 12 months", value: "yearly" },
  ];

  useEffect(() => {
    if (params.id === undefined) {
      navigate(paths.root.notFound.getHref());
    }
  }, [navigate, params.id]);

  useEffect(() => {
    const getResolution = (zoom: string) => {
      switch (zoom) {
        case "daily":
          return "ten_minutes";
        case "weekly":
          return "hourly";
        case "monthly":
          return "daily";
        case "yearly":
          return "weekly";
        default:
          return "ten_minutes";
      }
    };

    const dateNow = Date.now();

    const getQueryStart = (queryEnd: number) => {
      switch (usageChartZoom) {
        case "daily":
          return queryEnd - 24 * 60 * 60 * 1000;
        case "weekly":
          return queryEnd - 7 * 24 * 60 * 60 * 1000;
        case "monthly":
          return queryEnd - 30 * 24 * 60 * 60 * 1000;
        case "yearly":
          return queryEnd - 365 * 24 * 60 * 60 * 1000;
        default:
          return queryEnd;
      }
    };

    const fetchData = async () => {
      if (params.id === undefined) return;
      const res = await getTunnelUsage(
        params.id,
        new Date(getQueryStart(dateNow)),
        new Date(dateNow),
        getResolution(usageChartZoom),
      );
      if (typeof res === "number") {
        toast.error(`An error has occurred. Error code: ${res}`);
      } else {
        setUsageChartData(handleUsageChartDataPoints(res, usageChartZoom));
      }
    };

    const updateChartTimer = async () => {
      if (new Date().getMinutes() % 10 === 0) {
        await fetchData();
      }
    };

    void fetchData();

    const intervalID = setInterval(updateChartTimer, 60 * 1000);
    return () => clearInterval(intervalID);
  }, [usageChartZoom, params.id]);

  return (
    <div className="grid gird-cols-1 md:grid-cols-2 gap-4 text-foreground">
      <Card className={"col-span-1 md:col-span-2"}>
        <CardHeader>
          <CardTitle>Tunnel Usage</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-1">
          <Select
            defaultValue={"daily"}
            onValueChange={(value) => {
              setUsageChartZoom(value);
            }}
          >
            <SelectTrigger className="mb-2 w-45 self-end">
              <SelectValue placeholder="Zoom" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {usageChartZoomSelectItems.map((item) => (
                  <SelectItem key={item.value} value={item.value}>
                    {item.label}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
          <div className="grid grid-cols-1 md:grid-cols-2">
            <div className="px-4">
              <p className="font-semibold">Throughput</p>
              <TunnelUsageIOChart chartData={usageChartData} />
            </div>
            <div className="px-4">
              <p className="font-semibold">New External Connections</p>
              <TunnelUsageConnectionsChart chartData={usageChartData} />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
