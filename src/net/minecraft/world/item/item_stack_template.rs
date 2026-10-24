//! net/minecraft/world/item/ItemStackTemplate.java
//!
//! An item stack as a hover event shows one: an item (not air), a count from 1 to 99 and its
//! component patch, whose values are kept as data (`DataComponentPatch`). Not ported yet:
//! `STREAM_CODEC`, `create` (an `ItemStack`, which `ItemStack.validateStrict` checks) and the
//! conversions from stacks, which come with `ItemStack`.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::core::component::data_component_patch::DataComponentPatch;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;
use crate::net::minecraft::world::item::item::Item;

static MAP_CODEC: StaticMapCodec<ItemStackTemplate> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            Item::CODEC
                .shared()
                .field_of("id")
                .for_getter(|template: &ItemStackTemplate| template.item.clone()),
            ExtraCodecs::int_range(1, 99)
                .optional_field_of_default("count", 1)
                .for_getter(|template: &ItemStackTemplate| template.count),
            DataComponentPatch::CODEC
                .shared()
                .optional_field_of_default("components", DataComponentPatch::EMPTY)
                .for_getter(|template: &ItemStackTemplate| template.components.clone()),
        ),
        ItemStackTemplate::new,
    )
    .boxed()
});

#[derive(Clone, Debug, PartialEq)]
pub struct ItemStackTemplate {
    item: Holder,
    count: i32,
    components: DataComponentPatch,
}

impl ItemStackTemplate {
    pub const MAP_CODEC: &'static StaticMapCodec<ItemStackTemplate> = &MAP_CODEC;

    /// Panics for no items or air, where Java throws `IllegalStateException`; its codec reads
    /// neither.
    pub fn new(item: Holder, count: i32, components: DataComponentPatch) -> ItemStackTemplate {
        if count == 0 || Item::is_air(&item) {
            panic!("IllegalStateException: Item must be non-empty");
        }
        ItemStackTemplate {
            item,
            count,
            components,
        }
    }

    pub fn item(&self) -> &Holder {
        &self.item
    }

    pub fn count(&self) -> i32 {
        self.count
    }

    pub fn components(&self) -> &DataComponentPatch {
        &self.components
    }
}

/// The record's `toString`, the item as its holder (`Reference{key=minecraft:stone}`).
impl fmt::Display for ItemStackTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = self.item.unwrap_key().expect("a reference has its key");
        write!(
            f,
            "ItemStackTemplate[item=Reference{{{key}={}}}, count={}, components={}]",
            key.identifier(),
            self.count,
            self.components
        )
    }
}
