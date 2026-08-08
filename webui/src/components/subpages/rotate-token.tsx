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

import { Button } from "@/components/ui/button.tsx";
import { useState } from "react";
import { rotateTunnelUserToken } from "@/services/tunnel/users.ts";

interface RotateTokenProps {
  id: string | null;
}

export const RotateToken = ({ id }: RotateTokenProps) => {
  const [token, setToken] = useState<{ id: string; value: string } | null>(
    null,
  );

  const rotateToken = async () => {
    if (id === null) return;
    const res = await rotateTunnelUserToken(id);
    if (typeof res === "string") {
      setToken({ id, value: res });
    }
  };

  const currentToken = token?.id === id ? token.value : null;

  return (
    <div>
      {id !== null && token === null && (
        <div className="flex flex-col p-1 gap-1">
          <p className="font-semibold">Refresh Token?</p>
          <p>
            Clients with the old token would lose access to the tunnel service.
          </p>
          <Button onClick={rotateToken} className="w-80 mt-1 self-center">
            Refresh
          </Button>
        </div>
      )}
      {id !== null && currentToken !== null && (
        <div className="flex flex-col p-1 gap-1">
          <p className="font-semibold">New Token: </p>
          <p className="font-mono">{currentToken}</p>
        </div>
      )}
    </div>
  );
};
