BEGIN;

create table "tunnel_status"
(
    "id"                      bigserial primary key,
    "bucket_start"            timestamp not null,
    "sample_count"            bigint    not null,
    "active_service_avg"      bigint    not null,
    "active_service_max"      bigint    not null,
    "external_connection_avg" bigint    not null,
    "external_connection_max" bigint    not null
);

alter table "tunnel_status"
    add constraint "tunnel_status_bucket_start_unique" unique ("bucket_start");

INSERT INTO "mikro_orm_migrations" ("name", "executed_at")
VALUES ('Migration20260815064212', CURRENT_TIMESTAMP);

COMMIT;