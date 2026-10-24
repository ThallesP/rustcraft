//! net/minecraft/client/resources/server/DownloadedPackSource.java
//!
//! Not ported yet: downloading packs, their cache and applying them (a resource reload), local
//! packs, and popping packs; and `ServerPackManager`'s bookkeeping of the packs, of which what's
//! here is its prompt status (`ServerPackManager.PackPromptStatus`) and what it answers a pack
//! pushed while the server's packs are declined. A pack the player allowed can't be downloaded
//! yet, so it's answered as a download that failed, after accepting it, like Java's when its
//! download fails. Java's answers go to the connection it was set up for (`PackLoadFeedback`);
//! here `push_pack` gives them to the listener, which sends them in its protocol (`Connection.send`
//! names the protocol's listener).

use crate::java::util::uuid::UUID;
use crate::net::minecraft::client::multiplayer::server_data::ServerPackStatus;
use crate::net::minecraft::network::protocol::common::serverbound_resource_pack_packet::Action;

/// `ServerPackManager.PackPromptStatus`: whether the player let the server's packs in.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackPromptStatus {
    PENDING,
    ALLOWED,
    DECLINED,
}

/// The server's resource packs, as the client takes them: `Minecraft.downloadedPackSource`.
pub struct DownloadedPackSource {
    pack_prompt_status: PackPromptStatus,
}

impl Default for DownloadedPackSource {
    fn default() -> DownloadedPackSource {
        DownloadedPackSource::new()
    }
}

impl DownloadedPackSource {
    pub fn new() -> DownloadedPackSource {
        DownloadedPackSource {
            pack_prompt_status: PackPromptStatus::PENDING,
        }
    }

    /// `ConnectScreen`'s conversion of the server's setting (`convertPackStatus`).
    pub fn convert_pack_status(status: ServerPackStatus) -> PackPromptStatus {
        match status {
            ServerPackStatus::ENABLED => PackPromptStatus::ALLOWED,
            ServerPackStatus::DISABLED => PackPromptStatus::DECLINED,
            ServerPackStatus::PROMPT => PackPromptStatus::PENDING,
        }
    }

    pub fn configure_for_server_control(&mut self, pack_prompt_status: PackPromptStatus) {
        self.pack_prompt_status = PackPromptStatus::PENDING;
        match pack_prompt_status {
            PackPromptStatus::PENDING => {}
            PackPromptStatus::ALLOWED => self.allow_server_packs(),
            PackPromptStatus::DECLINED => self.reject_server_packs(),
        }
    }

    pub fn allow_server_packs(&mut self) {
        self.pack_prompt_status = PackPromptStatus::ALLOWED;
    }

    pub fn reject_server_packs(&mut self) {
        self.pack_prompt_status = PackPromptStatus::DECLINED;
    }

    /// `pushPack`, with what the client answers: a declined pack at once; one still waiting
    /// on the player stays queued in Java until they answer.
    pub fn push_pack(&mut self, id: UUID) -> Vec<Action> {
        match self.pack_prompt_status {
            PackPromptStatus::DECLINED => vec![Action::DECLINED],
            PackPromptStatus::ALLOWED => {
                eprintln!("Downloading server resource packs isn't ported: {id} fails");
                vec![Action::ACCEPTED, Action::FAILED_DOWNLOAD]
            }
            PackPromptStatus::PENDING => Vec::new(),
        }
    }
}
