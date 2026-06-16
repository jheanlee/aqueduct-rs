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

import { cronFetcher, fetcher, publicFetcher } from "@/services/fetcher.ts";
import { isAxiosError } from "axios";

export const refreshToken = async () => {
  try {
    const res = await publicFetcher.post<{ access_token: string }>(
      "/api/refresh",
      {
        refresh_token: localStorage.getItem("aqueduct.refresh_token"),
      },
    );

    localStorage.setItem("aqueduct.access_token", res.data.access_token);
    fetcher.defaults.headers["Authorization"] = res.data.access_token;
    cronFetcher.defaults.headers["Authorization"] = res.data.access_token;
    publicFetcher.defaults.headers["Authorization"] = res.data.access_token;
    return 200;
  } catch (error) {
    if (isAxiosError(error)) {
      return error.status || 500;
    }
    return 500;
  }
};

interface LoginParams {
  username: string;
  password: string;
}

export const login = async ({ username, password }: LoginParams) => {
  try {
    const res = await publicFetcher.post<{
      refresh_token: string;
      access_token: string;
    }>("/api/login", {
      username,
      password,
    });

    localStorage.setItem("aqueduct.refresh_token", res.data.access_token);
    localStorage.setItem("aqueduct.access_token", res.data.access_token);
    fetcher.defaults.headers["Authorization"] = res.data.access_token;
    cronFetcher.defaults.headers["Authorization"] = res.data.access_token;
    publicFetcher.defaults.headers["Authorization"] = res.data.access_token;
    return 200;
  } catch (error) {
    if (isAxiosError(error)) {
      return error.status || 500;
    }
    return 500;
  }
};

export const logout = async () => {
  void (await fetcher.post("/api/logout"));
  localStorage.removeItem("aqueduct.refresh_token");
  localStorage.removeItem("aqueduct.access_token");
  fetcher.defaults.headers["Authorization"] = null;
  cronFetcher.defaults.headers["Authorization"] = null;
  publicFetcher.defaults.headers["Authorization"] = null;
};
