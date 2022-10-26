FROM rust:1.64-slim as builder

WORKDIR /build
RUN cargo init --bin --name agexample
COPY Cargo.lock Cargo.toml /build/

RUN cargo build --release
RUN rm src/*.rs && rm target/release/deps/agexample*

COPY src /build/src

RUN cargo build --release

# Debian 11 (bullseye)
FROM debian:11-slim

RUN apt update
RUN apt install curl -y

WORKDIR /app

ENV HOST "0.0.0.0"
ENV PORT 9090
COPY --from=builder /build/target/release/agexample /app/


CMD ["./agexample"]
