//!
//! Try it out with cargo test --example cargo_tests -- --nocapture

use color_eyre::{eyre::Result};
use eyre::bail;
use tracing::{info, instrument};

#[instrument]
fn do_some_work_well() -> Result<()> {
    info!("Doing some work.");
    Ok(())
}

#[instrument]
fn do_some_work_badly() -> Result<()> {
    bail!("Something went wrong")
}

fn main() -> Result<()> {
    do_some_work_well()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // cargo test --example cargo_tests -- --nocapture
    // put mod somewhere accessible so that it can be used elsewhere.
    pub mod some_common_spot {
        use once_cell::sync::OnceCell;

        pub struct ColorEyreGuard(());
        static INIT_COLOR_EYRE: OnceCell<ColorEyreGuard> = OnceCell::new();

        pub fn init_color_eyre() {
            INIT_COLOR_EYRE.get_or_init(|| {
                if std::env::var("RUST_SPANTRACE").is_err() {
                    std::env::set_var("RUST_SPANTRACE", "0");
                }
                // Run with RUST_BACKTRACE=1 to see the backtrace. 
                if std::env::var("RUST_BACKTRACE").is_err() {
                    std::env::set_var("RUST_BACKTRACE", "0");
                }
                if std::env::var("COLOR_EYRE").is_err() {
                    std::env::set_var("COLOR_EYRE", "1");
                }
                if std::env::var("COLOR_EYRE").unwrap() == "1" {
                    color_eyre::install().expect("Failed to initialize color_eyre");
                }
                ColorEyreGuard(())
            });
        }
    }

    #[test]
    fn test_eyre_init() {
        some_common_spot::init_color_eyre();
        do_some_work_badly().unwrap();
        assert!(true);
    }

    #[test]
    fn test_without_colors_work_as_is() {
        some_common_spot::init_color_eyre();
        do_some_work_badly().unwrap();
        assert!(true);
    }

}