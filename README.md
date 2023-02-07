# Quantum Meta Chain

## Build

### Runner dependencies
```bash
python3 -m venv venv
. ./venv/bin/activate
pip3 install -r requirements.txt
```

### Node dependencies
```bash
cargo build --release
```

## Run

Start the node using one of the names: alice, bob, charlie, dave, eve, ferdie.

```bash
python3 runner/app.py --config <config_path> ./target/release/qmc-node \
--base-path /tmp/<node_name> \
--chain ./quantumMetachainSpecRaw.json \
--name <node_name> \
--port <port_number> \
--ws-port <port_number> \
--rpc-port <port_number> \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--rpc-methods Unsafe \
--no-mdns
```

For example:

```bash
python3 runner/app.py --config config.json ./target/release/qmc-node \
--base-path /tmp/alice \
--chain ./quantumMetachainSpecRaw.json \
--name alice \
--port 30333 \
--ws-port 9945 \
--rpc-port 9933 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--rpc-methods Unsafe \
--no-mdns
```

## Docker

```bash
docker build -t quantum-metachain .
docker run -it quantum-metachain
```

## Test

### Unit tests

```bash
cargo test
```

### Key rotation testing

```bash
python3 runner/psk_rotation_test.py
```

## Documentation

### Generate

```bash
cargo doc
```

### Display

In order to display documentation go to `target/doc/<crate you want to see>` and open in the browser located there `index.html` file, e.g.

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
