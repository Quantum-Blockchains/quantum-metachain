# Quantum Meta-chain

This is a repository for Quantum Meta-chain, an implementation of a quantum node using quantum 
and post-quantum security. It is a fork of a rust-based repository, [Substrate](https://github.com/paritytech/substrate).

## Table of contents
- [1. Setup](#1-setup)
  - [1.1. Prerequisites](#11-prerequisites)
- [2. Building](#2-build)
  - [2.1. Using `cargo`](#21-using-cargo)
  - [2.2. Using Docker](#22-using-docker)
- [3. Running](#3-running)
  - [3.1. Using Python](#31-using-python)
  - [3.2. Using Docker](#32-using-docker)
- [4 - Testing](#4-testing)
  - [4.1 Runner unit tests](#41-runner-unit-tests)
  - [4.2 Rust unit tests](#42-rust-unit-tests)
  - [4.3 Key rotation tests](#43-key-rotation-tests)
- [5 - Documentation](#5-documentation)

## 1. Setup
### 1.1. Prerequisites 
To begin working with this repository you will need the following dependencies:
- [Rust](https://www.rust-lang.org/tools/install)
- [Python](https://www.python.org/downloads/)
- [Docker](https://docs.docker.com/engine/install/) (optional)

After downloading your dependencies you need to make sure to continue with these steps:
- Because this a substrate fork you will also need to configure Rust with few additional steps, listed [here](https://docs.substrate.io/install/)
by substrate team.
- Install Python dependencies:
```bash
python3 -m venv venv
. ./venv/bin/activate
pip3 install -r requirements.txt
```

## 2. Build
There are few ways to build this repository before running, listed below.

### 2.1. Using `cargo` 
Cargo is a tool provided by rust framework to easily manage building, running and testing rust code.
You can use it to build quantum node code with command:
```bash
cargo build --release
```
This will create a binary file in `./target/release`, called `qmc-node`.

### 2.2. Using Docker
Alternate way of building this repository uses Docker. To build a node use command:
```bash
docker build -t quantum-metachain .
```
This will create a `quantum-metachain` docker image.
Alternatively, you can also build it with `make`:
```bash
make build
```

## 3. Running
Depending on how you built your project you can run it in different ways

Before you start the node, you need to create a configuration file, the path to which must then be provided when you start the node.
Example:
```json
{
  "__type__": "Config",
  "local_peer_id": "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc",
  "local_server_port": 5001,
  "external_server_port": 5002,
  "psk_file_path": "test/tmp/alice/psk",
  "psk_sig_file_path": "test/tmp/alice/psk_sig",
  "node_key_file_path": "test/tmp/alice/node_key",
  "key_rotation_time": 50,
  "qrng_api_key": "",
  "node_logs_path": "test/tmp/alice/node.log",
  "qkd_cert_path": null,
  "qkd_cert_key_path": null,
  "peers": {
    "12D3KooWT1niMg9KUXFrcrworoNBmF9DTqaswSuDpdX8tBLjAvpW": {
      "qkd_addr": "http://localhost:8182/alice/bob",
      "server_addr": "http://localhost:5004"
    },
    "12D3KooWDNdLiaUM2161yCQMvZy9LVgP3fcySk8nuimcKMDBXryj": {
      "qkd_addr": "http://localhost:8182/alice/dave",
      "server_addr": "http://localhost:5008"
    }
  }
}
```

### 3.1. Using Python
Quantum Meta-chain introduces a concept of **Pre-shared key rotation**.
To make things work we introduced a system for managing rotating those keys called a **runner**.
Runner works as a wrapper around Rust-built code that rotates pre-shared keys after some period of time.
To run a Rust-built node run a following command:

```bash
python3 runner/app.py --config-file <config_path> --process './target/release/qmc-node \
--base-path /tmp/<node_name> \
--chain ./quantumMetachainSpecRaw.json \
--name <node_name> \
--port <port_number> \
--ws-port <port_number> \
--rpc-port <port_number> \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--rpc-methods Unsafe \
--no-mdns'
```
If you want to run a configuration f.e. for alice node, you would need to change configuration to that on of alice:
```bash
python3 runner/app.py --config-file ./path/to/alice/config.json --process './target/release/qmc-node \
--chain ./quantumMetachainSpecRaw.json \
--name Alice \

...
```
For a list of all available flags run your `qmc-node` with `--help` flag

### 3.2. Using Docker
To run a Docker container from docker image built in earlier steps run:
```bash
docker run -it quantum-metachain
```
Optionally you can type
```bash
make start
# and
make stop
```

## 4. Testing
There are few layers that need to be covered with testing suites:
- Quantum Meta-chain code
- Runner code
- Key rotating flow
Each of those layers have their separate way of writing/running tests

### 4.1 Runner unit tests
To run runner unit tests:
```bash
pytest ./runner/test --ignore=psk_rotation_test.py
```

### 4.2 Rust unit tests
To run QMC unit tests:
```bash
cargo test
```

### 4.3 Key rotation tests
To run integration key rotation tests:
```bash
cd runner
python3 psk_rotation_test.py
```
  

## 5. Documentation
To generate documentation run:
```bash
cargo doc
```

In order to display documentation go to `target/doc/<crate you want to see>` and open `index.html` file in the browser that you want to, e.g.
#### MAC

```bash
cd target/doc/qmc_node
open -a "Google Chrome" index.html
```

#### Linux

```bash
cd target/doc/qmc_node
firefox index.html
```
