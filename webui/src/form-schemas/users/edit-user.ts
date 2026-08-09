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

export const editUserSchema = z.object({
  password: z
    .string()
    .normalize()
    .min(8, {
      error: "Password must be at least 8 characters.",
    })
    .max(256, {
      error: "Password must not exceed 256 characters.",
    })
    .regex(/^[A-Za-z0-9~!@#$%^&*()_\-+={}[\]|\\:;,./]+$/, {
      error:
        "Password must only contain letters (A-Z, a-z), numbers (0-9) and symbols.",
    })
    .nullable(),

  label: z
    .array(
      z
        .string()
        .normalize()
        .min(1, { error: "A label must have at least one character." })
        .max(32, { error: "A label must not exceed 32 characters" }),
    )
    .max(32, { error: "A user may only have up to 32 labels" }),

  administrator: z.boolean(),
});
