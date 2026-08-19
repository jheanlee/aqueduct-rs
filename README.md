# Aqueduct

A simple, secure TCP tunnel written in Rust.

Aqueduct lets you securely expose and access TCP services through a server with a public IP.

The [Aqueduct client](https://github.com/jheanlee/aqueduct-client-rs) is maintained in a separate repository.

## Quickstart

Download prebuilt binaries from [server releases](https://github.com/jheanlee/aqueduct-rs/releases)
or [client releases](https://github.com/jheanlee/aqueduct-client-rs/releases), or build from source for other platforms.

Docker images for the [server](https://hub.docker.com/r/jheanlee/aqueduct)
and [client](https://hub.docker.com/r/jheanlee/aqueduct-client) are also available.

To use Aqueduct, you need a server accessible to your external clients, a service you want to expose, and a machine that
can access both the server and the service. The Aqueduct client can run on the same machine as the service or on the
Aqueduct server itself, which can be useful when the service is accessible from the server through an overlay network.

### Binary

#### Server

The server requires an existing [PostgreSQL](https://www.postgresql.org/) database, a TLS certificate and private key,
and separate public/private key pairs for access and refresh JWTs.

1. Download the prebuilt binary from [server releases](https://github.com/jheanlee/aqueduct-rs/releases)
2. Copy the [server .env.example](.env.example) to `.env` and configure the values.
3. Apply database migrations:

```shell
./aqueduct migrate up
```

4. Initialize the database:

```shell
./aqueduct init
```

This creates the default admin user and initial settings. Change the generated password before using the server in
production.

```
2026-08-19T14:40:18.207567Z  INFO aqueduct::init::runner: Initialized database:
        -------- User --------
        Username: admin
        Password: password
        -------- Settings --------
        blacklist: enabled
        blacklist ips: none
        whitelist: disabled
        whitelist ips: none
2026-08-19T14:40:18.207629Z  WARN aqueduct::init::runner: IMPORTANT: please change the default password using the management web UI
```

5. Start the Aqueduct server.

```shell
./aqueduct
```

#### Client

1. Download the prebuilt binary from [client releases](https://github.com/jheanlee/aqueduct-client-rs/releases)
2. Copy the [client .env.example](https://github.com/jheanlee/aqueduct-client-rs/blob/master/.env.example) to
   `.env` and configure the values.
3. Start the Aqueduct client.

```shell
./aqueduct-client
```

If the client fails to connect due to an `InvalidCertificate` error, see [Troubleshooting](#troubleshooting).

## Usage & Configuration

### Logging

Run the binary with the environment variable `RUST_LOG` to customize log levels. Available levels are: `OFF`, `ERROR`,
`WARN`, `INFO`, `DEBUG` and `TRACE`. For advanced usage, please refer to
the [env_logger documentation](https://docs.rs/env_logger/).

### Server

| Environment Variable                      | Command Line Option                | Default Value     | Description                                                                                                                                                                                    |
|-------------------------------------------|------------------------------------|-------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `AQUEDUCT_BIND_ADDRESS`                   | `--bind-address`                   | `"0.0.0.0:30330"` | Address and port on which the tunnel control server listens                                                                                                                                    |
| `AQUEDUCT_API_BIND_ADDRESS`               | `--api-bind-address`               | `"0.0.0.0:30331"` | Address and port on which the API server listens and the web UI is served                                                                                                                      |  
| `AQUEDUCT_TUNNEL_ALLOWED_PORTS`           | `--tunnel-allowed-ports`           | `"51000-51999"`   | Ports to which Aqueduct is allowed to bind as tunnel service endpoints, separated by commas; ranges can be specified as `start-end`, inclusive                                                 |
| `AQUEDUCT_TUNNEL_GLOBAL_CONNECTION_LIMIT` | `--tunnel-global-connection-limit` | `16384`           | Maximum number of concurrent external connections globally. Connections exceeding this limit are queued                                                                                        |
| `AQUEDUCT_TUNNEL_CLIENT_CONNECTION_LIMIT` | `--tunnel-client-connection-limit` | `256`             | Maximum number of concurrent external connections per client. Connections exceeding this limit by up to the same amount are queued; connections beyond twice the limit are dropped immediately |
| `AQUEDUCT_TLS_CERTIFICATE_FILE`           | `--tls-certificate-file`           | `N/A` (required)  | TLS certificate                                                                                                                                                                                |
| `AQUEDUCT_TLS_PRIVATE_KEY_FILE`           | `--tls-private-key-file`           | `N/A` (required)  | TLS private key                                                                                                                                                                                |
| `AQUEDUCT_JWT_ACCESS_PRIVATE_KEY_FILE`    | `--jwt-access-private-key-file`    | `N/A` (required)  | Private key used to sign JWT access tokens. Requires the `api` feature (enabled by default)                                                                                                    |
| `AQUEDUCT_JWT_ACCESS_PUBLIC_KEY_FILE`     | `--jwt-access-public-key-file`     | `N/A` (required)  | Public key used to verify JWT access tokens. Requires the `api` feature (enabled by default)                                                                                                   |
| `AQUEDUCT_JWT_REFRESH_PRIVATE_KEY_FILE`   | `--jwt-refresh-private-key-file`   | `N/A` (required)  | Private key used to sign JWT refresh tokens. Requires the `api` feature (enabled by default)                                                                                                   |
| `AQUEDUCT_JWT_REFRESH_PUBLIC_KEY_FILE`    | `--jwt-refresh-public-key-file`    | `N/A` (required)  | Public key used to verify JWT refresh tokens. Requires the `api` feature (enabled by default)                                                                                                  |
| `AQUEDUCT_DB_HOST`                        | `--db-host`                        | `"127.0.0.1"`     | Hostname or address of the database to connect to                                                                                                                                              |
| `AQUEDUCT_DB_PORT`                        | `--db-port`                        | `5432`            | Port of the database to connect to                                                                                                                                                             |
| `AQUEDUCT_DB_NAME`                        | `--db-name`                        | `"aqueduct"`      | Name of the database to connect to                                                                                                                                                             |
| `AQUEDUCT_DB_USER`                        | `--db-user`                        | `"postgres"`      | User used to connect to the database                                                                                                                                                           |
| `AQUEDUCT_DB_PASSWORD`                    | `--db-password`                    | `""`              | Password used to connect to the database                                                                                                                                                       |

### Client

| Environment Variable | Command Line Option | Default Value                                          | Description                                                                                                              |
|----------------------|---------------------|--------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| `AQUEDUCT_HOST`      | `--host`            | `"127.0.0.1:30330"`                                    | Address and port of the Aqueduct server to connect to                                                                    |
| `AQUEDUCT_SERVICE`   | `--service`         | `"127.0.0.1:80"`                                       | Address and port of the local service to expose through the tunnel                                                       |
| `AQUEDUCT_USER`      | `--user`            | `N/A` (required with `password` if `token` is not set) | Username used to authenticate with the Aqueduct server                                                                   |
| `AQUEDUCT_PASSWORD`  | `--password`        | `N/A` (required with `user` if `token` is not set)     | Password used to authenticate with the Aqueduct server                                                                   |
| `AQUEDUCT_TOKEN`     | `--token`           | `N/A` (required if `user` or `password` is not set)    | Authentication token used to authenticate with the Aqueduct server; takes precedence over `user` and `password` when set |
| N/A                  | `--insecure-tls`    | `not set` (flag)                                       | Disables TLS certificate verification                                                                                    |

## Troubleshooting

### InvalidCertificate Error

If the server's TLS certificate cannot be verified, the client will fail to connect and return an
`InvalidCertificate` error. For example:

```
2026-08-19T13:39:31.747515Z ERROR aqueduct_client: Unable to connect to the server: Custom { kind: InvalidData, error: InvalidCertificate(UnknownIssuer) }
```

This can occur when using a self-signed certificate or a certificate signed by a CA that is not trusted by the client.
It can also occur if a CA certificate is incorrectly configured as the server's TLS certificate.

For testing, you can disable TLS certificate verification with the `--insecure-tls` flag. This disables verification of
the certificate's issuer, hostname, expiration, and other certificate properties. Do not use this flag in production.

## License

This project is licensed under the [Apache License 2.0](LICENSE).