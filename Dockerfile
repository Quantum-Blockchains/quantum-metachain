FROM rustlang/rust:nightly as builder

WORKDIR /node

RUN apt-get update && apt-get -y install clang cmake protobuf-compiler
RUN rustup target add wasm32-unknown-unknown

COPY bin bin
COPY pallets pallets
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo build --release --target-dir target

FROM python:3.10.10-slim

WORKDIR /app

EXPOSE 30333 9944 9933 5002 5004

COPY --from=builder node/target/release/qmc-node qmc-node

COPY runner runner
COPY requirements.txt requirements.txt
RUN pip install -r requirements.txt

ENV PYTHONPATH "${PYTHONPATH}:/runner/app.py"