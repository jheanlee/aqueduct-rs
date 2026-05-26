import { Migration } from "@mikro-orm/migrations";

export class Migration20260519032707 extends Migration {
  override async up(): Promise<void> {
    this.addSql(`drop table if exists "web_users" cascade;`);

    this.addSql(
      `alter table "tunnel_users" add column "administrator" boolean not null default false;`,
    );
    this.addSql(
      `alter table "tunnel_users" alter column "administrator" drop default ;`,
    );
    this.addSql(
      `alter table "tunnel_users" alter column "label" type text[] using (case when "label" = '' then '{}'::text[] else array["label"] end);`,
    );
  }

  override async down(): Promise<void> {
    this.addSql(
      `create table "web_users" ("id" varchar(21) not null, "username" text not null, "hashed_password" text not null, "salt" varchar(22) not null, "administrator" bool not null, constraint "web_users_pkey" primary key ("id"));`,
    );
    this.addSql(
      `alter table "web_users" add constraint "web_users_username_unique" unique ("username");`,
    );

    this.addSql(`alter table "tunnel_users" drop column "administrator";`);

    this.addSql(
      `alter table "tunnel_users" alter column "label" type text using array_to_string("label", ',');`,
    );
  }
}
