FROM rustlang/rust:nightly as builder

EXPOSE 30333 9933 9944

WORKDIR /app

RUN apt-get update && apt-get -y install clang cmake protobuf-compiler
RUN rustup target add wasm32-unknown-unknown

COPY bin bin
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo build --release --target-dir target

FROM ubuntu

COPY --from=builder app/target/release/qmc-node /usr/local/bin
RUN chmod +x /usr/local/bin/qmc-node

ENTRYPOINT ["/usr/local/bin/qmc-node"]
