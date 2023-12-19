//! Module for new types that isolate complext formatting
use std::fmt;

pub(crate) struct LocationSection<'a>(
    pub(crate) Option<&'a std::panic::Location<'a>>,
);

impl fmt::Display for LocationSection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // If known, print panic location.
        if let Some(loc) = self.0 {
            write!(f, "{}", loc.file())?;
            write!(f, ":")?;
            write!(f, "{}", loc.line())?;
        } else {
            write!(f, "<unknown>")?;
        }

        Ok(())
    }
}
