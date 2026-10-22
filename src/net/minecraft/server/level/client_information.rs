//! net/minecraft/server/level/ClientInformation.java
//!
//! Not ported: `hashCode`, a record's, which hashes its enum constants' identity hashes, and so
//! changes from run to run. Nothing hashes it. Nor `createDefault`, which only the server calls
//! (`CommonListenerCookie`), and which reads `Player.DEFAULT_MAIN_HAND`, whose class only loads in
//! a bootstrapped game, so the golden harness can't call it.

use std::fmt;

use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::server::level::particle_status::ParticleStatus;
use crate::net::minecraft::world::entity::humanoid_arm::HumanoidArm;
use crate::net::minecraft::world::entity::player::chat_visiblity::ChatVisiblity;

/// The options a client tells the server about (in configuration, and again in play when they
/// change): its language, render distance, chat settings, which parts of the skin show, its main
/// hand, and how many particles it shows.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientInformation {
    language: String,
    view_distance: i32,
    chat_visibility: ChatVisiblity,
    chat_colors: bool,
    model_customisation: i32,
    main_hand: HumanoidArm,
    text_filtering_enabled: bool,
    allows_listing: bool,
    particle_status: ParticleStatus,
}

impl ClientInformation {
    pub const MAX_LANGUAGE_LENGTH: i32 = 16;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        language: String,
        view_distance: i32,
        chat_visibility: ChatVisiblity,
        chat_colors: bool,
        model_customisation: i32,
        main_hand: HumanoidArm,
        text_filtering_enabled: bool,
        allows_listing: bool,
        particle_status: ParticleStatus,
    ) -> ClientInformation {
        ClientInformation {
            language,
            view_distance,
            chat_visibility,
            chat_colors,
            model_customisation,
            main_hand,
            text_filtering_enabled,
            allows_listing,
            particle_status,
        }
    }

    /// `ClientInformation(FriendlyByteBuf)`: a language of at most 16 characters, and the view
    /// distance as a signed byte. Panics on a longer language, where Java throws
    /// `DecoderException`.
    pub fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientInformation {
        let language = input.read_utf_max(16);
        let view_distance = i32::from(input.read_byte());
        let chat_visibility = ChatVisiblity::STREAM_CODEC.decode(input);
        let chat_colors = input.read_boolean();
        let model_customisation = i32::from(input.read_unsigned_byte());
        let main_hand = HumanoidArm::STREAM_CODEC.decode(input);
        let text_filtering_enabled = input.read_boolean();
        let allows_listing = input.read_boolean();
        let particle_status = ParticleStatus::STREAM_CODEC.decode(input);
        ClientInformation::new(
            language,
            view_distance,
            chat_visibility,
            chat_colors,
            model_customisation,
            main_hand,
            text_filtering_enabled,
            allows_listing,
            particle_status,
        )
    }

    /// Writes the language with `writeUtf`'s limit of 32767, not the reader's 16, and the view
    /// distance and model customisation as their low bytes.
    pub fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_utf(&self.language);
        output.write_byte(self.view_distance);
        ChatVisiblity::STREAM_CODEC.encode(output, &self.chat_visibility);
        output.write_boolean(self.chat_colors);
        output.write_byte(self.model_customisation);
        HumanoidArm::STREAM_CODEC.encode(output, &self.main_hand);
        output.write_boolean(self.text_filtering_enabled);
        output.write_boolean(self.allows_listing);
        ParticleStatus::STREAM_CODEC.encode(output, &self.particle_status);
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn view_distance(&self) -> i32 {
        self.view_distance
    }

    pub fn chat_visibility(&self) -> ChatVisiblity {
        self.chat_visibility
    }

    pub fn chat_colors(&self) -> bool {
        self.chat_colors
    }

    pub fn model_customisation(&self) -> i32 {
        self.model_customisation
    }

    pub fn main_hand(&self) -> HumanoidArm {
        self.main_hand
    }

    pub fn text_filtering_enabled(&self) -> bool {
        self.text_filtering_enabled
    }

    pub fn allows_listing(&self) -> bool {
        self.allows_listing
    }

    pub fn particle_status(&self) -> ParticleStatus {
        self.particle_status
    }
}

/// A record's: `ClientInformation[language=en_us, viewDistance=2, …, particleStatus=ALL]`.
impl fmt::Display for ClientInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientInformation[language={}, viewDistance={}, chatVisibility={}, chatColors={}, \
             modelCustomisation={}, mainHand={}, textFilteringEnabled={}, allowsListing={}, \
             particleStatus={}]",
            self.language,
            self.view_distance,
            self.chat_visibility,
            self.chat_colors,
            self.model_customisation,
            self.main_hand,
            self.text_filtering_enabled,
            self.allows_listing,
            self.particle_status
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientInformation;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
    use crate::net::minecraft::server::level::particle_status::ParticleStatus;
    use crate::net::minecraft::world::entity::humanoid_arm::HumanoidArm;
    use crate::net::minecraft::world::entity::player::chat_visiblity::ChatVisiblity;

    // A record: its components.
    encode_fields!(ClientInformation {
        language: String,
        view_distance: i32,
        chat_visibility: ChatVisiblity,
        chat_colors: bool,
        model_customisation: i32,
        main_hand: HumanoidArm,
        text_filtering_enabled: bool,
        allows_listing: bool,
        particle_status: ParticleStatus,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.server.level.ClientInformation");
        golden.pending(
            "hashCode()I",
            "hashes an enum constant's identity hash, which changes from run to run",
        );
        golden.field(
            "MAX_LANGUAGE_LENGTH",
            ClientInformation::MAX_LANGUAGE_LENGTH,
        );
        golden.check(
            "<init>(Ljava/lang/String;ILnet/minecraft/world/entity/player/ChatVisiblity;ZILnet/minecraft/world/entity/HumanoidArm;ZZLnet/minecraft/server/level/ParticleStatus;)V",
            ClientInformation::new,
        );
        golden.check_mut(
            "<init>(Lnet/minecraft/network/FriendlyByteBuf;)V",
            |input: &mut FriendlyByteBuf| ClientInformation::from_friendly_byte_buf(input),
        );
        golden.check_mut(
            "write(Lnet/minecraft/network/FriendlyByteBuf;)V",
            |this: &mut ClientInformation, output: &mut FriendlyByteBuf| this.write(output),
        );
        golden.pending(
            "createDefault()Lnet/minecraft/server/level/ClientInformation;",
            "only the server calls it, and vanilla's needs a bootstrapped game",
        );
        golden.check_method(
            "language()Ljava/lang/String;",
            |this: &mut ClientInformation| this.language().to_owned(),
        );
        golden.check_method("viewDistance()I", |this: &mut ClientInformation| {
            this.view_distance()
        });
        golden.check_method(
            "chatVisibility()Lnet/minecraft/world/entity/player/ChatVisiblity;",
            |this: &mut ClientInformation| this.chat_visibility(),
        );
        golden.check_method("chatColors()Z", |this: &mut ClientInformation| {
            this.chat_colors()
        });
        golden.check_method("modelCustomisation()I", |this: &mut ClientInformation| {
            this.model_customisation()
        });
        golden.check_method(
            "mainHand()Lnet/minecraft/world/entity/HumanoidArm;",
            |this: &mut ClientInformation| this.main_hand(),
        );
        golden.check_method("textFilteringEnabled()Z", |this: &mut ClientInformation| {
            this.text_filtering_enabled()
        });
        golden.check_method("allowsListing()Z", |this: &mut ClientInformation| {
            this.allows_listing()
        });
        golden.check_method(
            "particleStatus()Lnet/minecraft/server/level/ParticleStatus;",
            |this: &mut ClientInformation| this.particle_status(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ClientInformation, other: ClientInformation| *this == other,
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut ClientInformation| this.to_string(),
        );
        golden.finish();
    }
}
