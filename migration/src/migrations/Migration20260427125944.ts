import { Migration } from '@mikro-orm/migrations';

export class Migration20260427125944 extends Migration {

  override async up(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" alter column "id" type bigint using ("id"::bigint);`);
    this.addSql(`create sequence if not exists "tunnel_sessions_id_seq";`);
    this.addSql(`select setval('tunnel_sessions_id_seq', (select max("id") from "tunnel_sessions"));`);
    this.addSql(`alter table "tunnel_sessions" alter column "id" set default nextval('tunnel_sessions_id_seq');`);
  }

  override async down(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" alter column "id" type varchar(21) using ("id"::varchar(21));`);
    this.addSql(`alter table "tunnel_sessions" alter column "id" drop default;`);
  }

}
