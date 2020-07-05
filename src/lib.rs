//! An error report handler for panics and the [`eyre`] crate for colorful, consistent, and well
//! formatted error reports for all kinds of errors.
//!
//! ## TLDR
//!
//! `color_eyre` helps you build error reports that look like this:
//!
//! ![custom section example](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/custom_section.png)
//!
//! ## Setup
//!
//! Add the following to your toml file:
//!
//! ```toml
//! [dependencies]
//! color-eyre = "0.5"
//! ```
//!
//! ### Disabling tracing support
//!
//! If you don't plan on using `tracing_error` and `SpanTrace` you can disable the
//! tracing integration to cut down on unused dependencies:
//!
//! ```toml
//! [dependencies]
//! color-eyre = { version = "0.5", default-features = false }
//! ```
//!
//! ### Disabling SpanTrace capture by default
//!
//! color-eyre defaults to capturing span traces. This is because `SpanTrace`
//! capture is significantly cheaper than `Backtrace` capture. However, like
//! backtraces, span traces are most useful for debugging applications, and it's
//! not uncommon to want to disable span trace capture by default to keep noise out
//! of error messages intended for users of an application rather than the
//! developer.
//!
//! To disable span trace capture you must explicitly set one of the env variables
//! that regulate `SpanTrace` capture to `"0"`:
//!
//! ```rust
//! if std::env::var("RUST_SPANTRACE").is_err() {
//!     std::env::set_var("RUST_SPANTRACE", "0");
//! }
//! ```
//!
//! ### Improving perf on debug builds
//!
//! In debug mode `color-eyre` behaves noticably worse than `eyre`. This is caused
//! by the fact that `eyre` uses `std::backtrace::Backtrace` instead of
//! `backtrace::Backtrace`. The std version of backtrace is precompiled with
//! optimizations, this means that whether or not you're in debug mode doesn't
//! matter much for how expensive backtrace capture is, it will always be in the
//! 10s of milliseconds to capture. A debug version of `backtrace::Backtrace`
//! however isn't so lucky, and can take an order of magnitude more time to capture
//! a backtrace compared to it's std counterpart.
//!
//! Cargo [profile
//! overrides](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides)
//! can be used to mitigate this problem. By configuring your project to always
//! build `backtrace` with optimizations you should get the same performance from
//! `color-eyre` that you're used to with `eyre`. To do so add the following to
//! your Cargo.toml:
//!
//! ```toml
//! [profile.dev.package.backtrace]
//! opt-level = 3
//! ```
//!
//! ## Features
//!
//! ### Multiple report format verbosity levels
//!
//! `color-eyre` provides 3 different report formats for how it formats the captured `SpanTrace`
//! and `Backtrace`, minimal, short, and full. Take the below screenshots of the output produced by [`examples/usage.rs`]:
//!
//! ---
//!
//! Running `cargo run --example usage` without `RUST_LIB_BACKTRACE` set will produce a minimal
//! report like this:
//!
//! ![minimal report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/minimal.png)
//!
//! <br>
//!
//! Running `RUST_LIB_BACKTRACE=1 cargo run --example usage` tells `color-eyre` to use the short
//! format, which additionally capture a [`backtrace::Backtrace`]:
//!
//! ![short report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/short.png)
//!
//! <br>
//!
//! Finally, running `RUST_LIB_BACKTRACE=full cargo run --example usage` tells `color-eyre` to use
//! the full format, which in addition to the above will attempt to include source lines where the
//! error originated from, assuming it can find them on the disk.
//!
//! ![full report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/full.png)
//!
//! ### Custom `Section`s for error reports via [`Help`] trait
//!
//! The `section` module provides helpers for adding extra sections to error
//! reports. Sections are disinct from error messages and are displayed
//! independently from the chain of errors. Take this example of adding sections
//! to contain `stderr` and `stdout` from a failed command, taken from
//! [`examples/custom_section.rs`]:
//!
//! ```rust
//! use color_eyre::{eyre::eyre, SectionExt, Help, eyre::Report};
//! use std::process::Command;
//! use tracing::instrument;
//!
//! trait Output {
//!     fn output2(&mut self) -> Result<String, Report>;
//! }
//!
//! impl Output for Command {
//!     #[instrument]
//!     fn output2(&mut self) -> Result<String, Report> {
//!         let output = self.output()?;
//!
//!         let stdout = String::from_utf8_lossy(&output.stdout);
//!
//!         if !output.status.success() {
//!             let stderr = String::from_utf8_lossy(&output.stderr);
//!             Err(eyre!("cmd exited with non-zero status code"))
//!                 .with_section(move || stdout.trim().to_string().header("Stdout:"))
//!                 .with_section(move || stderr.trim().to_string().header("Stderr:"))
//!         } else {
//!             Ok(stdout.into())
//!         }
//!     }
//! }
//! ```
//!
//! ---
//!
//! Here we have an function that, if the command exits unsuccessfully, creates a
//! report indicating the failure and attaches two sections, one for `stdout` and
//! one for `stderr`.
//!
//! Running `cargo run --example custom_section` shows us how these sections are
//! included in the output:
//!
//! ![custom section example](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/custom_section.png)
//!
//! Only the `Stderr:` section actually gets included. The `cat` command fails,
//! so stdout ends up being empty and is skipped in the final report. This gives
//! us a short and concise error report indicating exactly what was attempted and
//! how it failed.
//!
//! ### Aggregating multiple errors into one report
//!
//! It's not uncommon for programs like batched task runners or parsers to want
//! to return an error with multiple sources. The current version of the error
//! trait does not support this use case very well, though there is [work being
//! done](https://github.com/rust-lang/rfcs/pull/2895) to improve this.
//!
//! For now however one way to work around this is to compose errors outside the
//! error trait. `color-eyre` supports such composition in its error reports via
//! the `Help` trait.
//!
//! For an example of how to aggregate errors check out [`examples/multiple_errors.rs`].
//!
//! ### Custom configuration for `color-backtrace` for setting custom filters and more
//!
//! The pretty printing for backtraces and span traces isn't actually provided by
//! `color-eyre`, but instead comes from its dependencies [`color-backtrace`] and
//! [`color-spantrace`]. `color-backtrace` in particular has many more features
//! than are exported by `color-eyre`, such as customized color schemes, panic
//! hooks, and custom frame filters. The custom frame filters are particularly
//! useful when combined with `color-eyre`, so to enable their usage we provide
//! the `install` fn for setting up a custom `BacktracePrinter` with custom
//! filters installed.
//!
//! For an example of how to setup custom filters, check out [`examples/custom_filter.rs`].
//!
//! [`eyre`]: https://docs.rs/eyre
//! [`tracing-error`]: https://docs.rs/tracing-error
//! [`color-backtrace`]: https://docs.rs/color-backtrace
//! [`eyre::EyreHandler`]: https://docs.rs/eyre/*/eyre/trait.EyreHandler.html
//! [`backtrace::Backtrace`]: https://docs.rs/backtrace/*/backtrace/struct.Backtrace.html
//! [`tracing_error::SpanTrace`]: https://docs.rs/tracing-error/*/tracing_error/struct.SpanTrace.html
//! [`color-spantrace`]: https://github.com/yaahc/color-spantrace
//! [`Help`]: https://docs.rs/color-eyre/*/color_eyre/trait.Help.html
//! [`eyre::Report`]: https://docs.rs/eyre/*/eyre/struct.Report.html
//! [`eyre::Result`]: https://docs.rs/eyre/*/eyre/type.Result.html
//! [`Handler`]: https://docs.rs/color-eyre/*/color_eyre/struct.Handler.html
//! [`examples/usage.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/usage.rs
//! [`examples/custom_filter.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/custom_filter.rs
//! [`examples/custom_section.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/custom_section.rs
//! [`examples/multiple_errors.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/multiple_errors.rs
#![doc(html_root_url = "https://docs.rs/color-eyre/0.5.0")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    missing_doc_code_examples,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
#![allow(clippy::try_err)]
use backtrace::Backtrace;
pub use eyre;
#[doc(hidden)]
pub use eyre::Report;
#[doc(hidden)]
pub use eyre::Result;
use once_cell::sync::OnceCell;
use section::help::HelpInfo;
pub use section::{Section, SectionExt};
use std::fmt::Display;
#[cfg(feature = "capture-spantrace")]
use tracing_error::SpanTrace;
#[doc(hidden)]
pub use Handler as Context;

pub mod config;
mod handler;
pub(crate) mod private;
pub mod section;
mod writers;

/// A custom handler type for [`eyre::Report`] which provides colorful error
/// reports and [`tracing-error`] support.
///
/// This type is not intended to be used directly, prefer using it via the
/// [`color_eyre::Report`] and [`color_eyre::Result`] type aliases.
///
/// [`eyre::Report`]: https://docs.rs/eyre/*/eyre/struct.Report.html
/// [`tracing-error`]: https://docs.rs/tracing-error
/// [`color_eyre::Report`]: type.Report.html
/// [`color_eyre::Result`]: type.Result.html
#[derive(Debug)]
pub struct Handler {
    backtrace: Option<Backtrace>,
    #[cfg(feature = "capture-spantrace")]
    span_trace: Option<SpanTrace>,
    sections: Vec<HelpInfo>,
}

static CONFIG: OnceCell<config::Printer> = OnceCell::new();

/// A helper trait for attaching informational sections to error reports to be
/// displayed after the chain of errors
///
/// `color_eyre` provides two types of help text that can be attached to error reports: custom
/// sections and pre-configured sections. Custom sections are added via the `section` and
/// `with_section` methods, and give maximum control over formatting. For more details check out
/// the docs for [`Section`].
///
/// The pre-configured sections are provided via `suggestion`, `warning`, and `note`. These
/// sections are displayed after all other sections with no extra newlines between subsequent Help
/// sections. They consist only of a header portion and are prepended with a colored string
/// indicating the kind of section, e.g. `Note: This might have failed due to ..."
///
/// [`Section`]: struct.Section.html
pub trait Help<T>: private::Sealed {
    /// Add a section to an error report, to be displayed after the chain of errors.
    ///
    /// Sections are displayed in the order they are added to the error report. They are displayed
    /// immediately after the `Error:` section and before the `SpanTrace` and `Backtrace` sections.
    /// They consist of a header and an optional body. The body of the section is indented by
    /// default.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use color_eyre::{eyre::eyre, eyre::Report, Help};
    ///
    /// Err(eyre!("command failed"))
    ///     .section("Please report bugs to https://real.url/bugs")?;
    /// # Ok::<_, Report>(())
    /// ```
    fn section<D>(self, section: D) -> eyre::Result<T>
    where
        D: Display + Send + Sync + 'static;

    /// Add a Section to an error report, to be displayed after the chain of errors. The closure to
    /// create the Section is lazily evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color_eyre::{eyre::eyre, eyre::Report, Help, SectionExt};
    ///
    /// let output = std::process::Command::new("ls")
    ///     .output()?;
    ///
    /// let output = if !output.status.success() {
    ///     let stderr = String::from_utf8_lossy(&output.stderr);
    ///     Err(eyre!("cmd exited with non-zero status code"))
    ///         .with_section(move || stderr.trim().to_string().header("Stderr:"))?
    /// } else {
    ///     String::from_utf8_lossy(&output.stdout)
    /// };
    ///
    /// println!("{}", output);
    /// # Ok::<_, Report>(())
    /// ```
    fn with_section<D, F>(self, section: F) -> eyre::Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Add an error section to an error report, to be displayed after the primary error message
    /// section.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use color_eyre::{eyre::eyre, eyre::Report, Help};
    /// use thiserror::Error;
    ///
    /// #[derive(Debug, Error)]
    /// #[error("{0}")]
    /// struct StrError(&'static str);
    ///
    /// Err(eyre!("command failed"))
    ///     .error(StrError("got one error"))
    ///     .error(StrError("got a second error"))?;
    /// # Ok::<_, Report>(())
    /// ```
    fn error<E>(self, error: E) -> eyre::Result<T>
    where
        E: std::error::Error + Send + Sync + 'static;

    /// Add an error section to an error report, to be displayed after the primary error message
    /// section. The closure to create the Section is lazily evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use color_eyre::{eyre::eyre, eyre::Report, Help};
    /// use thiserror::Error;
    ///
    /// #[derive(Debug, Error)]
    /// #[error("{0}")]
    /// struct StringError(String);
    ///
    /// Err(eyre!("command failed"))
    ///     .with_error(|| StringError("got one error".into()))
    ///     .with_error(|| StringError("got a second error".into()))?;
    /// # Ok::<_, Report>(())
    /// ```
    fn with_error<E, F>(self, error: F) -> eyre::Result<T>
    where
        F: FnOnce() -> E,
        E: std::error::Error + Send + Sync + 'static;

    /// Add a Note to an error report, to be displayed after the chain of errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{error::Error, fmt::{self, Display}};
    /// # use color_eyre::eyre::Result;
    /// # #[derive(Debug)]
    /// # struct FakeErr;
    /// # impl Display for FakeErr {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "FakeErr")
    /// #     }
    /// # }
    /// # impl std::error::Error for FakeErr {}
    /// # fn main() -> Result<()> {
    /// # fn fallible_fn() -> Result<(), FakeErr> {
    /// #       Ok(())
    /// # }
    /// use color_eyre::Help as _;
    ///
    /// fallible_fn().note("This might have failed due to ...")?;
    /// # Ok(())
    /// # }
    /// ```
    fn note<D>(self, note: D) -> eyre::Result<T>
    where
        D: Display + Send + Sync + 'static;

    /// Add a Note to an error report, to be displayed after the chain of errors. The closure to
    /// create the Note is lazily evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{error::Error, fmt::{self, Display}};
    /// # use color_eyre::eyre::Result;
    /// # #[derive(Debug)]
    /// # struct FakeErr;
    /// # impl Display for FakeErr {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "FakeErr")
    /// #     }
    /// # }
    /// # impl std::error::Error for FakeErr {}
    /// # fn main() -> Result<()> {
    /// # fn fallible_fn() -> Result<(), FakeErr> {
    /// #       Ok(())
    /// # }
    /// use color_eyre::Help as _;
    ///
    /// fallible_fn().with_note(|| {
    ///         format!("This might have failed due to ... It has failed {} times", 100)
    ///     })?;
    /// # Ok(())
    /// # }
    /// ```
    fn with_note<D, F>(self, f: F) -> eyre::Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Add a Warning to an error report, to be displayed after the chain of errors.
    fn warning<D>(self, warning: D) -> eyre::Result<T>
    where
        D: Display + Send + Sync + 'static;

    /// Add a Warning to an error report, to be displayed after the chain of errors. The closure to
    /// create the Warning is lazily evaluated only in the case of an error.
    fn with_warning<D, F>(self, f: F) -> eyre::Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Add a Suggestion to an error report, to be displayed after the chain of errors.
    fn suggestion<D>(self, suggestion: D) -> eyre::Result<T>
    where
        D: Display + Send + Sync + 'static;

    /// Add a Suggestion to an error report, to be displayed after the chain of errors. The closure
    /// to create the Suggestion is lazily evaluated only in the case of an error.
    fn with_suggestion<D, F>(self, f: F) -> eyre::Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;
}

// TODO: remove when / if ansi_term merges these changes upstream
trait ColorExt {
    fn make_intense(self) -> Self;
}

/// Install the default panic and error report hooks
pub fn install() -> Result<(), crate::eyre::Report> {
    config::HookBuilder::default()
        .add_default_filters()
        .install()
}
