# HTTP Health Checker

The purpose of this tool is to have the lightest HTTP Health Checker as possible for container health checking.

This health checker should:

- Take a target URL as argument
- Return 0 on success
- Return 1 on failure
- Be compiled
- Be statically linked

## How To Build

As any other Cargo crate:

```bash
cargo build --release
```

This results in a 1.2 MB executable:

```bash
$ du -h target/release/http_health_checker
1,2M    target/release/http_health_checker
```

## How to use

To use the health checker via CLI, simply give it a target as argument.

It will return 0 on success:

```bash
$ ./http_health_checker "https://www.google.com/"
$ echo $?
0
```

And 1 on failure:

```bash
$ ./http_health_checker "https://random.nonsense.local/false/route/parameter"
$ echo $?
1
```

## Dockerfile

To enable the health checking in your container, add the following in your Dockerfile:

```dockerfile
FROM rust:1.85 AS healthcheck_builder

COPY http_health_checker/ /build/http_health_checker/
RUN cd /build/http_health_checker/ && \
    export RUSTFLAGS='-C target-feature=+crt-static' && \
    cargo build --release && \
    mv target/release/http_health_checker /opt/http_health_checker && \
    chmod 555 /opt/http_health_checker

FROM your_final_image

COPY --from=healthcheck_builder --chown=root:root /opt/http_health_checker /home/app/http_health_checker
EXPOSE 12345
HEALTHCHECK \
    --start-period=5s \
    --interval=10s \
    --timeout=10s \
    --retries=3 \
    CMD [ "/home/app/http_health_checker", "http://localhost:12345/ping" ]
```

This assumes that:

- Your API runs on port 12345
- The API health check path is `/ping`

You may use different arguments `HEALTHCHECK` depending on your needs.

### Full Dockerfile Example

Suppose you have and application `app` and the following directories:

```text
├── Cargo.toml
├── Dockerfile
├── http_health_checker
│   ├── Cargo.toml
│   └── src
│       └── main.rs
└── src
    └── main.rs
```

You can use the following Dockerfile to build you application and the health checker alongside it:

```dockerfile
FROM rust:1.85 AS healthcheck_builder

COPY http_health_checker/ /build/http_health_checker/
RUN cd /build/http_health_checker/ && \
    export RUSTFLAGS='-C target-feature=+crt-static' && \
    cargo build --release && \
    mv target/release/http_health_checker /opt/http_health_checker && \
    chmod 555 /opt/http_health_checker

FROM rust:1.85 AS app_builder

COPY src/ /build/src/
COPY Cargo.toml /build/Cargo.toml
RUN cd /build && \
    export RUSTFLAGS='-C target-feature=+crt-static' && \
    cargo build --release && \
    mv target/release/app /opt/app && \
    chmod 555 /opt/app


FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=healthcheck_builder --chown=root:root /opt/http_health_checker /home/app/http_health_checker
COPY --from=app_builder --chown=root:root /opt/app /home/app/app

USER nonroot
WORKDIR /home/app/bin
EXPOSE 12345

HEALTHCHECK \
    --start-period=5s \
    --interval=10s \
    --timeout=10s \
    --retries=3 \
    CMD [ "/home/app/http_health_checker", "http://localhost:12345/ping" ]
ENTRYPOINT [ "/home/app/app", "--port", "12345" ]

```
