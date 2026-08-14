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
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table.tsx";
import { useEffect, useState } from "react";
import { getTunnelUsers } from "@/services/tunnel/users.ts";
import { Badge } from "@/components/ui/badge";
import { CreateUser } from "@/components/forms/users/create-user.tsx";
import { EditUser } from "@/components/forms/users/edit-user.tsx";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu.tsx";
import { ChartSpline, Menu, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button.tsx";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
} from "@/components/ui/dialog.tsx";
import { RotateToken } from "@/components/subpages/rotate-token.tsx";
import { useNavigate } from "react-router";
import { paths } from "@/config/paths.ts";

export const Users = () => {
  const navigate = useNavigate();

  const [users, setUsers] = useState<
    | {
        id: string;
        username: string;
        label: string[];
        last_login: Date;
        administrator: boolean;
      }[]
    | null
  >(null);
  const [usersUpdateTrigger, setUsersUpdateTrigger] = useState<boolean>(false);
  const [rotateTokenDialogTarget, setRotateTokenDialogTarget] = useState<
    string | null
  >(null);

  const triggerUsersUpdate = () => setUsersUpdateTrigger(!usersUpdateTrigger);

  useEffect(() => {
    const fetchUsers = async () => {
      setUsers(await getTunnelUsers());
    };

    void fetchUsers();
  }, [usersUpdateTrigger]);

  return (
    <div className="flex flex-col">
      <div className="flex flex-row justify-end gap-2 w-full">
        <CreateUser onClose={triggerUsersUpdate} />
      </div>
      <div className="overflow-x-auto pb-4">
        <Table className="min-w-max">
          <TableHeader>
            <TableRow>
              <TableHead>Username</TableHead>
              <TableHead>Admin</TableHead>
              <TableHead>Label</TableHead>
              <TableHead>Last Login</TableHead>
              <TableHead className="sticky right-0 text-right">
                Actions
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {users !== null &&
              users.map((item) => {
                return (
                  <TableRow
                    key={item.id}
                    className="group bg-background transition-colors"
                  >
                    <TableCell className="transition-colors group-hover:bg-muted/50">
                      {item.username}
                    </TableCell>
                    <TableCell className="transition-colors group-hover:bg-muted/50">
                      {item.administrator.toString()}
                    </TableCell>
                    <TableCell className="transition-colors group-hover:bg-muted/50">
                      <div className="flex flex-row gap-1">
                        {item.label.map((label, index) => {
                          return (
                            <Badge
                              key={`${item.id}-${index}`}
                              variant="secondary"
                            >
                              {label}
                            </Badge>
                          );
                        })}
                      </div>
                    </TableCell>
                    <TableCell className="transition-colors group-hover:bg-muted/50">
                      {item.last_login.toISOString()}
                    </TableCell>
                    <TableCell className="sticky right-0 z-10 bg-background transition-colors group-hover:bg-muted/50">
                      <div className="flex flex-row gap-1 justify-end">
                        <Button
                          variant={"outline"}
                          onClick={() => {
                            navigate(paths.root.users.usage.getHref(item.id));
                          }}
                          className="mx-1"
                        >
                          <ChartSpline />
                        </Button>
                        <EditUser
                          onClose={triggerUsersUpdate}
                          user={{
                            id: item.id,
                            username: item.username,
                            administrator: item.administrator,
                            label: item.label,
                          }}
                        />
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="outline">
                              <Menu />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end" className="w-36">
                            <DropdownMenuGroup>
                              <DropdownMenuItem
                                onClick={() =>
                                  setRotateTokenDialogTarget(item.id)
                                }
                              >
                                <RefreshCw />
                                Refresh Token
                              </DropdownMenuItem>
                            </DropdownMenuGroup>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </div>
                    </TableCell>
                  </TableRow>
                );
              })}
          </TableBody>
        </Table>
        <Dialog open={rotateTokenDialogTarget !== null}>
          <DialogContent showCloseButton={false} className="sm:max-w-md">
            <RotateToken id={rotateTokenDialogTarget} />
            <DialogFooter>
              <DialogClose>
                <Button onClick={() => setRotateTokenDialogTarget(null)}>
                  Close
                </Button>
              </DialogClose>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
    </div>
  );
};
