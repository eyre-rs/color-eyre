//!
//! Try it out with cargo test --example cargo_tests -- --nocapture

use color_eyre::eyre::Result;
use eyre::bail;
use tracing::info;

fn do_some_work_well() -> Result<()> {
    info!("Doing some work.");
    Ok(())
}

fn do_some_work_badly() -> Result<()> {
    bail!("Something went wrong")
}

fn main() -> Result<()> {
    do_some_work_well()?;
    do_some_work_badly()?;
    Ok(())
}
/// # Introduction 
/// The following module requires copying the init_color_eyre() to every test
/// function that uses it. This is a bit of a pain, but it's not a big deal.
/// However, if you wanted to avoid it you could use the
/// [ctor crate method](#ctor-method) below .


#[cfg(test)]
mod tests {
    use super::*;

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
    #[should_panic]
    fn test_eyre_init() {
        #[cfg(not(with_ctor))]
        some_common_spot::init_color_eyre();
        do_some_work_badly().unwrap();
        assert!(true);
    }

    /// ## Ctor Method 
    /// You can run the tests below with the following command:
    /// ```rust
    /// RUSTFLAGS="--cfg with_ctor" cargo test --example cargo_tests -- --nocapture 
    /// ```
    #[cfg(with_ctor)]
    #[test]
    #[should_panic(expected = "Something went wrong")]
    fn no_need_to_call_init_color_eyre() {
        // do not have to call some_common_spot::init_color_eyre();
        do_some_work_badly().unwrap();
        assert!(true);
    }

    #[cfg(with_ctor)]
    #[ctor::ctor]
    /// Initializes color_eyre the tests run.
    /// Note that this approach assumes that other constructors that
    /// may panic, would still use the method above to initialize color_eyre
    /// because the order in which constructors execute is random.
    /// Which is why we still use the OnceCell guard.
    /// If you are certain that no other constructor is interested in 
    /// initializing color_eyre, you simply call the color_eyre::install() 
    /// directly, and then remove the OnceCell guard.
    fn __init_color_eyre_ctor() {
        some_common_spot::init_color_eyre();
    }


}