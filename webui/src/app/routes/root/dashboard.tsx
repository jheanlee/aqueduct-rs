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
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card.tsx";
import { useEffect, useState } from "react";
import { get_system_status, get_tunnel_status } from "@/services/status.ts";
import {
  TunnelUsageConnectionsChart,
  TunnelUsageIOChart,
  type UsageDataPointFormatted,
} from "@/components/charts/tunnel-usage.tsx";
import { getTunnelUsage } from "@/services/tunnel/usage.ts";
import { toast } from "sonner";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select.tsx";
import { formatBytes, formatTimeFromSeconds } from "@/lib/format.ts";
import { handleUsageChartDataPoints } from "@/components/charts/tunnel-usage-utils.ts";

export const Dashboard = () => {
  const [systemStatus, setSystemStatus] = useState<{
    cpu_usage: number | null;
    used_memory: number | null;
    total_memory: number | null;
    process_cpu_usage: number | null;
    process_memory: number | null;
    process_fd_count: number | null;
  }>({
    cpu_usage: null,
    used_memory: null,
    total_memory: null,
    process_cpu_usage: null,
    process_memory: null,
    process_fd_count: null,
  });
  const [tunnelStatus, setTunnelStatus] = useState<{
    uptime: number | null;
    active_service_count: number | null;
    active_external_connection_count: number | null;
  }>({
    uptime: null,
    active_service_count: null,
    active_external_connection_count: null,
  });

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
    const fetchData = async () => {
      const [tunnel, system] = await Promise.all([
        get_tunnel_status(),
        get_system_status(),
      ]);
      setTunnelStatus(tunnel);
      setSystemStatus(system);
    };

    void fetchData();

    const intervalID = setInterval(fetchData, 10000);

    return () => clearInterval(intervalID);
  }, []);

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
      const res = await getTunnelUsage(
        null,
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
  }, [usageChartZoom]);

  return (
    <div className="grid gird-cols-1 md:grid-cols-2 gap-4 text-foreground">
      <Card>
        <CardHeader>
          <CardTitle>Tunnel Service Status</CardTitle>
          <CardDescription>Update interval: 10s</CardDescription>
        </CardHeader>

        <CardContent className="flex flex-col gap-1">
          <p>{`Uptime: ${tunnelStatus.uptime !== null ? formatTimeFromSeconds(tunnelStatus.uptime) : "N/A"}`}</p>
          <p>{`Active service count: ${tunnelStatus.active_service_count !== null ? tunnelStatus.active_service_count : "N/A"}`}</p>
          <p>{`Active external connection count: ${tunnelStatus.active_external_connection_count !== null ? tunnelStatus.active_external_connection_count : "N/A"}`}</p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>System Status</CardTitle>
          <CardDescription>
            Update interval: 10s (60 s for file descriptor)
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-1">
          <p>{`Global CPU usage: ${systemStatus.cpu_usage !== null ? systemStatus.cpu_usage.toFixed(1) : "N/A"}%`}</p>
          <p>{`Global memory usage: ${systemStatus.used_memory !== null ? formatBytes(systemStatus.used_memory) : "N/A "}/${systemStatus.total_memory !== null ? formatBytes(systemStatus.total_memory) : " N/A"}`}</p>
          <p>{`Process CPU usage: ${systemStatus.process_cpu_usage !== null ? systemStatus.process_cpu_usage.toFixed(1) : "N/A"}`}</p>
          <p>{`Process memory usage (RSS): ${systemStatus.process_memory !== null ? formatBytes(systemStatus.process_memory) : "N/A"}`}</p>
          <p>{`Process file descriptor count: ${systemStatus.process_fd_count !== null ? systemStatus.process_fd_count : "N/A"}`}</p>
        </CardContent>
      </Card>
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
              <p className="font-semibold">Tunnelled IO</p>
              <TunnelUsageIOChart chartData={usageChartData} />
            </div>
            <div className="px-4">
              <p className="font-semibold">Connections</p>
              <TunnelUsageConnectionsChart chartData={usageChartData} />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
