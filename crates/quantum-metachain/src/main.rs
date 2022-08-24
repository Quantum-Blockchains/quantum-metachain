use chrono::Local;
use env_logger::fmt::Color;
use env_logger::{Builder, WriteStyle};
use log::{Level, LevelFilter};
use qmc_p2p::config::P2PConfiguration;
use qmc_p2p::config;
use qmc_p2p::service::{DevP2PService, P2PService};
use std::env;
use std::io::Write;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| {
            let mut level_style = buf.style();
            match record.level() {
                Level::Error => level_style.set_color(Color::Red),
                Level::Warn => level_style.set_color(Color::Yellow),
                Level::Info => level_style.set_color(Color::Green),
                Level::Debug => level_style.set_color(Color::Blue),
                Level::Trace => level_style.set_color(Color::Magenta),
            };
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                level_style.value(record.level()),
                record.args()
            )
        })
        .write_style(WriteStyle::Always);

    if env::var("RUST_LOG").is_err() {
        builder.filter(None, LevelFilter::Info);
    }

    builder.init();

    let p2p_config = P2PConfiguration {
        // TODO JEQB-79 read listen address from config
        listen_address: "/ip4/0.0.0.0/tcp/0".to_string(),
    let p2p_config = match config::new("./config/p2p/config.toml") {
        Ok(c) => c,
        Err(err) => panic!("Couldn't load config file: {:?}", err),
    };

    // basic printing until logger is introduced
    println!(
        "Found config file:\n- listen_address: {}\n--------",
        p2p_config.listen_address
    );

    let p2p_service = DevP2PService::new(p2p_config);
    match p2p_service.start().await {
        Ok(_) => {}
        Err(err) => panic!("Cannot start p2p service: {:?}", err),
    };
    Ok(())
}
