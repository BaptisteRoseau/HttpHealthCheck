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
ARG target=x86_64-unknown-linux-gnu

FROM rust as healthcheck_builder

COPY health_checker/ /build/health_checker/
RUN cd /build/health_checker/ && \
    export RUSTFLAGS='-C target-feature=+crt-static' && \
    cargo build \
        --release \
        --target $target && \
    mv target/$target/release/health_checker /opt/health_checker && \
    chmod 555 /opt/health_checker

FROM your_final_image

COPY --from=healthcheck_builder --chown=root:root /opt/health_checker /home/app/health_checker
EXPOSE 12345
HEALTHCHECK \
    --start-period=5s \
    --interval=10s \
    --timeout=10s \
    --retries=3 \
    CMD [ "/path/to/health_checker", "http://localhost:12345/ping" ]
```

This assumes that:

- Your API runs on port 12345
- The API health check path is `/ping`

You may use different arguments `HEALTHCHECK` depending on your needs.

### Full Dockerfile Example

Suppose you have and application `application` and the following directories:

```text
├── Cargo.toml
├── Dockerfile
├── health_checker
│   ├── Cargo.toml
│   └── src
│      └── main.rs
└── src
    └── main.rs
```

You can use the following Dockerfile to build you application and the health checker alongside it:

```dockerfile
ARG target=x86_64-unknown-linux-gnu

FROM rust:1.85 as healthcheck_builder

COPY health_checker/ /build/health_checker/
RUN cd /build/health_checker/ && \
    export RUSTFLAGS='-C target-feature=+crt-static' && \
    cargo build \
        --release \
        --target $target && \
    mv target/$target/release/health_checker /opt/health_checker && \
    chmod 555 /opt/health_checker

FROM rust:1.85 as app_builder

COPY src/ /build/src/
COPY Cargo.toml /build/Cargo.toml
RUN cd /build && \
    export RUSTFLAGS='-C target-feature=+crt-static' && \
    cargo build \
        --release \
        --target $target && \
    mv target/$target/release/application /opt/application && \
    chmod 555 /opt/application


FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=builder --chown=root:root /opt/application /home/app/application
COPY --from=builder --chown=root:root /opt/health_checker /home/app/health_checker

USER nonroot
WORKDIR /home/app/bin
EXPOSE 12345

HEALTHCHECK \
    --start-period=5s \
    --interval=10s \
    --timeout=10s \
    --retries=3 \
    CMD [ "/home/app/health_checker", "http://localhost:12345/ping" ]
ENTRYPOINT [ "/home/app/application", "--port", "12345" ]
```
