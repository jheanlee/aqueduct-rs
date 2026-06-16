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

import { Button } from "@/components/ui/button";
import { NavLink } from "react-router";
import { paths } from "@/config/paths.ts";

export const NotFound = () => {
  return (
    <div className="h-full w-full mt-15 flex flex-col items-center font-semibold">
      <p>404 Not Found</p>
      <p>The requested page does not exist.</p>
      <Button variant="link" className="mt-5" asChild>
        <NavLink to={paths.root.dashboard.getHref()}>
          Return to Dashboard
        </NavLink>
      </Button>
    </div>
  );
};
