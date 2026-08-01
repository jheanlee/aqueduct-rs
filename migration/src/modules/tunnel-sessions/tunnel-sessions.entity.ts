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
  Entity,
  Index,
  ManyToOne,
  PrimaryKey,
  Property,
  Unique,
} from "@mikro-orm/core";
import { TunnelUsers } from "../tunnel-users/tunnel-users.entity.js";

@Entity()
@Unique({ properties: ["user", "bucketStart", "tunnelClient"] })
@Index({
  name: "idx_tunnel_sessions_user_bucket_start",
  properties: ["user", "bucketStart"],
})
@Index({
  name: "idx_tunnel_sessions_bucket_start",
  properties: ["bucketStart"],
})
export class TunnelSessions {
  @PrimaryKey({ type: "bigint", autoincrement: true })
  id!: string;

  @ManyToOne(() => TunnelUsers, { deleteRule: "set null" })
  user!: TunnelUsers | null;

  @Property({ type: "timestamp" })
  bucketStart!: Date;

  //  ip address
  @Property({ type: "inet" })
  tunnelClient!: string;

  @Property({ type: "bigint" })
  inbound!: string;

  @Property({ type: "bigint" })
  outbound!: string;

  @Property({ type: "bigint" })
  external_connection_count!: string;
}
