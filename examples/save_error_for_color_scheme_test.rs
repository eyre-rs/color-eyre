
use color_eyre::{eyre::Report, Section};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct TestError(&'static str);

#[tracing::instrument]
fn get_errors(msg: &'static str) -> (Report, Report) {

    let create_report = |msg|
        Report::msg(msg)
            .note("note")
            .warning("warning")
            .suggestion("suggestion")
            .error(TestError("error"));

    // Getting regular `Report`. Using iterator to trigger `is_dependency_code`. See https://github.com/yaahc/color-eyre/blob/4ddaeb2126ed8b14e4e6aa03d7eef49eb8561cf0/src/config.rs#L56
    let r1 = std::iter::once(msg).map(create_report).last().unwrap();

    // Gettting `Report` of `panic`
    let x = std::panic::catch_unwind(|| panic!(Report::msg(msg))).unwrap_err();

    let r2 = if let Ok(r) = x.downcast::<Report>() {
        *r
    } else {
        unreachable!()
    };

    (r1, r2)
}
fn main(){
    setup();
    let (error1, error2) = get_errors("test");


    // If any line from line 1 til this line is changed, a saved error might stop to be compatible with a previously saved error (because line numbers and so on might have changed). 


    let error_file = "color_eyre_error.txt";
    let error_string = format!("{:?}\n\n{:?}", error1, error2);

    std::fs::write(error_file, &error_string)
        .expect("Error saving `error` to a file");

    println!("{}\n\n [The above was saved to {}]", &error_string, error_file);
}

fn setup() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    std::env::set_var("RUST_LIB_BACKTRACE", "full");

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    color_eyre::install().expect("Failed to install `color_eyre`");
}