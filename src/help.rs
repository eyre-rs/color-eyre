use crate::{Report, Result};
use ansi_term::Color::*;
use std::fmt::{self, Display, Write};

/// A helper trait for attaching help text to errors to be displayed after the chain of errors
pub trait Help<T>: private::Sealed {
    /// Add a section to an error report, to be displayed after the chain of errors.
    fn section<C>(self, section: C) -> Result<T>
    where
        C: Into<Section>;

    /// Add a section to an error report, to be displayed after the chain of errors, which is
    /// lazily evaluated only in the case of an error
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

/// Extension trait for customizing the content of a custom section
///
/// # Example
///
/// ```rust
/// use eyre::eyre;
/// use color_eyre::{Report, Help, SectionExt};
///
/// fn run_command() -> Result<(), Report> {
///     let stderr = "this command doesn't exist and this stderr output isn't real";
///
///     Err(eyre!("error running command"))
///         .with_section(|| "Stderr:".body(stderr))
/// }
/// ```
pub trait SectionExt {
    /// Add a body to a section
    ///
    /// Bodies are always indented to the same level that error messages and spans are indented.
    fn body<C>(self, body: C) -> Section
    where
        C: Display + Send + Sync + 'static;

    /// Skip printing section if condition is true
    ///
    /// Useful for skipping sections based on the content of its body, even if the section header
    /// is not empty
    ///
    /// # Examples
    ///
    /// By default `color-eyre` will skip sections if they're empty, to prevent unintentional
    /// newlines from sneaking into error reports. The following section would not end up changing
    /// the output for an error report:
    ///
    /// ```rust
    /// use eyre::eyre;
    /// use color_eyre::{Report, Help};
    ///
    /// let header = String::new();
    ///
    /// let base_report: Report = eyre!("an error occurred");
    /// let original_string = format!("{:?}", base_report);
    ///
    /// let res: Result<(), Report> = Err(base_report).section(header);
    /// let new_report = res.unwrap_err();
    /// let new_string = format!("{:?}", new_report);
    ///
    /// assert_eq!(original_string, new_string);
    /// ```
    ///
    /// However often times sections have are setup with a static string as the header, where the
    /// body itself is the only part that changes. `color-eyre` wont skip these sections by default
    /// even if the body is empty, which can result in undesireable cruft sneaking into your error
    /// reports.
    ///
    /// ```rust
    /// use eyre::eyre;
    /// use color_eyre::{SectionExt, Report, Help};
    ///
    /// let body = String::new();
    ///
    /// let base_report: Report = eyre!("an error occurred");
    /// let original_string = format!("{:?}", base_report);
    ///
    /// let res: Result<(), Report> = Err(base_report).section("ExtraInfo:".body(body));
    /// let new_report = res.unwrap_err();
    /// let new_string = format!("{:?}", new_report);
    ///
    /// assert_ne!(original_string, new_string);
    /// ```
    ///
    /// `skip_if` can be used to help with these situations, where you can set a flag within the
    /// generated `Section` that if set will prevent it from being pushed into the `Report`.
    ///
    /// ```rust
    /// use eyre::eyre;
    /// use color_eyre::{SectionExt, Report, Help};
    ///
    /// let body = String::new();
    ///
    /// let base_report: Report = eyre!("an error occurred");
    /// let original_string = format!("{:?}", base_report);
    ///
    /// let res: Result<(), Report> = Err(base_report)
    ///     .section("ExtraInfo:".skip_if(|| body.is_empty()).body(body));
    /// let new_report = res.unwrap_err();
    /// let new_string = format!("{:?}", new_report);
    ///
    /// assert_eq!(original_string, new_string);
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
            e.context_mut().help.push(HelpInfo::Note(Box::new(context)));
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
            e.context_mut()
                .help
                .push(HelpInfo::Note(Box::new(context())));
            e
        })
    }

    fn warning<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut()
                .help
                .push(HelpInfo::Warning(Box::new(context)));
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
            e.context_mut()
                .help
                .push(HelpInfo::Warning(Box::new(context())));
            e
        })
    }

    fn suggestion<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut()
                .help
                .push(HelpInfo::Suggestion(Box::new(context)));
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
            e.context_mut()
                .help
                .push(HelpInfo::Suggestion(Box::new(context())));
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

            if !section.should_skip {
                e.context_mut().custom_sections.push(section);
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

            if !section.should_skip {
                e.context_mut().custom_sections.push(section);
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
        section.should_skip = condition();
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
/// Sections are displayed in the order they are added to the error report. They are displayed
/// immediately after the `Error:` section and before the `SpanTrace` and `Backtrace` sections.
/// The body of the section is indented by default.
///
/// Sections consist of two parts, a header, and an optional body. The header can contain any
/// nuber of lines and has no indentation applied to it by default. The body can contain any
/// number of lines and is always written after the header with indentation inserted before
/// every line. The body also has trailling whitespace trimmed by default.
///
/// ```rust
/// use eyre::eyre;
/// use color_eyre::{Report, Help};
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
///                 .with_section(|| "Stdout:".skip_if(|| stdout.is_empty()).body(stdout))
///                 .with_section(|| "Stderr:".skip_if(|| stderr.is_empty()).body(stderr))
///         } else {
///             Ok(stdout.into())
///         }
///     }
/// }
/// ```
pub struct Section {
    header: Box<dyn Display + Send + Sync + 'static>,
    body: Option<Box<dyn Display + Send + Sync + 'static>>,
    should_skip: bool,
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
            should_skip: false,
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
