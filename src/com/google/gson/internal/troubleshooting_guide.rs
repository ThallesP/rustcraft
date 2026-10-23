//! com/google/gson/internal/TroubleshootingGuide.java (Gson 2.14.0)

pub struct TroubleshootingGuide;

impl TroubleshootingGuide {
    /// A URL referring to the troubleshooting section, which Gson's messages end with.
    pub fn create_url(id: &str) -> String {
        format!("https://github.com/google/gson/blob/main/Troubleshooting.md#{id}")
    }
}
