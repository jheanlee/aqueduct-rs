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

import { Outlet } from "react-router";
import { NavSidebar } from "@/components/layouts/nav-sidebar.tsx";
import { NavHeader } from "@/components/layouts/nav-header.tsx";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar.tsx";

export const Root = () => {
  return (
    <div className="w-screen h-screen">
      <SidebarProvider
        style={
          {
            "--sidebar-width": "calc(var(--spacing) * 72)",
            "--header-height": "calc(var(--spacing) * 12)",
          } as React.CSSProperties
        }
        className="flex w-full h-full"
      >
        <NavSidebar />
        <SidebarInset className="flex flex-col h-full min-w-0 overflow-hidden">
          <NavHeader />
          <main className="flex-1 overflow-y-auto p-6">
            <Outlet />
          </main>
        </SidebarInset>
      </SidebarProvider>
    </div>
  );
};
