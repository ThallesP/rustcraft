//! net/minecraft/world/entity/decoration/painting/PaintingVariant.java
//!
//! A painting's size, art and texts, an element of the synchronized `painting_variant` registry,
//! as `DIRECT_CODEC` reads it (`RegistryDataLoader`). `CODEC`, `DIRECT_STREAM_CODEC` and
//! `STREAM_CODEC` come with paintings.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::MapCodecExt;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;

static DIRECT_CODEC: StaticCodec<PaintingVariant> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            ExtraCodecs::int_range(1, 16)
                .field_of("width")
                .for_getter(|o: &PaintingVariant| o.width),
            ExtraCodecs::int_range(1, 16)
                .field_of("height")
                .for_getter(|o: &PaintingVariant| o.height),
            Identifier::CODEC
                .shared()
                .field_of("asset_id")
                .for_getter(|o: &PaintingVariant| o.asset_id.clone()),
            ComponentSerialization::CODEC
                .shared()
                .optional_field_of("title")
                .for_getter(|o: &PaintingVariant| o.title.clone()),
            ComponentSerialization::CODEC
                .shared()
                .optional_field_of("author")
                .for_getter(|o: &PaintingVariant| o.author.clone()),
        ),
        PaintingVariant::new,
    )
    .boxed()
});

#[derive(Clone, Debug, PartialEq)]
pub struct PaintingVariant {
    width: i32,
    height: i32,
    asset_id: Identifier,
    title: Option<Component>,
    author: Option<Component>,
}

impl PaintingVariant {
    pub const DIRECT_CODEC: &'static StaticCodec<PaintingVariant> = &DIRECT_CODEC;

    pub fn new(
        width: i32,
        height: i32,
        asset_id: Identifier,
        title: Option<Component>,
        author: Option<Component>,
    ) -> PaintingVariant {
        PaintingVariant {
            width,
            height,
            asset_id,
            title,
            author,
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn asset_id(&self) -> &Identifier {
        &self.asset_id
    }

    pub fn title(&self) -> Option<&Component> {
        self.title.as_ref()
    }

    pub fn author(&self) -> Option<&Component> {
        self.author.as_ref()
    }
}
