use tracing::level_filters::LevelFilter;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::app::App;

pub mod app;
pub mod event;
pub mod services;
pub mod tui;
pub mod ui;
pub mod widgets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging()?;
    let mut app = App {
        tick_rate: 250,
        ..Default::default()
    };

    app.run().await?;

    Ok(())
}

fn init_logging() -> anyhow::Result<()> {
    let log_file = std::fs::File::create("testlog.log")?;

    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(LevelFilter::TRACE);
    tracing_subscriber::registry().with(file_subscriber).init();
    Ok(())
}
