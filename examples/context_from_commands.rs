use color_eyre::{eyre, section::ContextFrom};
use tracing::instrument;

#[instrument]
fn main() -> eyre::Result<()> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();

    color_eyre::install()?;

    visit_the_shell()?;

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
fn visit_the_shell() -> eyre::Result<()> {
    let mut cmd = std::process::Command::new("bash");
    cmd.arg("-c")
        // uh oh, there's an extra ' in that string and bash isn't gonna like it!
        .arg("echo 'Hello bash, I hope you're doing well!'");
    let output = cmd.output().context_from(&cmd)?;
    if !output.status.success() {
        Err(eyre::eyre!("invalid bash command"))
            .context_from(&cmd)
            .context_from(&output)
    } else {
        // I know the command is invalid ;)
        Ok(())
    }
}
