//! net/minecraft/world/level/block/entity/SignText.java
//!
//! A side of a sign: its four lines, their filtered versions, the dye colour and the glow, as
//! `CODEC` reads them. Not ported yet: `STREAM_CODEC`, the tooltips, the render cache
//! (`getRenderMessages`) and the `with…` methods the sign editor calls.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{BOOL, CodecExt, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::map_codec::MapCodecExt;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::world::item::dye_color::DyeColor;

static CODEC: StaticCodec<SignText> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            ComponentSerialization::CODEC
                .shared()
                .list_of_sized(4, 4)
                .field_of("messages")
                .for_getter(|o: &SignText| o.messages.clone()),
            ComponentSerialization::CODEC
                .shared()
                .list_of_sized(4, 4)
                .lenient_optional_field_of("filtered_messages")
                .for_getter(SignText::filtered_messages_for_serialization),
            // ExtraCodecs.optionalAlwaysPresentFieldOf: the default when missing, always written.
            DyeColor::CODEC
                .shared()
                .optional_field_of("color")
                .xmap(
                    |color: Option<DyeColor>| color.unwrap_or(DyeColor::BLACK),
                    |color: &DyeColor| Some(*color),
                )
                .for_getter(|o: &SignText| o.color),
            BOOL.optional_field_of("has_glowing_text")
                .xmap(
                    |glowing: Option<bool>| glowing.unwrap_or(false),
                    |glowing: &bool| Some(*glowing),
                )
                .for_getter(|o: &SignText| o.has_glowing_text),
        ),
        SignText::load,
    )
    .validate(SignText::validate_line_counts)
    .boxed()
});

/// The lines `EMPTY` holds: `CommonComponents.EMPTY`, an empty literal.
fn empty_messages() -> Vec<Component> {
    vec![Component::from(Component::literal("")); 4]
}

#[derive(Clone, Debug, PartialEq)]
pub struct SignText {
    messages: Vec<Component>,
    filtered_messages: Vec<Component>,
    color: DyeColor,
    has_glowing_text: bool,
}

impl SignText {
    pub const LINES: i32 = 4;
    pub const CODEC: &'static StaticCodec<SignText> = &CODEC;

    pub fn new(
        messages: Vec<Component>,
        filtered_messages: Vec<Component>,
        color: DyeColor,
        has_glowing_text: bool,
    ) -> SignText {
        SignText {
            messages,
            filtered_messages,
            color,
            has_glowing_text,
        }
    }

    /// `EMPTY`.
    pub fn empty() -> SignText {
        SignText::new(empty_messages(), empty_messages(), DyeColor::BLACK, false)
    }

    fn load(
        messages: Vec<Component>,
        filtered_messages: Option<Vec<Component>>,
        color: DyeColor,
        has_glowing_text: bool,
    ) -> SignText {
        let filtered_messages = filtered_messages.unwrap_or_else(|| messages.clone());
        SignText::new(messages, filtered_messages, color, has_glowing_text)
    }

    fn validate_line_counts(sign_text: SignText) -> DataResult<SignText> {
        if sign_text.filtered_messages.len() != sign_text.messages.len() {
            DataResult::error(|| "Filtered and raw line counts are not equal".to_owned())
        } else {
            DataResult::success(sign_text)
        }
    }

    /// The filtered lines, where they differ from the lines.
    fn filtered_messages_for_serialization(&self) -> Option<Vec<Component>> {
        (self.filtered_messages != self.messages).then(|| self.filtered_messages.clone())
    }

    pub fn has_glowing_text(&self) -> bool {
        self.has_glowing_text
    }

    pub fn get_color(&self) -> DyeColor {
        self.color
    }

    /// `getMessages(shouldFilter)`.
    pub fn get_messages(&self, should_filter: bool) -> &[Component] {
        if should_filter {
            &self.filtered_messages
        } else {
            &self.messages
        }
    }

    pub fn get_message(&self, index: usize, is_filtered: bool) -> &Component {
        &self.get_messages(is_filtered)[index]
    }
}
