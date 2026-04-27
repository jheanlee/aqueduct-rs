import { Migration } from "@mikro-orm/migrations";

export class Migration20260427124151 extends Migration {
  override async up(): Promise<void> {
    this.addSql(
      `create table "ip_blacklist" ("id" serial primary key, "subnet" inet not null, "comment" text not null);`,
    );

    this.addSql(
      `create table "ip_whitelist" ("id" serial primary key, "subnet" inet not null, "comment" text not null);`,
    );

    this.addSql(
      `create table "settings" ("key" text not null, "value" text not null, constraint "settings_pkey" primary key ("key"));`,
    );

    this.addSql(
      `create table "tunnel_users" ("id" varchar(21) not null, "username" text not null, "token" text not null, "hashed_password" text not null, "label" text not null, "last_login" timestamp not null, constraint "tunnel_users_pkey" primary key ("id"));`,
    );
    this.addSql(
      `alter table "tunnel_users" add constraint "tunnel_users_username_unique" unique ("username");`,
    );
    this.addSql(
      `alter table "tunnel_users" add constraint "tunnel_users_token_unique" unique ("token");`,
    );

    this.addSql(
      `create table "tunnel_sessions" ("id" varchar(21) not null, "user_id" varchar(21) not null, "bucket_start" timestamp not null, "tunnel_client" varchar(255) not null, "inbound" bigint not null, "outbound" bigint not null, "external_connection_count" bigint not null, constraint "tunnel_sessions_pkey" primary key ("id"));`,
    );

    this.addSql(
      `create table "web_users" ("id" varchar(21) not null, "username" text not null, "hashed_password" text not null, "salt" varchar(22) not null, "administrator" boolean not null, constraint "web_users_pkey" primary key ("id"));`,
    );
    this.addSql(
      `alter table "web_users" add constraint "web_users_username_unique" unique ("username");`,
    );

    this.addSql(
      `alter table "tunnel_sessions" add constraint "tunnel_sessions_user_id_foreign" foreign key ("user_id") references "tunnel_users" ("id") on update cascade;`,
    );
  }

  override async down(): Promise<void> {
    this.addSql(
      `alter table "tunnel_sessions" drop constraint "tunnel_sessions_user_id_foreign";`,
    );

    this.addSql(`drop table if exists "ip_blacklist" cascade;`);

    this.addSql(`drop table if exists "ip_whitelist" cascade;`);

    this.addSql(`drop table if exists "settings" cascade;`);

    this.addSql(`drop table if exists "tunnel_users" cascade;`);

    this.addSql(`drop table if exists "tunnel_sessions" cascade;`);

    this.addSql(`drop table if exists "web_users" cascade;`);
  }
}
