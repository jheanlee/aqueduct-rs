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

import { cronFetcher, fetcher } from "@/services/fetcher.ts";
import { isAxiosError } from "axios";
import { createUserSchema } from "@/form-schemas/users/create-user.ts";
import { z } from "zod";
import type { editUserSchema } from "@/form-schemas/users/edit-user.ts";

export const getTunnelUsers = async () => {
  try {
    const users = await cronFetcher.get<
      {
        id: string;
        username: string;
        label: string[];
        last_login: string;
        administrator: boolean;
      }[]
    >("/api/tunnel/users");
    return users.data;
  } catch (error) {
    console.log(`Failed to get users: ${error}`);
    return null;
  }
};

export const createTunnelUser = async (
  values: z.infer<typeof createUserSchema>,
) => {
  try {
    const res = await fetcher.post("/api/tunnel/users", values);
    return res.status;
  } catch (error) {
    console.log(`Failed to create user: ${error}`);
    if (isAxiosError(error)) {
      return error.status || 500;
    } else {
      return 500;
    }
  }
};

export const editTunnelUser = async (
  id: string,
  values: z.infer<typeof editUserSchema>,
) => {
  try {
    const res = await fetcher.put(`/api/tunnel/users/${id}`, values);
    return res.status;
  } catch (error) {
    console.log(`Failed to edit user: ${error}`);
    if (isAxiosError(error)) {
      return error.status || 500;
    } else {
      return 500;
    }
  }
};

export const deleteTunnelUser = async (id: string) => {
  try {
    const res = await fetcher.delete(`/api/tunnel/users/${id}`);
    return res.status;
  } catch (error) {
    console.log(`Failed to edit user: ${error}`);
    if (isAxiosError(error)) {
      return error.status || 500;
    } else {
      return 500;
    }
  }
};

export const rotateTunnelUserToken = async (id: string) => {
  try {
    const res = await fetcher.post<{ token: string }>(
      `/api/tunnel/users/${id}/token/rotate`,
    );
    return res.status;
  } catch (error) {
    console.log(`Failed to edit user: ${error}`);
    if (isAxiosError(error)) {
      return error.status || 500;
    } else {
      return 500;
    }
  }
};
