use chrono::Local;
use env_logger::fmt::Color;
use env_logger::{Builder, WriteStyle};
use log::{Level, LevelFilter};
use std::env;
use std::io::Write;

pub fn init() {
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
}
