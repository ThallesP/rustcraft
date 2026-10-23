//! net/minecraft/network/chat/ChatType.java
//!
//! A kind of chat message, an element of the synchronized `chat_type` registry, as `DIRECT_CODEC`
//! reads it (`RegistryDataLoader`). Not ported yet: `DIRECT_STREAM_CODEC` and `STREAM_CODEC`
//! (`player_chat`'s, `disguised_chat`'s), `bind` and `Bound`, which come with chat.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::MapCodecExt;
use crate::net::minecraft::network::chat::chat_type_decoration::ChatTypeDecoration;

static DIRECT_CODEC: StaticCodec<ChatType> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            ChatTypeDecoration::CODEC
                .shared()
                .field_of("chat")
                .for_getter(|o: &ChatType| o.chat.clone()),
            ChatTypeDecoration::CODEC
                .shared()
                .field_of("narration")
                .for_getter(|o: &ChatType| o.narration.clone()),
        ),
        ChatType::new,
    )
    .boxed()
});

#[derive(Clone, Debug, PartialEq)]
pub struct ChatType {
    chat: ChatTypeDecoration,
    narration: ChatTypeDecoration,
}

impl ChatType {
    pub const DIRECT_CODEC: &'static StaticCodec<ChatType> = &DIRECT_CODEC;

    pub fn new(chat: ChatTypeDecoration, narration: ChatTypeDecoration) -> ChatType {
        ChatType { chat, narration }
    }

    pub fn chat(&self) -> &ChatTypeDecoration {
        &self.chat
    }

    pub fn narration(&self) -> &ChatTypeDecoration {
        &self.narration
    }
}
