//! Nothing interesting here. This is just a small helper used in a test.

//! This needs to be an "example" until binaries can declare separate dependencies (see https://github.com/rust-lang/cargo/issues/1982)

//! See "tests/theme.rs" for more information.

use nocolor_eyre::{eyre::Report, Section};

#[rustfmt::skip]
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct TestError(&'static str);

#[rustfmt::skip]
fn get_error(msg: &'static str) -> Report {

    #[rustfmt::skip]
    #[inline(never)]
    fn create_report(msg: &'static str) -> Report {
        Report::msg(msg)
            .note("note")
            .warning("warning")
            .suggestion("suggestion")
            .error(TestError("error"))
    }

    // Getting regular `Report`. Using `Option` to trigger `is_dependency_code`. See https://github.com/yaahc/color-eyre/blob/4ddaeb2126ed8b14e4e6aa03d7eef49eb8561cf0/src/config.rs#L56
    None::<Option<()>>.ok_or_else(|| create_report(msg)).unwrap_err()
}

fn main() {
    setup();
    let msg = "test";
    let span = tracing::info_span!("get_error", msg);
    let _guard = span.enter();
    let error = get_error(msg);
    std::panic::panic_any(error)
}

fn setup() {
    std::env::set_var("RUST_BACKTRACE", "1");

    nocolor_eyre::install().expect("Failed to install `color_eyre`");
}
