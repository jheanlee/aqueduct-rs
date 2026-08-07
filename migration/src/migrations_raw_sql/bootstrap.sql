BEGIN;

CREATE SEQUENCE IF NOT EXISTS mikro_orm_migrations_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO CYCLE;

CREATE TABLE IF NOT EXISTS mikro_orm_migrations
(
    id          integer NOT NULL DEFAULT nextval('mikro_orm_migrations_id_seq'::regclass),
    name        varchar(255),
    executed_at timestamptz      DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

ALTER SEQUENCE mikro_orm_migrations_id_seq
    OWNED BY mikro_orm_migrations.id;

COMMIT;