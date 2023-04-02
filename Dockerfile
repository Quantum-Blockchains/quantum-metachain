FROM rustlang/rust:nightly as builder

WORKDIR /node

RUN apt-get update && apt-get -y install clang cmake protobuf-compiler

COPY bin bin
COPY pallets pallets
COPY target target
COPY Cargo.lock .
COPY Cargo.toml .
COPY rust-toolchain .

RUN rustup target add wasm32-unknown-unknown

ENV RUST_BACKTRACE=1
ENV CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_DEBUG=true

#CMD ["/bin/bash"]

RUN cargo build --release --target-dir target

FROM python:3.10.10-slim

WORKDIR /app

EXPOSE 30333 9944 9933 5002

COPY --from=builder node/target/release/qmc-node qmc-node

COPY runner runner
COPY requirements.txt requirements.txt
RUN pip install -r requirements.txt

ENV PYTHONPATH "${PYTHONPATH}:/runner/app.py"