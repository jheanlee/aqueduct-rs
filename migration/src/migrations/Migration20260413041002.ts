import { Migration } from '@mikro-orm/migrations';

export class Migration20260413041002 extends Migration {

  override async up(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" add column "external_client" inet not null;`);
    this.addSql(`alter table "tunnel_sessions" rename column "ip_addr" to "tunnel_client";`);
  }

  override async down(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" drop column "external_client";`);

    this.addSql(`alter table "tunnel_sessions" rename column "tunnel_client" to "ip_addr";`);
  }

}
