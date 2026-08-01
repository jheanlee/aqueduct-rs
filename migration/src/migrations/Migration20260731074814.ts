import { Migration } from '@mikro-orm/migrations';

export class Migration20260731074814 extends Migration {

  override async up(): Promise<void> {
    this.addSql(`create index "idx_tunnel_sessions_bucket_start" on "tunnel_sessions" ("bucket_start");`);
    this.addSql(`create index "idx_tunnel_sessions_user_bucket_start" on "tunnel_sessions" ("user_id", "bucket_start");`);
  }

  override async down(): Promise<void> {
    this.addSql(`drop index "idx_tunnel_sessions_bucket_start";`);
    this.addSql(`drop index "idx_tunnel_sessions_user_bucket_start";`);
  }

}
