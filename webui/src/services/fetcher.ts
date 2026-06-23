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

import axios from "axios";
import { Buffer } from "buffer";
import { paths } from "@/config/paths.ts";
import { toast } from "sonner";
import { refreshToken } from "@/services/auth.ts";
import { clearAuth, useAuthStore } from "@/store/authStore.ts";

// public fetcher (login & refresh token)
export const publicFetcher = axios.create();
publicFetcher.interceptors.request.use(async (config) => {
  if (useAuthStore.getState().accessToken !== null) {
    config.headers["Authorization"] = useAuthStore.getState().accessToken;
  }
  return config;
});

// cron job (automatic api calls)
export const cronFetcher = axios.create();

cronFetcher.interceptors.request.use(async (config) => {
  if (useAuthStore.getState().accessToken === null) {
    clearAuth();
    toast.error("Session expired");
    window.location.href = paths.root.login.getHref();
    return config;
  }

  config.headers["Authorization"] = useAuthStore.getState().accessToken;

  if (
    JSON.parse(
      Buffer.from(
        useAuthStore.getState().accessToken.split(".")[1],
        "base64",
      ).toString("ascii"),
    ).exp <
    Date.now() / 1000
  ) {
    clearAuth();
    toast.error("Session expired");
    window.location.href = paths.root.login.getHref();
    return config;
  }
  return config;
});

// private fetcher (user conducted api calls)
export const fetcher = axios.create();

fetcher.interceptors.request.use(async (config) => {
  if (useAuthStore.getState().accessToken === null) {
    clearAuth();
    toast.error("Login required");
    window.location.href = paths.root.login.getHref();
    return config;
  }

  config.headers["Authorization"] = useAuthStore.getState().accessToken;

  const res = await refreshToken();
  if (res !== 200) {
    clearAuth();
    toast.error("Session expired");
    window.location.href = paths.root.login.getHref();
    return config;
  }

  return config;
});
