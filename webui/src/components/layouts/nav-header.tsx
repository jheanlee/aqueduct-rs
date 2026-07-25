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

import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb.tsx";
import { useMatches } from "react-router";
import { SidebarTrigger } from "@/components/ui/sidebar.tsx";
import { Separator } from "@/components/ui/separator.tsx";
import { Fragment } from "react";

interface RouteHandle {
  title?: string;
}

export const NavHeader = () => {
  const matches = useMatches();
  const breadcrumbMatches = matches.filter(
    (match) => (match.handle as RouteHandle)?.title,
  );

  return (
    <header className="flex h-(--header-height) w-full shrink-0 items-center gap-2 border-b transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)">
      <div className="flex w-full items-center gap-2 px-4 lg:px-6">
        <SidebarTrigger className="-ml-2" />
        <Separator orientation="vertical" className="mx-2" />
        <Breadcrumb>
          <BreadcrumbList>
            {breadcrumbMatches.map((match, index) => {
              return (
                <Fragment key={match.id}>
                  <BreadcrumbItem>
                    {index !== breadcrumbMatches.length - 1 && (
                      <BreadcrumbLink href={match.pathname}>
                        {(match.handle as RouteHandle).title}
                      </BreadcrumbLink>
                    )}
                    {index === breadcrumbMatches.length - 1 && (
                      <BreadcrumbPage>
                        {(match.handle as RouteHandle).title}
                      </BreadcrumbPage>
                    )}
                  </BreadcrumbItem>
                  {index !== breadcrumbMatches.length - 1 && (
                    <BreadcrumbSeparator />
                  )}
                </Fragment>
              );
            })}
          </BreadcrumbList>
        </Breadcrumb>
      </div>
    </header>
  );
};
