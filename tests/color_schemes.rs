
use color_eyre::{eyre::Report, Section};

#[rustfmt::skip]
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct TestError(&'static str);

#[rustfmt::skip]
#[tracing::instrument]
fn get_errors(msg: &'static str) -> (Report, Report) {

    #[rustfmt::skip]
    #[inline(never)]
    fn create_report(msg: &'static str) -> Report {
        Report::msg(msg)
            // An easy way to see if the test case below works, is to uncomment one of the following lines. After do that, the test case should fail.
            .note("note")
            .warning("warning")
            .suggestion("suggestion")
            .error(TestError("error"))
    }

    // Getting regular `Report`. Using `Option` to trigger `is_dependency_code`. See https://github.com/yaahc/color-eyre/blob/4ddaeb2126ed8b14e4e6aa03d7eef49eb8561cf0/src/config.rs#L56
    let r1 = None::<Option<()>>.ok_or_else(|| create_report("test")).unwrap_err();

    // Getting `Report` of `panic`
    let x = std::panic::catch_unwind(|| panic!(Report::msg(msg))).unwrap_err();

    let r2 = if let Ok(r) = x.downcast::<Report>() {
        *r
    } else {
        unreachable!()
    };

    (r1, r2)
}

#[rustfmt::skip]
#[test]
fn test_backwards_compatibility(){
    setup();
    let (error, panic_error) = get_errors("test");


    /*
        Note: If you change anything above this comment, it could make the stored test data invalid (because it could change something in regard to the generated error). In most cases, it shouldn't, but keep this in mind if you change something and suddenly this test case fails.

        If a change of the code above leads to incompatibility, you therefore have to backport this (changed) file to the version of `color_eyre` that you want to test against and execute it to generate new control test data.

        To do this, do the following:

        1) Change this file, and if the test now fails do:

        2) Checkout the `color_eyre` version from Git that you want to test against

        3) Add this test file to '/tests'

        4) If `error_file_path` or `panic_file_path` exist (see below), delete these files
        
        5) If you now run this test, it will generate test data files in the current working directory

        6) copy these files to `error_file_path` and `panic_file_path` in the current version of `color_eyre` (see the instructions that are printed out in step 5)

        Now this test shouldn't fail anymore in the current version.


        # How this test works:
    
        1) generate errors with the code above

        2) convert these error to string

        3) load stored error data to compare to (stored in `error_file_path` and `panic_file_path`)

        4) if `error_file_path` and/or `panic_file_path` doesn't exist, generate corresponding files in the current working directory and request the user to fix the issue (see below)

        5) extract ANSI escaping sequences (of controls and current errors)

        6) compare if the current error and the control contains the same ANSI escape sequences

        7) If not, fail and show the full strings of the control and the current error

    */

    use std::{fs, path::Path};

    let error_file_name = "color_scheme_error_control.txt";
    let error_file_path = ["tests/data/", error_file_name].concat();

    let panic_file_name = "color_scheme_panic_control.txt";
    let panic_file_path = ["tests/data/", panic_file_name].concat();

    let error_string = format!("{:?}", error);
    let panic_string = format!("{:?}", panic_error);

    // If `error_file` and/or `panic_file` are missing, save corresponding files to current working directory, and panic with the request to move these files to `error_file` and/or `panic_file`, and to commit them to Git. Being explicit (instead of saving directly to `error_file` and/or `panic_file`) to make sure `error_file` and/or `panic_file` are committed to Git. These files anyway should already exist.

    let mut fail = false;
    let mut failure_message = "Required test data missing! Fix this, by doing:\n".to_string();
    
    if !Path::new(&error_file_path).is_file() {
        std::fs::write(error_file_name, &error_string)
            .expect("Error saving `error` to a file");
        failure_message.push_str(&format!(
            "\n- move '{}'¹ to '{}', and commit it to Git\n",
            error_file_name, error_file_path
        ));
        fail = true;
    }
    
    if !Path::new(&panic_file_path).is_file() {
        std::fs::write(panic_file_name, &panic_string)
            .expect("Error saving `error` to a file");
        failure_message.push_str(&format!(
            "\n- move '{}'¹ to '{}', and commit it to Git\n",
            panic_file_name, panic_file_path
        ));
        fail = true;
    }

    if fail {
        failure_message.push_str(&format!("\n¹ just generated in the current working directory\n"));
        panic!(failure_message)
    }

    // these `unwraps` should never fail with files generated by this test
    let error_string_control = String::from_utf8(
        fs::read(error_file_path).unwrap()
    ).unwrap(); 

    // same as above
    let panic_string_control = String::from_utf8(
        fs::read(panic_file_path).unwrap()
    ).unwrap(); 

    use ansi_parser::{AnsiParser, Output, AnsiSequence};

    fn get_ansi<'a>(s: &'a str) -> impl Iterator<Item=AnsiSequence> + 'a {
        s.ansi_parse().filter_map(|x|
            if let Output::Escape(ansi) = x {
                Some(ansi)
            } else {
                None
            }
        )
    };

    let error_string_ansi = get_ansi(&error_string);
    let error_string_control_ansi = get_ansi(&error_string_control);

    let panic_string_ansi = get_ansi(&panic_string);
    let panic_string_control_ansi = get_ansi(&panic_string_control);

    assert!(
        error_string_ansi.eq(error_string_control_ansi),
        format!("`error` ANSI escape sequences are not identical to control!\n\nCONTROL ERROR:\n{}\n\nCURRENT ERROR:\n\n{}\n", &error_string, &error_string_control)
    );

    assert!(
        panic_string_ansi.eq(panic_string_control_ansi),
        format!("`panic` ANSI escape sequences are not identical to control!\n\nCONTROL ERROR:\n{}\n\nCURRENT ERROR:\n\n{}\n", &error_string, &error_string_control)
    );

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