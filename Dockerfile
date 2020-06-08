FROM rust:1

WORKDIR /src

# Pre-build all dependencies
RUN USER=root cargo init --bin --name rex
COPY ./Cargo.lock .
COPY ./Cargo.toml .
RUN cargo build --release
RUN rm src/*.rs
RUN rm ./target/release/deps/rex*

# Add the source code
COPY . .

# Run the test suite
RUN cargo test --release
RUN rm /src/target/release/deps/rex*


# Build the rest of the project
RUN cargo build --release --bin rex --features "table_storage"

# Ensure that the binary is at a known location for the next stage
RUN rm /src/target/release/deps/rex*.d
RUN cp /src/target/release/deps/rex* /src/target/release/deps/rex

FROM debian:buster-slim
#RUN apt-get update && apt-get install -y extra-runtime-dependencies

RUN apt-get update && apt-get install -y libssl1.1 ca-certificates

COPY --from=0 /src/target/release/deps/rex /app/rex

WORKDIR /app
CMD [ "/app/rex" ]