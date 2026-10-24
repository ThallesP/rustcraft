//! net/minecraft/world/item/Item.java
//!
//! Only `CODEC`, an item by its registered name (a holder of `BuiltInRegistries.ITEM`, not air),
//! which an item stack's template reads. The items themselves (`Items`), their properties and
//! behaviour, `STREAM_CODEC` (by id, over the registries a buffer holds) and
//! `CODEC_WITH_BOUND_COMPONENTS` come with their callers.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;
use crate::net::minecraft::world::item::items;

// `BuiltInRegistries.ITEM.holderByNameCodec()`: a registered name, stable either way (a built-in
// entry's lifecycle), then checked not to be air.
static CODEC: StaticCodec<Holder> = LazyLock::new(|| {
    ExtraCodecs::override_lifecycle_stable(Identifier::CODEC.shared().comap_flat_map(
        |name: Identifier| {
            match BuiltInRegistries::ITEM.get(&ResourceKey::create(Registries::ITEM, name.clone()))
            {
                Some(holder) => DataResult::success(holder),
                None => {
                    let registry = (**Registries::ITEM).clone();
                    DataResult::error(|| format!("Unknown registry key in {registry}: {name}"))
                }
            }
        },
        |holder: &Holder| {
            holder
                .unwrap_key()
                .expect("a reference has its key")
                .identifier()
                .clone()
        },
    ))
    .flat_comap_map(
        |holder| holder,
        |holder: &Holder| DataResult::success(holder.clone()),
    )
    .validate(|item: Holder| {
        if Item::is_air(&item) {
            DataResult::error(|| "Item must not be minecraft:air".to_owned())
        } else {
            DataResult::success(item)
        }
    })
    .boxed()
});

pub struct Item;

impl Item {
    pub const CODEC: &'static StaticCodec<Holder> = &CODEC;

    /// Whether the holder is `Items.AIR`'s.
    pub fn is_air(item: &Holder) -> bool {
        item.is(&ResourceKey::create(
            Registries::ITEM,
            Identifier::with_default_namespace(items::AIR),
        ))
    }
}
