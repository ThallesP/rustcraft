//! net/minecraft/world/level/SpawnData.java
//!
//! What a spawner spawns next: the entity's tag, its id written in full. `CODEC` reads the entity
//! (`CompoundTag.CODEC`); its custom spawn rules (`InclusiveRange`s of light) and equipment
//! (`EquipmentTable`), which only the server reads, aren't ported and fail to decode saying so,
//! so a spawner's data with them doesn't load. `LIST_CODEC` (`SpawnPotentials`, which a server
//! doesn't send) isn't ported either.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::map_codec::{self, MapCodecExt};
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;

// CompoundTag.CODEC: any data, converted to NBT, that's a compound.
static COMPOUND_TAG_CODEC: StaticCodec<CompoundTag> = LazyLock::new(|| {
    ExtraCodecs::NBT
        .shared()
        .flat_xmap(
            |tag: Tag| match tag {
                Tag::Compound(compound) => DataResult::success(compound),
                other => DataResult::error(move || format!("Not a compound tag: {other}")),
            },
            |compound: &CompoundTag| DataResult::success(Tag::Compound(compound.copy())),
        )
        .boxed()
});

static CODEC: StaticCodec<SpawnData> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            COMPOUND_TAG_CODEC
                .shared()
                .field_of("entity")
                .for_getter(|o: &SpawnData| o.entity_to_spawn.clone()),
            map_codec::unported::<()>("SpawnData.CustomSpawnRules")
                .codec()
                .optional_field_of("custom_spawn_rules")
                .for_getter(|_: &SpawnData| None),
            map_codec::unported::<()>("EquipmentTable")
                .codec()
                .optional_field_of("equipment")
                .for_getter(|_: &SpawnData| None),
        ),
        |entity_to_spawn, _custom_spawn_rules, _equipment| SpawnData::new(entity_to_spawn),
    )
    .boxed()
});

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnData {
    entity_to_spawn: CompoundTag,
}

impl SpawnData {
    pub const CODEC: &'static StaticCodec<SpawnData> = &CODEC;

    /// `SpawnData(entityToSpawn, customSpawnRules, equipment)`: the tag's `id` written in full
    /// where it reads as an identifier, else taken out.
    pub fn new(mut entity_to_spawn: CompoundTag) -> SpawnData {
        let id = entity_to_spawn
            .get_string("id")
            .and_then(|id| Identifier::read(id).into_result());
        match id {
            Some(id) => entity_to_spawn.put_string("id".to_owned(), id.to_string()),
            None => {
                entity_to_spawn.remove("id");
            }
        }
        SpawnData { entity_to_spawn }
    }

    pub fn entity_to_spawn(&self) -> &CompoundTag {
        &self.entity_to_spawn
    }
}
