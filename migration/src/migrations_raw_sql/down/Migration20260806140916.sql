BEGIN;

alter table "tunnel_sessions"
    drop constraint "tunnel_sessions_user_id_foreign";

drop table if exists "ip_blacklist" cascade;

drop table if exists "ip_whitelist" cascade;

drop table if exists "settings" cascade;

drop table if exists "tunnel_users" cascade;

drop table if exists "tunnel_sessions" cascade;

COMMIT;