use crate::{Help, Report, SectionExt};
#[cfg(unix)]
use std::os::unix::prelude::ExitStatusExt;

/// Add context to an error report
pub trait ContextFrom<S> {
    /// The return type of the context_from combinator
    type Return;

    /// Insert context from `source` into `self`
    ///
    /// Used to abstract over gathering relevant context from specific types into error reports.
    fn context_from(self, source: S) -> Self::Return;
}

impl<C, T, E> ContextFrom<C> for Result<T, E>
where
    E: Into<Report>,
    Report: ContextFrom<C, Return = Report>,
{
    type Return = Result<T, Report>;

    fn context_from(self, source: C) -> Self::Return {
        self.map_err(|e| e.into())
            .map_err(|report| report.context_from(source))
    }
}

impl ContextFrom<&std::process::Command> for Report {
    type Return = Report;

    fn context_from(self, source: &std::process::Command) -> Self::Return {
        let original_cmd = format!("{:?}", source);
        self.section(original_cmd.header("Command:"))
    }
}

impl ContextFrom<&std::process::Output> for Report {
    type Return = Report;

    fn context_from(self, source: &std::process::Output) -> Self::Return {
        let stdout = String::from_utf8_lossy(&source.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&source.stderr).into_owned();
        self.context_from(&source.status)
            .section(stdout.header("Stdout:"))
            .section(stderr.header("Stderr:"))
    }
}

impl ContextFrom<&std::process::ExitStatus> for Report {
    type Return = Report;

    fn context_from(self, source: &std::process::ExitStatus) -> Self::Return {
        let how = if source.success() {
            "successfully"
        } else {
            "unsuccessfully"
        };

        if let Some(code) = source.code() {
            let msg = format!("command exited {} with status code {}", how, code);
            return self.section(msg.header("Exit Status:"));
        }

        #[cfg(unix)]
        if let Some(signal) = source.signal() {
            let msg = format!("command terminated {} by signal {}", how, signal);
            self.section(msg.header("Exit Status:"))
        } else {
            unreachable!("on unix all processes either terminate via signal or with an exit code");
        }

        #[cfg(not(unix))]
        {
            let msg = format!("command exited {} without a status code or signal", how);
            self.section(msg.header("Exit Status:"))
        }
    }
}
