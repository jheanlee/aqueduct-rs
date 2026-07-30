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

import { LogIn, LogOut } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar.tsx";
import { NavLink } from "react-router";
import { paths } from "@/config/paths.ts";
import ToggleThemeButton from "@/components/theme/theme-toggle-button.tsx";
import { Button } from "@/components/ui/button.tsx";
import { clearAuth, useAuthStore } from "@/store/authStore.ts";
import { logout } from "@/services/auth.ts";

export const NavSidebar = () => {
  const { isLoggedIn } = useAuthStore();

  return (
    <div>
      <Sidebar collapsible="offcanvas" variant="inset">
        <SidebarHeader>{/* TODO: Icon */}</SidebarHeader>
        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupContent>
              <SidebarMenu>
                <SidebarMenuItem key="dashboard">
                  <SidebarMenuButton asChild>
                    <NavLink to={paths.root.dashboard.getHref()}>
                      Dashboard
                    </NavLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
          <SidebarGroup>
            <SidebarGroupLabel>Management</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                <SidebarMenuItem key="users">
                  <SidebarMenuButton asChild>
                    <NavLink to={paths.root.users.getHref()}>Users</NavLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
                <SidebarMenuItem key="settings">
                  <SidebarMenuButton asChild>
                    <NavLink to={paths.root.settings.getHref()}>
                      Settings
                    </NavLink>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>
        <SidebarFooter>
          <SidebarMenu>
            <SidebarMenuItem className="flex justify-between align-middle gap-2">
              <SidebarMenuButton className="gap-0 p-0" asChild>
                <NavLink
                  to={paths.root.login.getHref()}
                  className="h-max w-auto"
                  onClick={() => {
                    if (isLoggedIn) {
                      void logout();
                      clearAuth();
                    }
                  }}
                >
                  {isLoggedIn ? (
                    <Button variant="ghost">
                      <LogOut />
                      Logout
                    </Button>
                  ) : (
                    <Button variant="ghost">
                      <LogIn />
                      Login
                    </Button>
                  )}
                </NavLink>
              </SidebarMenuButton>
              <ToggleThemeButton />
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarFooter>
      </Sidebar>
    </div>
  );
};
