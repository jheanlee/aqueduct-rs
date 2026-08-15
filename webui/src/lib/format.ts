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

export const formatBytes = (bytes: number) => {
  if (bytes === 0) return "0 B";

  const sizes = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

  const index = Math.floor(Math.log(bytes) / Math.log(1024));

  return `${parseFloat((bytes / Math.pow(1024, index)).toFixed(1))} ${sizes[index]}`;
};

export const formatTimeFromSeconds = (time: number) => {
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

export const formatTimeWithZoom = (timestamp: Date, zoom: string) => {
  const pad = (value: number) => {
    return String(value).padStart(2, "0");
  };

  switch (zoom) {
    case "daily":
      return timestamp.toISOString().substring(11, 16);
    case "weekly":
      return `${pad(timestamp.getUTCMonth() + 1)}/${pad(timestamp.getUTCDate())} ${pad(timestamp.getUTCHours())}:${pad(timestamp.getUTCMinutes())}`;
    case "monthly":
      return `${pad(timestamp.getUTCMonth() + 1)}/${pad(timestamp.getUTCDate())}`;
    case "yearly":
      return `${pad(timestamp.getUTCMonth() + 1)}/${pad(timestamp.getUTCDate())}`;
  }
};
