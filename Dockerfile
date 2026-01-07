FROM rust:1.81.0 AS builder

WORKDIR /node

RUN apt-get update && apt-get -y install clang cmake protobuf-compiler

RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates

COPY bin bin
COPY pallets pallets
# COPY target target
COPY Cargo.lock .
COPY Cargo.toml .
# COPY rust-toolchain .

RUN rustup target add wasm32-unknown-unknown

ENV RUST_BACKTRACE=1
ENV CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_DEBUG=true

#CMD ["/bin/bash"]

RUN cargo build --release

# FROM python:3.11.6
FROM ubuntu:24.04

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV RUST_BACKTRACE=1

COPY --from=builder node/target/release/qmc-node /usr/local/bin/qmc-node