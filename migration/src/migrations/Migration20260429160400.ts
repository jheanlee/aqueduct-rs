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

export class Migration20260429160400 extends Migration {
  override async up(): Promise<void> {
    this.addSql(
      `insert into "settings" ("key", "value") values ('blacklist', 'true') on conflict do nothing;`,
    );
    this.addSql(
      `insert into "settings" ("key", "value") values ('whitelist', 'false') on conflict do nothing;`,
    );
  }

  override async down(): Promise<void> {}
}
