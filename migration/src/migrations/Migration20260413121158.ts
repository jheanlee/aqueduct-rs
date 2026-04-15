import { Migration } from '@mikro-orm/migrations';

export class Migration20260413121158 extends Migration {

  override async up(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" alter column "tunnel_client" type varchar(255) using ("tunnel_client"::varchar(255));`);
    this.addSql(`alter table "tunnel_sessions" alter column "external_client" type varchar(255) using ("external_client"::varchar(255));`);
  }

  override async down(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" alter column "tunnel_client" type inet using ("tunnel_client"::inet);`);
    this.addSql(`alter table "tunnel_sessions" alter column "external_client" type inet using ("external_client"::inet);`);
  }

}
