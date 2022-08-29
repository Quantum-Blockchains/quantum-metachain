# Quantum Meta Chain

## Configuration

Configuration is stored inside `config/your_lib/config.toml`. Before running this repo copy fields from `config.toml.dist`
to `config.toml` in corresponding folder and fill them with correct data.

## Run
```
cargo run
```
By default, logger outputs error, warn and info level logs to the console. To output debug level logs:
```
RUST_LOG=debug cargo run
```
To output trace level logs:
```
RUST_LOG=trace cargo run
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
