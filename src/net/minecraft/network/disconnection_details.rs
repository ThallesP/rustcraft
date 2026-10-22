//! net/minecraft/network/DisconnectionDetails.java
//!
//! Not ported yet: `CODEC`, which waits for DataFixerUpper, and the reason as a `Component`,
//! which waits for text components: the reason is the text a component would show, or its
//! translation key (`disconnect.endOfStream`), until they come.

use std::path::PathBuf;

/// Why a connection ended, with the report written about it and the server's bug report link.
#[derive(Clone, Debug, PartialEq)]
pub struct DisconnectionDetails {
    reason: String,
    report: Option<PathBuf>,
    bug_report_link: Option<String>,
}

impl DisconnectionDetails {
    pub fn new(reason: impl Into<String>) -> DisconnectionDetails {
        DisconnectionDetails::with_report(reason, None, None)
    }

    pub fn with_report(
        reason: impl Into<String>,
        report: Option<PathBuf>,
        bug_report_link: Option<String>,
    ) -> DisconnectionDetails {
        DisconnectionDetails {
            reason: reason.into(),
            report,
            bug_report_link,
        }
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn report(&self) -> Option<&PathBuf> {
        self.report.as_ref()
    }

    pub fn bug_report_link(&self) -> Option<&str> {
        self.bug_report_link.as_deref()
    }
}
