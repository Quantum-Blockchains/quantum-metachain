# Quantum Meta Chain

## Build
```
cargo build --release
```

## Run
### Alice node
```
./target/release/qmc-node \
--base-path /tmp/alice \
--chain local \
--alice \
--port 30333 \
--ws-port 9945 \
--rpc-port 9933 \
--node-key 0000000000000000000000000000000000000000000000000000000000000001 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator
```

### Bob node
```
./target/release/qmc-node \
--base-path /tmp/bob \
--chain local \
--bob \
--port 30334 \
--ws-port 9946 \
--rpc-port 9934 \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

## Docker
```
docker build -t quantum-metachain .
docker run -it quantum-metachain
```

## Test

### Unit tests
```
cargo test
```

## Documentation
### Generate
```
cargo doc
```
### Display
In order to display documentation go to `target/doc/<crate you want to see>` and open in the browser located there `index.html` file, e.g.
#### MAC
```
cd target/doc/qmc_p2p
open -a "Google Chrome" index.html
```
#### Linux
```
cd target/doc/qmc_p2p
firefox index.html
```
