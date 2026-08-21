FROM node:24-trixie AS node-build

ARG DOCKER_BUILD=1

WORKDIR /webui

COPY webui/package.json webui/package-lock.json ./
RUN npm ci

COPY webui/ ./
RUN npm run build

FROM rust:1.97-trixie AS rust-build

ARG DOCKER_BUILD=1

WORKDIR /aqueduct

COPY Cargo.toml Cargo.lock ./
COPY core ./core
COPY entity ./entity
COPY migration/src/migrations_raw_sql ./migration/src/migrations_raw_sql
COPY --from=node-build /webui/dist ./webui/dist

RUN cargo build --release --locked

FROM debian:trixie-slim

COPY --from=rust-build /aqueduct/target/release/aqueduct /usr/local/bin/aqueduct

EXPOSE 30330 30331 51000-51999

ENTRYPOINT ["aqueduct"]