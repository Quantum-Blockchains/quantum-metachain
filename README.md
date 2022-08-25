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

