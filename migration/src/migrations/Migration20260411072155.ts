import { Migration } from '@mikro-orm/migrations';

export class Migration20260411072155 extends Migration {

  override async up(): Promise<void> {
    this.addSql(`create table "web_users" ("id" varchar(21) not null, "username" text not null, "hashed_password" text not null, "salt" varchar(22) not null, "administrator" boolean not null, constraint "web_users_pkey" primary key ("id"));`);
    this.addSql(`alter table "web_users" add constraint "web_users_username_unique" unique ("username");`);

    this.addSql(`alter table "tunnel_users" alter column "salt" type varchar(22) using ("salt"::varchar(22));`);
  }

  override async down(): Promise<void> {
    this.addSql(`drop table if exists "web_users" cascade;`);

    this.addSql(`alter table "tunnel_users" alter column "salt" type varchar(12) using ("salt"::varchar(12));`);
  }

}
