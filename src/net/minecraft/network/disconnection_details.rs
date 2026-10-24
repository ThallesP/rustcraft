//! net/minecraft/network/DisconnectionDetails.java
//!
//! Not ported yet: `CODEC`, which only the disconnection reports and transfers write and read.

use std::path::PathBuf;

use crate::java::net::uri::URI;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::contents::translatable_contents::TranslationArgument;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;

/// Why a connection ended, with the report written about it and the server's bug report link.
#[derive(Clone, Debug)]
pub struct DisconnectionDetails {
    reason: Component,
    report: Option<PathBuf>,
    // Boxed: a `URI` is large, and nothing but the disconnected screen reads it.
    bug_report_link: Option<Box<URI>>,
}

impl DisconnectionDetails {
    pub fn new(reason: impl Into<Component>) -> DisconnectionDetails {
        DisconnectionDetails::with_report(reason, None, None)
    }

    pub fn with_report(
        reason: impl Into<Component>,
        report: Option<PathBuf>,
        bug_report_link: Option<URI>,
    ) -> DisconnectionDetails {
        DisconnectionDetails {
            reason: reason.into(),
            report,
            bug_report_link: bug_report_link.map(Box::new),
        }
    }

    /// `new DisconnectionDetails(Component.translatable(key))`: the reasons the client gives
    /// itself are translation keys.
    pub fn translatable(key: &str) -> DisconnectionDetails {
        DisconnectionDetails::new(Component::translatable(key))
    }

    /// `Component.translatable("disconnect.genericReason", reason)`, which shows
    /// `Internal Exception: …` and the like.
    pub fn generic_reason(reason: &str) -> DisconnectionDetails {
        DisconnectionDetails::new(Component::translatable_with_args(
            "disconnect.genericReason",
            vec![TranslationArgument::String(reason.to_string())],
        ))
    }

    pub fn reason(&self) -> &Component {
        &self.reason
    }

    /// The reason's text, as the disconnected screen shows it (`getString`, through `en_us`).
    pub fn reason_text(&self) -> String {
        self.reason.get_string()
    }

    pub fn report(&self) -> Option<&PathBuf> {
        self.report.as_ref()
    }

    pub fn bug_report_link(&self) -> Option<&URI> {
        self.bug_report_link.as_deref()
    }
}
