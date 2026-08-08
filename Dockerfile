FROM node:24-trixie AS node-build

WORKDIR /webui

COPY webui/package.json webui/package-lock.json ./
RUN npm ci

COPY webui/ ./
RUN npm run build

FROM rust:1.97-trixie AS rust-build

WORKDIR /aqueduct

COPY Cargo.toml Cargo.lock ./
COPY core ./core
COPY entity ./entity
COPY migration/src/migrations_raw_sql ./migration/src/migrations_raw_sql
COPY --from=node-build /webui/dist ./webui/dist

RUN cargo build --profile release

FROM debian:trixie-slim

WORKDIR /aqueduct

COPY --from=rust-build /aqueduct/target/release/aqueduct-rs .

CMD ./aqueduct-rs