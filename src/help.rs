use crate::{Report, Result};
use ansi_term::Color::*;
use std::fmt::{self, Display, Write};

/// A helper trait for attaching help text to errors to be displayed after the chain of errors
///
/// `color_eyre` provides two types of help text that can be attached to error reports, custom
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
    /// use color_eyre::{Report, Help};
    /// use eyre::eyre;
    ///
    /// Err(eyre!("command failed"))
    ///     .section("Please report bugs to https://real.url/bugs")?;
    /// # Ok::<_, Report>(())
    /// ```
    fn section<C>(self, section: C) -> Result<T>
    where
        C: Into<Section>;

    /// Add a section to an error report, to be displayed after the chain of errors, which is
    /// lazily evaluated only in the case of an error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color_eyre::{Report, Help, SectionExt};
    /// use eyre::eyre;
    ///
    /// let output = std::process::Command::new("ls")
    ///     .output()?;
    ///
    /// let output = if !output.status.success() {
    ///     let stderr = String::from_utf8_lossy(&output.stderr);
    ///     Err(eyre!("cmd exited with non-zero status code"))
    ///         .with_section(move || {
    ///             "Stderr:"
    ///                 .skip_if(|| stderr.is_empty())
    ///                 .body(stderr.trim().to_string())
    ///         })?
    /// } else {
    ///     String::from_utf8_lossy(&output.stdout)
    /// };
    ///
    /// println!("{}", output);
    /// # Ok::<_, Report>(())
    /// ```
    fn with_section<C, F>(self, section: F) -> Result<T>
    where
        C: Into<Section>,
        F: FnOnce() -> C;

    /// Add a note to an error report, to be displayed after the chain of errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{error::Error, fmt::{self, Display}};
    /// # use color_eyre::Result;
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
    fn note<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;

    /// Add a Note to an error report, to be displayed after the chain of errors, which is lazily
    /// evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{error::Error, fmt::{self, Display}};
    /// # use color_eyre::Result;
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
    fn with_note<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    /// Add a Warning to an error report, to be displayed after the chain of errors.
    fn warning<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;

    /// Add a Warning to an error report, to be displayed after the chain of errors, which is lazily
    /// evaluated only in the case of an error.
    fn with_warning<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    /// Add a Suggestion to an error report, to be displayed after the chain of errors.
    fn suggestion<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;

    /// Add a Suggestion to an error report, to be displayed after the chain of errors, which is lazily
    /// evaluated only in the case of an error.
    fn with_suggestion<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

/// Extension trait for customizing the content of a `Section`
pub trait SectionExt {
    /// Add a body to a `Section`
    ///
    /// Bodies are always indented to the same level that error messages and spans are indented.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color_eyre::{Help, SectionExt, Report};
    /// use eyre::eyre;
    ///
    /// let all_in_header = "header\n   body\n   body";
    /// let report = Err::<(), Report>(eyre!("an error occurred"))
    ///     .section(all_in_header)
    ///     .unwrap_err();
    ///
    /// let just_header = "header";
    /// let just_body = "body\nbody";
    /// let report2 = Err::<(), Report>(eyre!("an error occurred"))
    ///     .section(just_header.body(just_body))
    ///     .unwrap_err();
    ///
    /// assert_eq!(format!("{:?}", report), format!("{:?}", report2))
    /// ```
    fn body<C>(self, body: C) -> Section
    where
        C: Display + Send + Sync + 'static;

    /// Skip printing `Section` if condition is true
    ///
    /// Useful for skipping sections based on the content of its body, even if the section header
    /// is not empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// use eyre::eyre;
    /// use color_eyre::{SectionExt, Report, Help};
    ///
    /// fn add_body(report: Report, body: String) -> Result<(), Report> {
    ///     Err(report)
    ///         .section("ExtraInfo:".skip_if(|| body.is_empty()).body(body))
    /// }
    ///
    /// let report = eyre!("an error occurred");
    /// let before = format!("{:?}", report);
    /// let body = String::new();
    /// let report = add_body(report, body).unwrap_err();
    /// let after = format!("{:?}", report);
    /// assert_eq!(before, after);
    ///
    /// let report = eyre!("an error occurred");
    /// let before = format!("{:?}", report);
    /// let body = String::from("Some actual text here");
    /// let report = add_body(report, body).unwrap_err();
    /// let after = format!("{:?}", report);
    /// assert_ne!(before, after);
    /// ```
    fn skip_if<F>(self, condition: F) -> Section
    where
        F: FnOnce() -> bool;
}

impl<T, E> Help<T> for std::result::Result<T, E>
where
    E: Into<Report>,
{
    fn note<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut().sections.push(
                Section::from(HelpInfo::Note(Box::new(context))).order(Order::AfterBackTrace),
            );
            e
        })
    }

    fn with_note<C, F>(self, context: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut().sections.push(
                Section::from(HelpInfo::Note(Box::new(context()))).order(Order::AfterBackTrace),
            );
            e
        })
    }

    fn warning<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut().sections.push(
                Section::from(HelpInfo::Warning(Box::new(context))).order(Order::AfterBackTrace),
            );
            e
        })
    }

    fn with_warning<C, F>(self, context: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut().sections.push(
                Section::from(HelpInfo::Warning(Box::new(context()))).order(Order::AfterBackTrace),
            );
            e
        })
    }

    fn suggestion<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut().sections.push(
                Section::from(HelpInfo::Suggestion(Box::new(context))).order(Order::AfterBackTrace),
            );
            e
        })
    }

    fn with_suggestion<C, F>(self, context: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut().sections.push(
                Section::from(HelpInfo::Suggestion(Box::new(context())))
                    .order(Order::AfterBackTrace),
            );
            e
        })
    }

    fn with_section<C, F>(self, section: F) -> Result<T>
    where
        C: Into<Section>,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let mut e = e.into();
            let section = section().into();

            if !matches!(section.order, Order::SkipEntirely) {
                e.context_mut().sections.push(section);
            }

            e
        })
    }

    fn section<C>(self, section: C) -> Result<T>
    where
        C: Into<Section>,
    {
        self.map_err(|e| {
            let mut e = e.into();
            let section = section.into();

            if !matches!(section.order, Order::SkipEntirely) {
                e.context_mut().sections.push(section);
            }

            e
        })
    }
}

impl<T> SectionExt for T
where
    Section: From<T>,
{
    fn body<C>(self, body: C) -> Section
    where
        C: Display + Send + Sync + 'static,
    {
        let mut section = Section::from(self);
        section.body = Some(Box::new(body));
        section
    }

    fn skip_if<F>(self, condition: F) -> Section
    where
        F: FnOnce() -> bool,
    {
        let mut section = Section::from(self);
        section.order = if condition() {
            Order::SkipEntirely
        } else {
            section.order
        };
        section
    }
}

pub(crate) enum HelpInfo {
    Note(Box<dyn Display + Send + Sync + 'static>),
    Warning(Box<dyn Display + Send + Sync + 'static>),
    Suggestion(Box<dyn Display + Send + Sync + 'static>),
}

/// A custom section for an error report.
///
/// # Details
///
/// Sections consist of two parts, a header, and an optional body. The header can contain any
/// number of lines and has no indentation applied to it by default. The body can contain any
/// number of lines and is always written after the header with indentation inserted before
/// every line.
///
/// # Construction
///
/// Sections are meant to be constructed via `Into<Section>`, which is implemented for all types
/// that implement `Display`. The constructed `Section` then takes ownership it the `Display` type
/// and boxes it internally for use later when printing the report.
///
/// # Examples
///
/// ```rust
/// use color_eyre::{SectionExt, Help, Report};
/// use eyre::eyre;
/// use std::process::Command;
/// use tracing::instrument;
///
/// trait Output {
///     fn output2(&mut self) -> Result<String, Report>;
/// }
///
/// impl Output for Command {
///     #[instrument]
///     fn output2(&mut self) -> Result<String, Report> {
///         let output = self.output()?;
///
///         let stdout = String::from_utf8_lossy(&output.stdout);
///
///         if !output.status.success() {
///             let stderr = String::from_utf8_lossy(&output.stderr);
///             Err(eyre!("cmd exited with non-zero status code"))
///                 .with_section(move || {
///                     "Stdout:"
///                         .skip_if(|| stdout.is_empty())
///                         .body(stdout.trim().to_string())
///                 })
///                 .with_section(move || {
///                     "Stderr:"
///                         .skip_if(|| stderr.is_empty())
///                         .body(stderr.trim().to_string())
///                 })
///         } else {
///             Ok(stdout.into())
///         }
///     }
/// }
/// ```
pub struct Section {
    header: Box<dyn Display + Send + Sync + 'static>,
    body: Option<Box<dyn Display + Send + Sync + 'static>>,
    pub(crate) order: Order,
}

impl Section {
    fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }
}

#[non_exhaustive]
///
#[derive(Debug)]
pub(crate) enum Order {
    ///
    AfterErrMsgs,
    ///
    AfterBackTrace,
    ///
    SkipEntirely,
}

impl Display for HelpInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Note(context) => write!(f, "{}: {}", Cyan.paint("Note"), context),
            Self::Warning(context) => write!(f, "{}: {}", Yellow.paint("Warning"), context),
            Self::Suggestion(context) => write!(f, "{}: {}", Cyan.paint("Suggestion"), context),
        }
    }
}

impl<T> From<T> for Section
where
    T: Display + Send + Sync + 'static,
{
    fn from(header: T) -> Self {
        let header = Box::new(header);

        Self {
            header,
            body: None,
            order: Order::AfterErrMsgs,
        }
    }
}

impl fmt::Debug for HelpInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Note(context) => f
                .debug_tuple("Note")
                .field(&format_args!("{}", context))
                .finish(),
            Self::Warning(context) => f
                .debug_tuple("Warning")
                .field(&format_args!("{}", context))
                .finish(),
            Self::Suggestion(context) => f
                .debug_tuple("Suggestion")
                .field(&format_args!("{}", context))
                .finish(),
        }
    }
}

impl fmt::Debug for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.header)?;
        if let Some(body) = &self.body {
            writeln!(f)?;
            write!(
                indenter::indented(f).with_format(indenter::Format::Uniform { indentation: "   " }),
                "{}",
                body
            )?;
        }

        Ok(())
    }
}

pub(crate) mod private {
    use crate::Report;
    pub trait Sealed {}

    impl<T, E> Sealed for std::result::Result<T, E> where E: Into<Report> {}
}
