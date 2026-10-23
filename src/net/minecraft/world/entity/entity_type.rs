//! net/minecraft/world/entity/EntityType.java
//!
//! Java's types are objects the game compares by reference, numbered by
//! `BuiltInRegistries.ENTITY_TYPE`, so here a type is its id there, `Copy`, like a `Block`, and
//! derefs to what Java's object holds (`EntityTypeData`), which `EntityTypes` registers on first
//! use. Not ported yet: the factory (`EntityFactory`, and `create`, `spawn` and the entity loading
//! built on it), which comes with the entity classes; the attachments its dimensions hold
//! (`EntityAttachments`, which `EntityDimensions` leaves out, and so the builder's
//! `passengerAttachments`, `vehicleAttachment`, `ridingOffset`, `nameTagOffset` and `attach`);
//! `requiredFeatures` (`FeatureFlagSet`: no vanilla type asks for more than the default
//! `FeatureFlags.VANILLA_SET`) and `canSpawn`, which reads it with the level's difficulty;
//! `getDescription` (text components); `CODEC` (DataFixerUpper's codecs); `isBlockDangerous`
//! (`NodeEvaluator`); `getSpawnAABB`, `tryCast` and `getBaseClass`, which wait for their callers;
//! and the data fixer lookup in `Builder.build`, which only logs.

use std::fmt;
use std::ops::Deref;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::tags::block_tags::{BlockTags, TagKey};
use crate::net::minecraft::util::util::Util;
use crate::net::minecraft::world::entity::entity_dimensions::EntityDimensions;
use crate::net::minecraft::world::entity::entity_types::{EntityTypes, REGISTRY};
use crate::net::minecraft::world::entity::mob_category::MobCategory;

// `ByteBufCodecs.registry(Registries.ENTITY_TYPE)`: a type by its id in the built-in registry, in
// a VarInt, read without the registry access, like `BlockEntityType`'s. The registry is a
// defaulted one, so an id it doesn't have reads as a pig.
static STREAM_CODEC: StaticStreamCodec<ByteBuf, EntityType> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::id_mapper(
        |id| BuiltInRegistries::ENTITY_TYPE.by_id_or_throw(id),
        |r#type: &EntityType| BuiltInRegistries::ENTITY_TYPE.get_id(*r#type),
    ))
});

/// A kind of entity, by its id in `BuiltInRegistries.ENTITY_TYPE`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityType(pub(crate) u16);

impl EntityType {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, EntityType> = &STREAM_CODEC;
    pub const NO_UPDATE_INTERVAL: i32 = i32::MAX;

    pub fn id(&self) -> i32 {
        self.0 as i32
    }

    /// `getKey(type)`: its name in the registry, `minecraft:pig`.
    pub fn get_key(r#type: EntityType) -> &'static Identifier {
        BuiltInRegistries::ENTITY_TYPE.get_key(r#type)
    }

    /// Whether only operators may set its data from an item (`EntityTypes.OP_ONLY_CUSTOM_DATA`).
    pub fn only_op_can_set_nbt(&self) -> bool {
        EntityTypes::OP_ONLY_CUSTOM_DATA.contains(self)
    }
}

impl Deref for EntityType {
    type Target = EntityTypeData;

    fn deref(&self) -> &'static EntityTypeData {
        REGISTRY.data(*self)
    }
}

/// `toString`: its description id.
impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_description_id())
    }
}

/// Its name in the registry.
impl fmt::Debug for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", EntityType::get_key(*self))
    }
}

/// What Java's `EntityType` object holds that's ported. `Builder.build` is the only caller of
/// Java's constructor, so it makes this itself.
pub struct EntityTypeData {
    category: MobCategory,
    immune_to: TagKey,
    serialize: bool,
    summon: bool,
    fire_immune: bool,
    can_spawn_far_from_player: bool,
    client_tracking_range: i32,
    update_interval: i32,
    description_id: String,
    loot_table: Option<ResourceKey>,
    dimensions: EntityDimensions,
    spawn_dimensions_scale: f32,
    allowed_in_peaceful: bool,
    track_deltas: bool,
}

impl EntityTypeData {
    pub fn can_serialize(&self) -> bool {
        self.serialize
    }

    pub fn can_summon(&self) -> bool {
        self.summon
    }

    pub fn fire_immune(&self) -> bool {
        self.fire_immune
    }

    pub fn can_spawn_far_from_player(&self) -> bool {
        self.can_spawn_far_from_player
    }

    pub fn get_category(&self) -> MobCategory {
        self.category
    }

    pub fn get_description_id(&self) -> &str {
        &self.description_id
    }

    /// The description id after its last dot, `pig`.
    pub fn to_short_string(&self) -> &str {
        match self.description_id.rfind('.') {
            None => &self.description_id,
            Some(dot) => &self.description_id[dot + 1..],
        }
    }

    pub fn get_default_loot_table(&self) -> Option<&ResourceKey> {
        self.loot_table.as_ref()
    }

    pub fn get_width(&self) -> f32 {
        self.dimensions.width()
    }

    pub fn get_height(&self) -> f32 {
        self.dimensions.height()
    }

    pub fn get_dimensions(&self) -> EntityDimensions {
        self.dimensions
    }

    /// Java's private `spawnDimensionsScale`, which `getSpawnAABB` scales the dimensions by.
    pub fn spawn_dimensions_scale(&self) -> f32 {
        self.spawn_dimensions_scale
    }

    /// Java's private `immuneTo`: the blocks `isBlockDangerous` calls safe for the type.
    pub fn immune_to(&self) -> TagKey {
        self.immune_to
    }

    pub fn client_tracking_range(&self) -> i32 {
        self.client_tracking_range
    }

    pub fn update_interval(&self) -> i32 {
        self.update_interval
    }

    pub fn has_update_interval(&self) -> bool {
        self.update_interval != i32::MAX
    }

    pub fn track_deltas(&self) -> bool {
        self.track_deltas
    }

    pub fn is_allowed_in_peaceful(&self) -> bool {
        self.allowed_in_peaceful
    }
}

/// `DependantName<EntityType<?>, T>`: worked out from the type's id, or fixed.
enum DependantName<T> {
    FromId,
    Fixed(T),
}

/// `EntityType.Builder`: what a type is made with. Its setters return it, to chain like Java's.
pub struct Builder {
    category: MobCategory,
    immune_to: TagKey,
    serialize: bool,
    summon: bool,
    fire_immune: bool,
    can_spawn_far_from_player: bool,
    client_tracking_range: i32,
    update_interval: i32,
    dimensions: EntityDimensions,
    spawn_dimensions_scale: f32,
    loot_table: DependantName<Option<ResourceKey>>,
    allowed_in_peaceful: bool,
    track_deltas: bool,
}

impl Builder {
    fn new(category: MobCategory) -> Builder {
        Builder {
            category,
            immune_to: BlockTags::DEFAULT_IMMUNE_TO,
            serialize: true,
            summon: true,
            fire_immune: false,
            can_spawn_far_from_player: category == MobCategory::CREATURE
                || category == MobCategory::MISC,
            client_tracking_range: 5,
            update_interval: 3,
            dimensions: EntityDimensions::scalable(0.6, 1.8),
            spawn_dimensions_scale: 1.0,
            loot_table: DependantName::FromId,
            allowed_in_peaceful: true,
            track_deltas: true,
        }
    }

    /// `of(factory, category)`, without the factory.
    pub fn of(category: MobCategory) -> Builder {
        Builder::new(category)
    }

    /// `createNothing(category)`: a type whose factory makes nothing.
    pub fn create_nothing(category: MobCategory) -> Builder {
        Builder::new(category)
    }

    pub fn sized(mut self, width: f32, height: f32) -> Builder {
        self.dimensions = EntityDimensions::scalable(width, height);
        self
    }

    pub fn spawn_dimensions_scale(mut self, scale: f32) -> Builder {
        self.spawn_dimensions_scale = scale;
        self
    }

    pub fn eye_height(mut self, eye_height: f32) -> Builder {
        self.dimensions = self.dimensions.with_eye_height(eye_height);
        self
    }

    pub fn no_summon(mut self) -> Builder {
        self.summon = false;
        self
    }

    pub fn no_save(mut self) -> Builder {
        self.serialize = false;
        self
    }

    pub fn fire_immune(mut self) -> Builder {
        self.fire_immune = true;
        self
    }

    pub fn immune_to(mut self, tag: TagKey) -> Builder {
        self.immune_to = tag;
        self
    }

    pub fn can_spawn_far_from_player(mut self) -> Builder {
        self.can_spawn_far_from_player = true;
        self
    }

    pub fn client_tracking_range(mut self, client_chunk_range: i32) -> Builder {
        self.client_tracking_range = client_chunk_range;
        self
    }

    pub fn update_interval(mut self, update_interval: i32) -> Builder {
        self.update_interval = update_interval;
        self
    }

    pub fn no_update_interval(self) -> Builder {
        self.update_interval(i32::MAX)
    }

    pub fn no_loot_table(mut self) -> Builder {
        self.loot_table = DependantName::Fixed(None);
        self
    }

    pub fn not_in_peaceful(mut self) -> Builder {
        self.allowed_in_peaceful = false;
        self
    }

    pub fn dont_track_deltas(mut self) -> Builder {
        self.track_deltas = false;
        self
    }

    /// The type registered under `name`: its description id and loot table come from the name.
    pub fn build(self, name: &ResourceKey) -> EntityTypeData {
        let loot_table = match self.loot_table {
            DependantName::Fixed(table) => table,
            DependantName::FromId => Some(ResourceKey::create(
                Registries::LOOT_TABLE,
                name.identifier().with_prefix("entities/"),
            )),
        };
        EntityTypeData {
            category: self.category,
            immune_to: self.immune_to,
            serialize: self.serialize,
            summon: self.summon,
            fire_immune: self.fire_immune,
            can_spawn_far_from_player: self.can_spawn_far_from_player,
            client_tracking_range: self.client_tracking_range,
            update_interval: self.update_interval,
            description_id: Util::make_description_id("entity", Some(name.identifier())),
            loot_table,
            dimensions: self.dimensions,
            spawn_dimensions_scale: self.spawn_dimensions_scale,
            allowed_in_peaceful: self.allowed_in_peaceful,
            track_deltas: self.track_deltas,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::Value;

    use super::EntityType;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
    use crate::net::minecraft::resources::identifier::Identifier;

    /// A type is its registered name, `minecraft:pig`.
    impl Decode for EntityType {
        fn decode(value: &Value) -> Self {
            let key = Identifier::parse(&String::decode(value));
            BuiltInRegistries::ENTITY_TYPE
                .get_optional(&key)
                .unwrap_or_else(|| panic!("no entity type {key}"))
        }
    }

    impl Encode for EntityType {
        fn encode(&self) -> Value {
            Value::from(EntityType::get_key(*self).to_string())
        }
    }

    /// What `EntityTypes`' golden suite writes for a type: `[id, key, category, [canSerialize,
    /// canSummon, fireImmune, canSpawnFarFromPlayer, isAllowedInPeaceful, trackDeltas,
    /// hasUpdateInterval, onlyOpCanSetNbt], immuneTo, [width, height, eyeHeight, fixed],
    /// spawnDimensionsScale, clientTrackingRange, updateInterval, descriptionId, toShortString,
    /// lootTable]`.
    pub(crate) fn values(r#type: EntityType) -> impl Encode {
        let dimensions = r#type.get_dimensions();
        (
            r#type.id(),
            EntityType::get_key(r#type).to_string(),
            r#type.get_category(),
            (
                r#type.can_serialize(),
                r#type.can_summon(),
                r#type.fire_immune(),
                r#type.can_spawn_far_from_player(),
                r#type.is_allowed_in_peaceful(),
                r#type.track_deltas(),
                r#type.has_update_interval(),
                r#type.only_op_can_set_nbt(),
            ),
            format!("minecraft:{}", r#type.immune_to().0),
            (
                dimensions.width(),
                dimensions.height(),
                dimensions.eye_height(),
                dimensions.fixed(),
            ),
            r#type.spawn_dimensions_scale(),
            r#type.client_tracking_range(),
            r#type.update_interval(),
            r#type.get_description_id().to_owned(),
            r#type.to_short_string().to_owned(),
            r#type
                .get_default_loot_table()
                .map(|table| table.identifier().to_string()),
        )
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.EntityType");
        golden.field("NO_UPDATE_INTERVAL", EntityType::NO_UPDATE_INTERVAL);
        golden.check_codec("STREAM_CODEC", || EntityType::STREAM_CODEC);
        golden.finish();
    }
}
