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

import { Entity, PrimaryKey, Property, Unique } from "@mikro-orm/core";

@Entity()
export class TunnelUsers {
  @PrimaryKey({ length: 21 })
  id!: string;

  @Property({ type: "text" })
  @Unique()
  username!: string;

  //  `aq_` + 32 random bytes, base58 encoded
  @Property({ type: "text" })
  @Unique()
  token!: string;

  //  hashed with argon2-id
  @Property({ type: "text" })
  hashedPassword!: string;

  @Property()
  label!: string[];

  @Property({ type: "timestamp" })
  lastLogin!: string;

  @Property()
  administrator!: boolean;
}
