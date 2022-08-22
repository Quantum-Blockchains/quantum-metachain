FROM rust:1.60 as builder

WORKDIR /rust

COPY crates crates
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo build --target-dir target

FROM ubuntu

COPY --from=builder /rust/target /target

ENTRYPOINT ["/target/debug/quantum-metachain"]
