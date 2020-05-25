//! example for manually testing the perf of color-eyre in debug vs release

use color_eyre::{Help, Report};
use eyre::WrapErr;
use tracing::{info, instrument};

#[instrument]
fn main() -> Result<(), Report> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();

    let report = read_config().unwrap_err();

    let start = std::time::Instant::now();
    let _ = format!("Error: {:?}", report);
    drop(report);
    let end = std::time::Instant::now();

    dbg!(end - start);

    Ok(())
}

#[cfg(feature = "capture-spantrace")]
fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

#[instrument]
fn read_file(path: &str) -> Result<(), Report> {
    info!("Reading file");
    Ok(std::fs::read_to_string(path).map(drop)?)
}

#[instrument]
fn read_config() -> Result<(), Report> {
    read_file("fake_file")
        .wrap_err("Unable to read config")
        .suggestion("try using a file that exists next time")
}
