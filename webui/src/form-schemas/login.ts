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

import { z } from "zod";

export const loginSchema = z.object({
  username: z
    .string()
    .min(4, {
      message: "Username must be at least 4 characters.",
    })
    .max(64, {
      message: "Username must not exceed 64 characters.",
    })
    .regex(/^[A-Za-z0-9_-]+$/, {
      message:
        "Username should only contain letters (A-Z, a-z), numbers (0-9), underscores (_) and hyphens (-).",
    }),

  password: z
    .string()
    .min(8, {
      message: "Password must be at least 8 characters.",
    })
    .max(256, {
      message: "Password must not exceed 256 characters.",
    })
    .regex(/^[A-Za-z0-9~!@#$%^&*()_\-+={}[\]|\\:;,./]+$/, {
      message:
        "Password should only contain letters (A-Z, a-z), numbers (0-9) and symbols.",
    }),
});
