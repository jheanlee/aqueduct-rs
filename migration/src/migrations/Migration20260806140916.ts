import { Migration } from '@mikro-orm/migrations';

export class Migration20260806140916 extends Migration {

  override async up(): Promise<void> {
    this.addSql(`create table "ip_blacklist" ("id" serial primary key, "network" inet not null, "notes" text not null);`);
    this.addSql(`alter table "ip_blacklist" add constraint "ip_blacklist_no_ip_overlaps" exclude USING gist (network inet_ops WITH &&);`);

    this.addSql(`create table "ip_whitelist" ("id" serial primary key, "network" inet not null, "notes" text not null);`);
    this.addSql(`alter table "ip_whitelist" add constraint "ip_whitelist_no_ip_overlaps" exclude USING gist (network inet_ops WITH &&);`);

    this.addSql(`create table "settings" ("key" text not null, "value" text not null, constraint "settings_pkey" primary key ("key"));`);

    this.addSql(`create table "tunnel_users" ("id" varchar(21) not null, "username" text not null, "token" text not null, "hashed_password" text not null, "label" text[] not null, "last_login" timestamptz not null, "administrator" boolean not null, constraint "tunnel_users_pkey" primary key ("id"));`);
    this.addSql(`alter table "tunnel_users" add constraint "tunnel_users_username_unique" unique ("username");`);
    this.addSql(`alter table "tunnel_users" add constraint "tunnel_users_token_unique" unique ("token");`);

    this.addSql(`create table "tunnel_sessions" ("id" bigserial primary key, "user_id" varchar(21) null, "bucket_start" timestamptz not null, "tunnel_client" inet not null, "inbound" bigint not null, "outbound" bigint not null, "external_connection_count" bigint not null);`);
    this.addSql(`create index "idx_tunnel_sessions_bucket_start" on "tunnel_sessions" ("bucket_start");`);
    this.addSql(`create index "idx_tunnel_sessions_user_bucket_start" on "tunnel_sessions" ("user_id", "bucket_start");`);
    this.addSql(`alter table "tunnel_sessions" add constraint "tunnel_sessions_user_id_bucket_start_tunnel_client_unique" unique ("user_id", "bucket_start", "tunnel_client");`);

    this.addSql(`alter table "tunnel_sessions" add constraint "tunnel_sessions_user_id_foreign" foreign key ("user_id") references "tunnel_users" ("id") on update cascade on delete set null;`);
  }

  override async down(): Promise<void> {
    this.addSql(`alter table "tunnel_sessions" drop constraint "tunnel_sessions_user_id_foreign";`);

    this.addSql(`drop table if exists "ip_blacklist" cascade;`);

    this.addSql(`drop table if exists "ip_whitelist" cascade;`);

    this.addSql(`drop table if exists "settings" cascade;`);

    this.addSql(`drop table if exists "tunnel_users" cascade;`);

    this.addSql(`drop table if exists "tunnel_sessions" cascade;`);
  }

}
