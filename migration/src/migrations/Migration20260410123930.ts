import { Migration } from '@mikro-orm/migrations';

export class Migration20260410123930 extends Migration {

  override async up(): Promise<void> {
    this.addSql(`create table "tunnel_users" ("id" varchar(21) not null, "username" text not null, "hashed_password" text not null, "salt" varchar(12) not null, constraint "tunnel_users_pkey" primary key ("id"));`);
    this.addSql(`alter table "tunnel_users" add constraint "tunnel_users_username_unique" unique ("username");`);

    this.addSql(`create table "tunnel_sessions" ("id" varchar(21) not null, "user_id" varchar(21) not null, "ip_addr" inet not null, "inbound" bigint not null, "outbound" bigint not null, "start_time" timestamptz not null, "end_time" timestamptz null, constraint "tunnel_sessions_pkey" primary key ("id"));`);

    this.addSql(`alter table "tunnel_sessions" add constraint "tunnel_sessions_user_id_foreign" foreign key ("user_id") references "tunnel_users" ("id") on update cascade;`);

    this.addSql(`drop table if exists "statistics" cascade;`);

    this.addSql(`drop table if exists "user" cascade;`);
  }

  override async down(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" drop constraint "tunnel_sessions_user_id_foreign";`);

    this.addSql(`create table "statistics" ("id" varchar(21) not null, "inbound" int8 not null, "outbound" int8 not null, constraint "statistics_pkey" primary key ("id"));`);

    this.addSql(`create table "user" ("id" varchar(21) not null, "username" text not null, "hashed_password" text not null, "salt" varchar(12) not null, constraint "user_pkey" primary key ("id"));`);
    this.addSql(`alter table "user" add constraint "user_username_unique" unique ("username");`);

    this.addSql(`drop table if exists "tunnel_users" cascade;`);

    this.addSql(`drop table if exists "tunnel_sessions" cascade;`);
  }

}
