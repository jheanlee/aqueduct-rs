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
export class User {
  @PrimaryKey({ length: 21 })
  id!: string;

  @Property({ type: "text" })
  @Unique()
  username!: string;

  @Property({ type: "text" })
  hashedPassword!: string;

  //  8 random bytes, base64 encoded
  @Property({ length: 12 })
  salt!: string;
}
