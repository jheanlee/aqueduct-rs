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

import { createJSONStorage, persist } from "zustand/middleware";
import { create } from "zustand/react";
import { cronFetcher, fetcher, publicFetcher } from "@/services/fetcher.ts";

interface AuthStore {
  accessToken: string | null;
  refreshToken: string | null;
  isLoggedIn: boolean;
}

export const useAuthStore = create<AuthStore>()(
  persist<AuthStore>(
    () => ({
      accessToken: null,
      refreshToken: null,
      isLoggedIn: false,
    }),
    {
      name: "auth-storage",
      storage: createJSONStorage(() => localStorage),
    },
  ),
);

export const setRefreshToken = (token: string) => {
  useAuthStore.setState({ refreshToken: token });
};

export const setAccessToken = (token: string) => {
  useAuthStore.setState({ accessToken: token, isLoggedIn: true });
  fetcher.defaults.headers["Authorization"] = token;
  cronFetcher.defaults.headers["Authorization"] = token;
  publicFetcher.defaults.headers["Authorization"] = token;
};

export const clearAuth = () => {
  useAuthStore.setState({
    accessToken: null,
    refreshToken: null,
    isLoggedIn: false,
  });
  fetcher.defaults.headers["Authorization"] = null;
  cronFetcher.defaults.headers["Authorization"] = null;
  publicFetcher.defaults.headers["Authorization"] = null;
};
