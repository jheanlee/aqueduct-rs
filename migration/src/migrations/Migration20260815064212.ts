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

export class Migration20260815064212 extends Migration {
  override async up(): Promise<void> {
    this.addSql(
      `create table "tunnel_status" ("id" bigserial primary key, "bucket_start" timestamp not null, "sample_count" bigint not null, "active_service_avg" bigint not null, "active_service_max" bigint not null, "external_connection_avg" bigint not null, "external_connection_max" bigint not null);`,
    );
    this.addSql(
      `alter table "tunnel_status" add constraint "tunnel_status_bucket_start_unique" unique ("bucket_start");`,
    );
  }

  override async down(): Promise<void> {
    this.addSql(`drop table if exists "tunnel_status" cascade;`);
  }
}
