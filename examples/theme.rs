use color_eyre::{
    config::{ListStyle, Theme},
    eyre::Report,
    owo_colors::style,
    Section,
};

/// To experiment with theme values, edit `theme()` below and execute `cargo run --example theme`
fn theme() -> Theme {
    Theme::dark()
        // ^ use `new` to derive from a blank theme, or `light` to derive from a light theme.
        // Now configure your theme (see the docs for all options):
        .line_number(style().blue())
        .help_info_suggestion(style().red())
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct TestError(&'static str);

#[tracing::instrument]
fn get_error(msg: &'static str) -> Report {
    fn create_report(msg: &'static str) -> Report {
        let root = Report::msg(msg)
            .note("I left a note")
            .warning("this is a warning")
            .suggestion("and a suggestion")
            .error(TestError("error"));

        root.wrap_err("wrapping error")
    }

    // Using `Option` to add dependency code. See https://github.com/yaahc/color-eyre/blob/4ddaeb2126ed8b14e4e6aa03d7eef49eb8561cf0/src/config.rs#L56
    None::<Option<()>>
        .ok_or_else(|| create_report(msg))
        .unwrap_err()
}

fn main() {
    setup();
    println!("{:?}", get_error("test"));
}

fn setup() {
    std::env::set_var("RUST_LIB_BACKTRACE", "full");

    #[cfg(feature = "capture-spantrace")]
    {
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::{fmt, EnvFilter};

        let fmt_layer = fmt::layer().with_target(false);
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();

        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .with(tracing_error::ErrorLayer::default())
            .init();
    }

    color_eyre::config::HookBuilder::new()
        .theme(theme())
        .list_style(ListStyle::Prefix("-"))
        .install()
        .expect("Failed to install `color_eyre`");
}
