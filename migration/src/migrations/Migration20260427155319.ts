import { Migration } from '@mikro-orm/migrations';

export class Migration20260427155319 extends Migration {

  override async up(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" add constraint "tunnel_sessions_user_id_bucket_start_tunnel_client_unique" unique ("user_id", "bucket_start", "tunnel_client");`);
  }

  override async down(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" drop constraint "tunnel_sessions_user_id_bucket_start_tunnel_client_unique";`);
  }

}
