use crate::config::{lib_verbosity, panic_verbosity, Verbosity};
use fmt::Write;
use std::fmt::{self, Display};
#[cfg(feature = "capture-spantrace")]
use tracing_error::{SpanTrace, SpanTraceStatus};

#[allow(explicit_outlives_requirements)]
pub(crate) struct HeaderWriter<'a, H, W>
where
    H: ?Sized,
{
    inner: W,
    header: &'a H,
    started: bool,
}

pub(crate) trait WriterExt: Sized {
    fn header<H: ?Sized>(self, header: &H) -> HeaderWriter<'_, H, Self>;
}

impl<W> WriterExt for W {
    fn header<H: ?Sized>(self, header: &H) -> HeaderWriter<'_, H, Self> {
        HeaderWriter {
            inner: self,
            header,
            started: false,
        }
    }
}

pub(crate) struct ReadyHeaderWriter<'a, 'b, H: ?Sized, W>(&'b mut HeaderWriter<'a, H, W>);

impl<'a, H: ?Sized, W> HeaderWriter<'a, H, W> {
    pub(crate) fn ready(&mut self) -> ReadyHeaderWriter<'a, '_, H, W> {
        self.started = false;

        ReadyHeaderWriter(self)
    }

    pub(crate) fn in_progress(&mut self) -> ReadyHeaderWriter<'a, '_, H, W> {
        self.started = true;

        ReadyHeaderWriter(self)
    }
}

impl<'a, H: ?Sized, W> fmt::Write for ReadyHeaderWriter<'a, '_, H, W>
where
    H: Display,
    W: fmt::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !self.0.started && !s.is_empty() {
            self.0.inner.write_fmt(format_args!("{}", self.0.header))?;
            self.0.started = true;
        }

        self.0.inner.write_str(s)
    }
}

#[cfg(feature = "capture-spantrace")]
pub(crate) struct FormattedSpanTrace<'a>(pub(crate) &'a SpanTrace);

#[cfg(feature = "capture-spantrace")]
impl fmt::Display for FormattedSpanTrace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use indenter::indented;
        use indenter::Format;

        if self.0.status() == SpanTraceStatus::CAPTURED {
            write!(
                indented(f).with_format(Format::Uniform { indentation: "  " }),
                "{}",
                color_spantrace::colorize(self.0)
            )?;
        }

        Ok(())
    }
}

pub(crate) struct EnvSection<'a> {
    pub(crate) bt_captured: &'a bool,
    #[cfg(feature = "capture-spantrace")]
    pub(crate) span_trace: Option<&'a SpanTrace>,
}

impl fmt::Display for EnvSection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = if std::thread::panicking() {
            panic_verbosity()
        } else {
            lib_verbosity()
        };
        write!(f, "{}", BacktraceOmited(!self.bt_captured))?;

        let mut separated = HeaderWriter {
            inner: &mut *f,
            header: &"\n",
            started: false,
        };
        write!(&mut separated.ready(), "{}", SourceSnippets(v))?;
        #[cfg(feature = "capture-spantrace")]
        write!(
            &mut separated.ready(),
            "{}",
            SpanTraceOmited(self.span_trace)
        )?;
        Ok(())
    }
}

#[cfg(feature = "capture-spantrace")]
struct SpanTraceOmited<'a>(Option<&'a SpanTrace>);

#[cfg(feature = "capture-spantrace")]
impl fmt::Display for SpanTraceOmited<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span_trace) = self.0 {
            if span_trace.status() == SpanTraceStatus::UNSUPPORTED {
                writeln!(f, "Warning: SpanTrace capture is Unsupported.")?;
                write!(
                    f,
                    "Ensure that you've setup an error layer and the versions match"
                )?;
            }
        }

        Ok(())
    }
}

struct BacktraceOmited(bool);

impl fmt::Display for BacktraceOmited {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print some info on how to increase verbosity.
        if self.0 {
            writeln!(f, "Backtrace omitted.")?;
            write!(
                f,
                "Run with RUST_BACKTRACE=1 environment variable to display it."
            )?;
        } else {
            // This text only makes sense if frames are displayed.
            write!(
                f,
                "Run with COLORBT_SHOW_HIDDEN=1 environment variable to disable frame filtering."
            )?;
        }

        Ok(())
    }
}

struct SourceSnippets(Verbosity);

impl fmt::Display for SourceSnippets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 <= Verbosity::Medium {
            write!(
                f,
                "Run with RUST_BACKTRACE=full to include source snippets."
            )?;
        }

        Ok(())
    }
}
