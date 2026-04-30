import { Migration } from "@mikro-orm/migrations";

export class Migration20260430112439 extends Migration {
  override async up(): Promise<void> {
    this.addSql(
      `alter table "ip_blacklist" rename column "subnet" to "network";`,
    );
    this.addSql(
      `alter table "ip_blacklist" rename column "comment" to "notes";`,
    );
    this.addSql(
      `alter table "ip_blacklist" add constraint "ip_blacklist_no_ip_overlaps" exclude USING gist (network inet_ops WITH &&);`,
    );

    this.addSql(
      `alter table "ip_whitelist" rename column "subnet" to "network";`,
    );
    this.addSql(
      `alter table "ip_whitelist" rename column "comment" to "notes";`,
    );
    this.addSql(
      `alter table "ip_whitelist" add constraint "ip_whitelist_no_ip_overlaps" exclude USING gist (network inet_ops WITH &&);`,
    );

    this.addSql(
      `alter table "tunnel_sessions" alter column "tunnel_client" type inet using ("tunnel_client"::inet);`,
    );
  }

  override async down(): Promise<void> {
    this.addSql(`drop constraint "ip_blacklist_no_ip_overlaps";`);

    this.addSql(
      `alter table "ip_blacklist" rename column "network" to "subnet";`,
    );
    this.addSql(
      `alter table "ip_blacklist" rename column "notes" to "comment";`,
    );

    this.addSql(`drop constraint "ip_whitelist_no_ip_overlaps";`);

    this.addSql(
      `alter table "ip_whitelist" rename column "network" to "subnet";`,
    );
    this.addSql(
      `alter table "ip_whitelist" rename column "notes" to "comment";`,
    );

    this.addSql(
      `alter table "tunnel_sessions" alter column "tunnel_client" type varchar(255) using ("tunnel_client"::varchar(255));`,
    );
  }
}
