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
  CardHeader,
  CardTitle,
} from "@/components/ui/card.tsx";
import { useEffect, useState } from "react";
import { get_system_status, get_tunnel_status } from "@/services/status.ts";

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

  const formatTime = (time: number) => {
    const days = Math.floor(time / 86400);
    const hours = Math.floor((time % 86400) / 3600);
    const minutes = Math.floor((time % 3600) / 60);
    const seconds = Math.floor(time % 60);

    const parts = [
      days > 0 ? `${days}d` : null,
      hours > 0 ? `${hours}h` : null,
      minutes > 0 ? `${minutes}m` : null,
      `${seconds}s`,
    ];

    return parts.filter(Boolean).join(" ");
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";

    const sizes = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

    const index = Math.floor(Math.log(bytes) / Math.log(1024));

    return `${parseFloat((bytes / Math.pow(1024, index)).toFixed(1))} ${sizes[index]}`;
  };

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

    const intervalId = setInterval(fetchData, 10000);

    return () => clearInterval(intervalId);
  }, []);

  return (
    <div className="flex flex-col gap-4 text-foreground">
      <Card>
        <CardHeader>
          <CardTitle>Tunnel Service Status</CardTitle>
        </CardHeader>
        <CardContent>
          <div>
            <p>{`Uptime: ${tunnelStatus.uptime !== null ? formatTime(tunnelStatus.uptime) : "N/A"}`}</p>
            <p>{`Active service count: ${tunnelStatus.active_service_count !== null ? tunnelStatus.active_service_count : "N/A"}`}</p>
            <p>{`Active external connection count: ${tunnelStatus.active_external_connection_count !== null ? tunnelStatus.active_external_connection_count : "N/A"}`}</p>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>System Status</CardTitle>
        </CardHeader>
        <CardContent>
          <div>
            <p>{`Global CPU usage: ${systemStatus.cpu_usage !== null ? systemStatus.cpu_usage.toFixed(1) : "N/A"}`}</p>
            <p>{`Global memory usage: ${systemStatus.used_memory !== null ? formatBytes(systemStatus.used_memory) : "N/A "}/${systemStatus.total_memory !== null ? formatBytes(systemStatus.total_memory) : " N/A"}`}</p>
            <p>{`Process CPU usage: ${systemStatus.process_cpu_usage !== null ? systemStatus.process_cpu_usage.toFixed(1) : "N/A"}`}</p>
            <p>{`Process memory usage (RSS): ${systemStatus.process_memory !== null ? formatBytes(systemStatus.process_memory) : "N/A"}`}</p>
            <p>{`Process file descriptor count: ${systemStatus.process_fd_count !== null ? systemStatus.process_fd_count : "N/A"}`}</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
