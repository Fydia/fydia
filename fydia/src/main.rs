#![warn(missing_debug_implementations)]
#![deny(missing_docs)]
//! Top-level crate of fydia
use fydia_config::get_config;
use log::{Level, LevelFilter};
use pretty_env_logger::env_logger::fmt::{Color, Style, StyledValue};
use std::io::Write;

/// Start function
#[tokio::main]
async fn main() -> Result<(), ()> {
    pretty_env_logger::formatted_builder()
        .format(|f, record| {
            let mut style = f.style();
            let level = colored_level(&mut style, record.level());
            writeln!(
                f,
                "{}:{} {} [ {}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                level,
                record.args()
            )
        })
        .filter(Some("fydia"), LevelFilter::Info)
        .filter(Some("fydia-router"), LevelFilter::Info)
        .init();

    let config = get_config().await;

    axum::Server::bind(&(config.format_ip().as_str()).parse().unwrap())
        .serve(
            fydia_router::get_axum_router_from_config(config)
                .await
                .unwrap()
                .into_make_service(),
        )
        .await
        .unwrap();

    Ok(())
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO "),
        Level::Warn => style.set_color(Color::Yellow).value("WARN "),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
