# Packages a binary that the release workflow has already built, with the UI
# baked into it. Building inside the image would mean shipping a Rust
# toolchain, a wasm target, and Trunk in order to produce one file.
FROM ubuntu:26.04

# The only runtime dependency. SQLite is compiled in and TLS uses rustls, but
# reaching the identity provider still needs the system trust store.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

ADD rex /app/rex
RUN chmod +x /app/rex

# The database lives on a volume; the image itself stays stateless.
VOLUME /data
ENV REX_CONFIG=/config/config.toml

EXPOSE 8000

CMD [ "/app/rex" ]
