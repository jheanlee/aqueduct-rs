import { Migration } from '@mikro-orm/migrations';

export class Migration20260718065308 extends Migration {

  override async up(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" drop constraint "tunnel_sessions_user_id_foreign";`);

    this.addSql(`alter table "tunnel_sessions" alter column "user_id" type varchar(21) using ("user_id"::varchar(21));`);
    this.addSql(`alter table "tunnel_sessions" alter column "user_id" drop not null;`);
    this.addSql(`alter table "tunnel_sessions" add constraint "tunnel_sessions_user_id_foreign" foreign key ("user_id") references "tunnel_users" ("id") on update cascade on delete set null;`);
  }

  override async down(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" drop constraint "tunnel_sessions_user_id_foreign";`);

    this.addSql(`alter table "tunnel_sessions" alter column "user_id" type varchar(21) using ("user_id"::varchar(21));`);
    this.addSql(`alter table "tunnel_sessions" alter column "user_id" set not null;`);
    this.addSql(`alter table "tunnel_sessions" add constraint "tunnel_sessions_user_id_foreign" foreign key ("user_id") references "tunnel_users" ("id") on update cascade on delete no action;`);
  }

}
