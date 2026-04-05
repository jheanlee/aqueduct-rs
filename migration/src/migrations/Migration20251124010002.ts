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

import { Migration } from "@mikro-orm/migrations";

export class Migration20251124010002 extends Migration {
  override async up(): Promise<void> {
    this.addSql(
      `create table "ip_blacklist" ("id" serial primary key, "subnet" inet not null, "comment" text not null);`,
    );

    this.addSql(
      `create table "ip_whitelist" ("id" serial primary key, "subnet" inet not null, "comment" text not null);`,
    );

    this.addSql(
      `create table "settings" ("key" text not null, "value" text not null, constraint "settings_pkey" primary key ("key"));`,
    );

    this.addSql(
      `create table "statistics" ("id" varchar(21) not null, "inbound" bigint not null, "outbound" bigint not null, constraint "statistics_pkey" primary key ("id"));`,
    );

    this.addSql(
      `create table "user" ("id" varchar(21) not null, "username" text not null, "hashed_password" text not null, "salt" varchar(12) not null, constraint "user_pkey" primary key ("id"));`,
    );
    this.addSql(
      `alter table "user" add constraint "user_username_unique" unique ("username");`,
    );
  }

  override async down(): Promise<void> {
    this.addSql(`drop table if exists "ip_blacklist" cascade;`);

    this.addSql(`drop table if exists "ip_whitelist" cascade;`);

    this.addSql(`drop table if exists "settings" cascade;`);

    this.addSql(`drop table if exists "statistics" cascade;`);

    this.addSql(`drop table if exists "user" cascade;`);
  }
}
