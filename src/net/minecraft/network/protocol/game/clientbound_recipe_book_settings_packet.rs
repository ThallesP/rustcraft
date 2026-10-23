//! net/minecraft/network/protocol/game/ClientboundRecipeBookSettingsPacket.java
//!
//! Not ported yet: the record's `equals`, `hashCode` and `toString`, which compare, hash and print
//! its settings by identity (`RecipeBookSettings` overrides none of them).

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::stats::recipe_book_settings::RecipeBookSettings;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundRecipeBookSettingsPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            RecipeBookSettings::STREAM_CODEC,
            ClientboundRecipeBookSettingsPacket::book_settings,
            ClientboundRecipeBookSettingsPacket::new,
        ))
    });

/// Whether the player's recipe books are open and filtering, which the client shows.
#[derive(Clone, Debug)]
pub struct ClientboundRecipeBookSettingsPacket {
    book_settings: RecipeBookSettings,
}

impl ClientboundRecipeBookSettingsPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundRecipeBookSettingsPacket,
    > = &STREAM_CODEC;

    pub fn new(book_settings: RecipeBookSettings) -> ClientboundRecipeBookSettingsPacket {
        ClientboundRecipeBookSettingsPacket { book_settings }
    }

    pub fn book_settings(&self) -> RecipeBookSettings {
        self.book_settings
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundRecipeBookSettingsPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_RECIPE_BOOK_SETTINGS
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_recipe_book_settings(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundRecipeBookSettingsPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::stats::recipe_book_settings::RecipeBookSettings;

    encode_fields!(ClientboundRecipeBookSettingsPacket {
        book_settings: RecipeBookSettings,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundRecipeBookSettingsPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundRecipeBookSettingsPacket");
        for name in [
            "equals(Ljava/lang/Object;)Z",
            "hashCode()I",
            "toString()Ljava/lang/String;",
        ] {
            golden.pending(name, "compares, hashes or prints the settings by identity");
        }
        golden.check_codec("STREAM_CODEC", || {
            ClientboundRecipeBookSettingsPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/stats/RecipeBookSettings;)V",
            ClientboundRecipeBookSettingsPacket::new,
        );
        golden.check_method(
            "bookSettings()Lnet/minecraft/stats/RecipeBookSettings;",
            |this: &mut This| this.book_settings(),
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut This| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
