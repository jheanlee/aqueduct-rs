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

import { createBrowserRouter, RouterProvider } from "react-router";
import { paths } from "@/config/paths.ts";
import { Root } from "@/app/routes/root/root.tsx";

const createAppRouter = () =>
  createBrowserRouter([
    {
      path: paths.root.path,
      Component: Root,
      handle: { title: paths.root.title },
      hydrateFallbackElement: <div>Loading...</div>,
      children: [
        {
          path: paths.root.dashboard.path,
          lazy: async () => {
            const { Dashboard } = await import("@/app/routes/root/dashboard");
            return { Component: Dashboard };
          },
        },
        {
          path: paths.root.users.path,
          lazy: async () => {
            const { Users } = await import("@/app/routes/root/users");
            return { Component: Users };
          },
          handle: { title: paths.root.users.title },
        },
        {
          path: paths.root.login.path,
          lazy: async () => {
            const { Login } = await import("@/app/routes/root/login");
            return { Component: Login };
          },
          handle: { title: paths.root.login.title },
        },
        {
          path: paths.root.notFound.path,
          lazy: async () => {
            const { NotFound } = await import("@/app/routes/root/not-found");
            return { Component: NotFound };
          },
          handle: { title: paths.root.notFound.title },
        },
      ],
    },
  ]);

export const AppRouter = () => {
  const router = createAppRouter();
  return <RouterProvider router={router} />;
};
