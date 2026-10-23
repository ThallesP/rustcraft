//! net/minecraft/world/entity/EntityTypes.java
//!
//! Every entity type, registered in Java's order, which numbers them in
//! `BuiltInRegistries.ENTITY_TYPE`. Each registration is the Java one with what's ported: not the
//! factory (`Allay::new`, `boatFactory(() -> Items.ACACIA_BOAT)`), which comes with the entity
//! classes, nor the attachments (`passengerAttachments`, `vehicleAttachment`, `ridingOffset`,
//! `nameTagOffset`, `attach`), which come with `EntityAttachments`.
//!
//! Java's fields are `EntityType` handles here, consts numbered by the order the fields register
//! in. `entity_types!` makes the consts and the function registering them, which checks each
//! registration lands where its field says. The registry is a static made on first use, like
//! Java's static initializer, and is what `EntityType` reads.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::world::entity::entity_type::{Builder, EntityType, EntityTypeData};
use crate::net::minecraft::world::entity::entity_type_ids::EntityTypeIds;
use crate::net::minecraft::world::entity::mob_category::MobCategory;

/// `BuiltInRegistries.ENTITY_TYPE`, a `DefaultedRegistry` whose default is the pig: every type in
/// the order `EntityTypes` registers them.
pub struct EntityTypeRegistry {
    types: Vec<EntityTypeData>,
    keys: Vec<ResourceKey>,
    by_key: HashMap<Identifier, EntityType>,
    default_key: Identifier,
    default_value: EntityType,
}

/// The registry, which `EntityTypes` fills on first use, like Java's static initializers.
pub(crate) static REGISTRY: LazyLock<EntityTypeRegistry> = LazyLock::new(EntityTypes::bootstrap);

impl EntityTypeRegistry {
    /// What the type holds.
    pub(crate) fn data(&self, r#type: EntityType) -> &EntityTypeData {
        &self.types[r#type.0 as usize]
    }

    /// `getKey`: the type's registered name.
    pub fn get_key(&self, r#type: EntityType) -> &Identifier {
        self.keys[r#type.0 as usize].identifier()
    }

    /// `getResourceKey`, which every registered type has.
    pub fn get_resource_key(&self, r#type: EntityType) -> &ResourceKey {
        &self.keys[r#type.0 as usize]
    }

    /// `getId`.
    pub fn get_id(&self, r#type: EntityType) -> i32 {
        r#type.id()
    }

    /// `getValue(Identifier)`: the type registered under the name, or the pig.
    pub fn get_value(&self, key: &Identifier) -> EntityType {
        self.get_optional(key).unwrap_or(self.default_value)
    }

    /// `getOptional(Identifier)`: the type registered under the name, if any.
    pub fn get_optional(&self, key: &Identifier) -> Option<EntityType> {
        self.by_key.get(key).copied()
    }

    /// `byId(int)`: the type with the id, or the pig.
    pub fn by_id(&self, id: i32) -> EntityType {
        usize::try_from(id)
            .ok()
            .filter(|&id| id < self.types.len())
            .map_or(self.default_value, |id| EntityType(id as u16))
    }

    /// `byIdOrThrow(int)`, which never throws: `byId` falls back to the pig.
    pub fn by_id_or_throw(&self, id: i32) -> EntityType {
        self.by_id(id)
    }

    /// `getDefaultKey()`: `minecraft:pig`.
    pub fn get_default_key(&self) -> &Identifier {
        &self.default_key
    }

    /// Every type's key, by id.
    pub fn keys(&self) -> &[ResourceKey] {
        &self.keys
    }

    /// Every type, by id.
    pub fn iter(&self) -> impl Iterator<Item = EntityType> + use<> {
        (0..self.types.len() as u16).map(EntityType)
    }

    pub fn size(&self) -> i32 {
        self.types.len() as i32
    }
}

/// `BuiltInRegistries.ENTITY_TYPE` while `EntityTypes` registers into it.
#[derive(Default)]
struct Registrar {
    types: Vec<EntityTypeData>,
    keys: Vec<ResourceKey>,
}

/// `register(id, builder)`: the type the builder makes, under the id.
fn register(registry: &mut Registrar, id: &ResourceKey, builder: Builder) -> EntityType {
    let r#type = EntityType(registry.types.len() as u16);
    registry.types.push(builder.build(id));
    registry.keys.push(id.clone());
    r#type
}

macro_rules! entity_types {
    ($registry:ident; $(pub const $name:ident: EntityType = $register:expr;)*) => {
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[derive(Clone, Copy)]
        enum Field {
            $($name,)*
        }

        impl EntityTypes {
            $(pub const $name: EntityType = EntityType(Field::$name as u16);)*
        }

        fn register_all($registry: &mut Registrar) {
            $(
                let registered: EntityType = $register;
                assert!(
                    registered == EntityTypes::$name,
                    concat!(stringify!($name), " isn't registered where its field says")
                );
            )*
        }

        /// Each field's name and type, in declaration order.
        #[cfg(test)]
        const FIELDS: &[(&str, EntityType)] = &[$((stringify!($name), EntityTypes::$name),)*];
    };
}

pub struct EntityTypes;

impl EntityTypes {
    /// The types only operators may set data on from an item (`onlyOpCanSetNbt`).
    pub(crate) const OP_ONLY_CUSTOM_DATA: [EntityType; 3] = [
        EntityTypes::FALLING_BLOCK,
        EntityTypes::COMMAND_BLOCK_MINECART,
        EntityTypes::SPAWNER_MINECART,
    ];

    /// Registers every type, like Java's static initializer.
    fn bootstrap() -> EntityTypeRegistry {
        let mut registrar = Registrar::default();
        register_all(&mut registrar);
        let Registrar { types, keys } = registrar;
        let by_key: HashMap<Identifier, EntityType> = keys
            .iter()
            .enumerate()
            .map(|(id, key)| (key.identifier().clone(), EntityType(id as u16)))
            .collect();
        let default_key = Identifier::parse("pig");
        let default_value = by_key[&default_key];
        EntityTypeRegistry {
            types,
            keys,
            by_key,
            default_key,
            default_value,
        }
    }
}

entity_types! {
    registry;
    pub const ACACIA_BOAT: EntityType = register(registry, EntityTypeIds::ACACIA_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const ACACIA_CHEST_BOAT: EntityType = register(registry, EntityTypeIds::ACACIA_CHEST_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const ALLAY: EntityType = register(registry, EntityTypeIds::ALLAY, Builder::of(MobCategory::CREATURE).sized(0.35, 0.6).eye_height(0.36).client_tracking_range(8).update_interval(2));
    pub const AREA_EFFECT_CLOUD: EntityType = register(registry, EntityTypeIds::AREA_EFFECT_CLOUD, Builder::of(MobCategory::MISC).no_loot_table().fire_immune().sized(6.0, 0.5).client_tracking_range(10).no_update_interval());
    pub const ARMADILLO: EntityType = register(registry, EntityTypeIds::ARMADILLO, Builder::of(MobCategory::CREATURE).sized(0.7, 0.65).eye_height(0.26).client_tracking_range(10));
    pub const ARMOR_STAND: EntityType = register(registry, EntityTypeIds::ARMOR_STAND, Builder::of(MobCategory::MISC).sized(0.5, 1.975).eye_height(1.7775).client_tracking_range(10));
    pub const ARROW: EntityType = register(registry, EntityTypeIds::ARROW, Builder::of(MobCategory::MISC).no_loot_table().sized(0.5, 0.5).eye_height(0.13).client_tracking_range(4).update_interval(20));
    pub const AXOLOTL: EntityType = register(registry, EntityTypeIds::AXOLOTL, Builder::of(MobCategory::AXOLOTLS).sized(0.75, 0.42).eye_height(0.2751).client_tracking_range(10));
    pub const BAMBOO_CHEST_RAFT: EntityType = register(registry, EntityTypeIds::BAMBOO_CHEST_RAFT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const BAMBOO_RAFT: EntityType = register(registry, EntityTypeIds::BAMBOO_RAFT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const BAT: EntityType = register(registry, EntityTypeIds::BAT, Builder::of(MobCategory::AMBIENT).sized(0.5, 0.9).eye_height(0.45).client_tracking_range(5).dont_track_deltas());
    pub const BEE: EntityType = register(registry, EntityTypeIds::BEE, Builder::of(MobCategory::CREATURE).sized(0.55, 0.5).eye_height(0.3).client_tracking_range(8));
    pub const BIRCH_BOAT: EntityType = register(registry, EntityTypeIds::BIRCH_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const BIRCH_CHEST_BOAT: EntityType = register(registry, EntityTypeIds::BIRCH_CHEST_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const BLAZE: EntityType = register(registry, EntityTypeIds::BLAZE, Builder::of(MobCategory::MONSTER).fire_immune().sized(0.6, 1.8).client_tracking_range(8).not_in_peaceful());
    pub const BLOCK_DISPLAY: EntityType = register(registry, EntityTypeIds::BLOCK_DISPLAY, Builder::of(MobCategory::MISC).no_loot_table().sized(0.0, 0.0).client_tracking_range(10).update_interval(1));
    pub const BOGGED: EntityType = register(registry, EntityTypeIds::BOGGED, Builder::of(MobCategory::MONSTER).sized(0.6, 1.99).eye_height(1.74).client_tracking_range(8).not_in_peaceful());
    pub const BREEZE: EntityType = register(registry, EntityTypeIds::BREEZE, Builder::of(MobCategory::MONSTER).sized(0.6, 1.77).eye_height(1.3452).client_tracking_range(10).not_in_peaceful());
    pub const BREEZE_WIND_CHARGE: EntityType = register(registry, EntityTypeIds::BREEZE_WIND_CHARGE, Builder::of(MobCategory::MISC).no_loot_table().sized(0.3125, 0.3125).eye_height(0.0).client_tracking_range(4).update_interval(10));
    pub const CAMEL: EntityType = register(registry, EntityTypeIds::CAMEL, Builder::of(MobCategory::CREATURE).sized(1.7, 2.375).eye_height(2.275).client_tracking_range(10));
    pub const CAMEL_HUSK: EntityType = register(registry, EntityTypeIds::CAMEL_HUSK, Builder::of(MobCategory::MONSTER).sized(1.7, 2.375).eye_height(2.275).client_tracking_range(10));
    pub const CAT: EntityType = register(registry, EntityTypeIds::CAT, Builder::of(MobCategory::CREATURE).sized(0.6, 0.7).eye_height(0.35).client_tracking_range(8));
    pub const CAVE_SPIDER: EntityType = register(registry, EntityTypeIds::CAVE_SPIDER, Builder::of(MobCategory::MONSTER).sized(0.7, 0.5).eye_height(0.45).client_tracking_range(8).not_in_peaceful());
    pub const CHERRY_BOAT: EntityType = register(registry, EntityTypeIds::CHERRY_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const CHERRY_CHEST_BOAT: EntityType = register(registry, EntityTypeIds::CHERRY_CHEST_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const CHEST_MINECART: EntityType = register(registry, EntityTypeIds::CHEST_MINECART, Builder::of(MobCategory::MISC).no_loot_table().sized(0.98, 0.7).client_tracking_range(8));
    pub const CHICKEN: EntityType = register(registry, EntityTypeIds::CHICKEN, Builder::of(MobCategory::CREATURE).sized(0.4, 0.7).eye_height(0.644).client_tracking_range(10));
    pub const COD: EntityType = register(registry, EntityTypeIds::COD, Builder::of(MobCategory::WATER_AMBIENT).sized(0.5, 0.3).eye_height(0.195).client_tracking_range(4));
    pub const COPPER_GOLEM: EntityType = register(registry, EntityTypeIds::COPPER_GOLEM, Builder::of(MobCategory::MISC).sized(0.49, 0.98).eye_height(0.8125).client_tracking_range(10));
    pub const COMMAND_BLOCK_MINECART: EntityType = register(registry, EntityTypeIds::COMMAND_BLOCK_MINECART, Builder::of(MobCategory::MISC).no_loot_table().sized(0.98, 0.7).client_tracking_range(8));
    pub const COW: EntityType = register(registry, EntityTypeIds::COW, Builder::of(MobCategory::CREATURE).sized(0.9, 1.4).eye_height(1.3).client_tracking_range(10));
    pub const CREAKING: EntityType = register(registry, EntityTypeIds::CREAKING, Builder::of(MobCategory::MONSTER).sized(0.9, 2.7).eye_height(2.3).client_tracking_range(8).not_in_peaceful());
    pub const CREEPER: EntityType = register(registry, EntityTypeIds::CREEPER, Builder::of(MobCategory::MONSTER).sized(0.6, 1.7).client_tracking_range(8).not_in_peaceful());
    pub const CUSHION: EntityType = register(registry, EntityTypeIds::CUSHION, Builder::of(MobCategory::MISC).no_loot_table().sized(1.0, 0.25).client_tracking_range(10).update_interval(i32::MAX).dont_track_deltas());
    pub const DARK_OAK_BOAT: EntityType = register(registry, EntityTypeIds::DARK_OAK_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const DARK_OAK_CHEST_BOAT: EntityType = register(registry, EntityTypeIds::DARK_OAK_CHEST_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const DOLPHIN: EntityType = register(registry, EntityTypeIds::DOLPHIN, Builder::of(MobCategory::WATER_CREATURE).sized(0.9, 0.6).eye_height(0.3));
    pub const DONKEY: EntityType = register(registry, EntityTypeIds::DONKEY, Builder::of(MobCategory::CREATURE).sized(1.3964844, 1.5).eye_height(1.425).client_tracking_range(10));
    pub const DRAGON_FIREBALL: EntityType = register(registry, EntityTypeIds::DRAGON_FIREBALL, Builder::of(MobCategory::MISC).no_loot_table().sized(1.0, 1.0).client_tracking_range(4).update_interval(10));
    pub const DROWNED: EntityType = register(registry, EntityTypeIds::DROWNED, Builder::of(MobCategory::MONSTER).sized(0.6, 1.95).eye_height(1.74).client_tracking_range(8).not_in_peaceful());
    pub const EGG: EntityType = register(registry, EntityTypeIds::EGG, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).client_tracking_range(4).update_interval(10));
    pub const ELDER_GUARDIAN: EntityType = register(registry, EntityTypeIds::ELDER_GUARDIAN, Builder::of(MobCategory::MONSTER).sized(1.9975, 1.9975).eye_height(0.99875).client_tracking_range(10).not_in_peaceful());
    pub const ENDERMAN: EntityType = register(registry, EntityTypeIds::ENDERMAN, Builder::of(MobCategory::MONSTER).sized(0.6, 2.9).eye_height(2.55).client_tracking_range(8).not_in_peaceful());
    pub const ENDERMITE: EntityType = register(registry, EntityTypeIds::ENDERMITE, Builder::of(MobCategory::MONSTER).sized(0.4, 0.3).eye_height(0.13).client_tracking_range(8).not_in_peaceful());
    pub const ENDER_DRAGON: EntityType = register(registry, EntityTypeIds::ENDER_DRAGON, Builder::of(MobCategory::MONSTER).fire_immune().sized(16.0, 8.0).client_tracking_range(10));
    pub const ENDER_PEARL: EntityType = register(registry, EntityTypeIds::ENDER_PEARL, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).client_tracking_range(4).update_interval(10));
    pub const END_CRYSTAL: EntityType = register(registry, EntityTypeIds::END_CRYSTAL, Builder::of(MobCategory::MISC).no_loot_table().fire_immune().sized(2.0, 2.0).client_tracking_range(16).no_update_interval().dont_track_deltas());
    pub const EVOKER: EntityType = register(registry, EntityTypeIds::EVOKER, Builder::of(MobCategory::MONSTER).sized(0.6, 1.95).client_tracking_range(8).not_in_peaceful());
    pub const EVOKER_FANGS: EntityType = register(registry, EntityTypeIds::EVOKER_FANGS, Builder::of(MobCategory::MISC).no_loot_table().sized(0.5, 0.8).client_tracking_range(6).update_interval(2).dont_track_deltas());
    pub const EXPERIENCE_BOTTLE: EntityType = register(registry, EntityTypeIds::EXPERIENCE_BOTTLE, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).client_tracking_range(4).update_interval(10));
    pub const EXPERIENCE_ORB: EntityType = register(registry, EntityTypeIds::EXPERIENCE_ORB, Builder::of(MobCategory::MISC).no_loot_table().sized(0.5, 0.5).client_tracking_range(6).update_interval(20));
    pub const EYE_OF_ENDER: EntityType = register(registry, EntityTypeIds::EYE_OF_ENDER, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).client_tracking_range(4).update_interval(4));
    pub const FALLING_BLOCK: EntityType = register(registry, EntityTypeIds::FALLING_BLOCK, Builder::of(MobCategory::MISC).no_loot_table().sized(0.98, 0.98).client_tracking_range(10).update_interval(20));
    pub const FIREBALL: EntityType = register(registry, EntityTypeIds::FIREBALL, Builder::of(MobCategory::MISC).no_loot_table().sized(1.0, 1.0).client_tracking_range(4).update_interval(10));
    pub const FIREWORK_ROCKET: EntityType = register(registry, EntityTypeIds::FIREWORK_ROCKET, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).client_tracking_range(4).update_interval(10));
    pub const FOX: EntityType = register(registry, EntityTypeIds::FOX, Builder::of(MobCategory::CREATURE).sized(0.6, 0.7).eye_height(0.4).client_tracking_range(8).immune_to(BlockTags::FOX_IMMUNE_TO));
    pub const FROG: EntityType = register(registry, EntityTypeIds::FROG, Builder::of(MobCategory::CREATURE).sized(0.5, 0.5).client_tracking_range(10));
    pub const FURNACE_MINECART: EntityType = register(registry, EntityTypeIds::FURNACE_MINECART, Builder::of(MobCategory::MISC).no_loot_table().sized(0.98, 0.7).client_tracking_range(8));
    pub const GHAST: EntityType = register(registry, EntityTypeIds::GHAST, Builder::of(MobCategory::MONSTER).fire_immune().sized(4.0, 4.0).eye_height(2.6).client_tracking_range(10).not_in_peaceful());
    pub const HAPPY_GHAST: EntityType = register(registry, EntityTypeIds::HAPPY_GHAST, Builder::of(MobCategory::CREATURE).sized(4.0, 4.0).eye_height(2.6).client_tracking_range(10));
    pub const GIANT: EntityType = register(registry, EntityTypeIds::GIANT, Builder::of(MobCategory::MONSTER).sized(3.6, 12.0).eye_height(10.44).client_tracking_range(10).not_in_peaceful());
    pub const GLOW_ITEM_FRAME: EntityType = register(registry, EntityTypeIds::GLOW_ITEM_FRAME, Builder::of(MobCategory::MISC).no_loot_table().sized(0.5, 0.5).eye_height(0.0).client_tracking_range(10).no_update_interval().dont_track_deltas());
    pub const GLOW_SQUID: EntityType = register(registry, EntityTypeIds::GLOW_SQUID, Builder::of(MobCategory::UNDERGROUND_WATER_CREATURE).sized(0.8, 0.8).eye_height(0.4).client_tracking_range(10));
    pub const GOAT: EntityType = register(registry, EntityTypeIds::GOAT, Builder::of(MobCategory::CREATURE).sized(0.9, 1.3).client_tracking_range(10));
    pub const GUARDIAN: EntityType = register(registry, EntityTypeIds::GUARDIAN, Builder::of(MobCategory::MONSTER).sized(0.85, 0.85).eye_height(0.425).client_tracking_range(8).not_in_peaceful());
    pub const HOGLIN: EntityType = register(registry, EntityTypeIds::HOGLIN, Builder::of(MobCategory::MONSTER).sized(1.3964844, 1.4).client_tracking_range(8).not_in_peaceful());
    pub const HOPPER_MINECART: EntityType = register(registry, EntityTypeIds::HOPPER_MINECART, Builder::of(MobCategory::MISC).no_loot_table().sized(0.98, 0.7).client_tracking_range(8));
    pub const HORSE: EntityType = register(registry, EntityTypeIds::HORSE, Builder::of(MobCategory::CREATURE).sized(1.3964844, 1.6).eye_height(1.52).client_tracking_range(10));
    pub const HUSK: EntityType = register(registry, EntityTypeIds::HUSK, Builder::of(MobCategory::MONSTER).sized(0.6, 1.95).eye_height(1.74).client_tracking_range(8).not_in_peaceful());
    pub const ILLUSIONER: EntityType = register(registry, EntityTypeIds::ILLUSIONER, Builder::of(MobCategory::MONSTER).sized(0.6, 1.95).client_tracking_range(8).not_in_peaceful());
    pub const INTERACTION: EntityType = register(registry, EntityTypeIds::INTERACTION, Builder::of(MobCategory::MISC).no_loot_table().sized(0.0, 0.0).client_tracking_range(10));
    pub const IRON_GOLEM: EntityType = register(registry, EntityTypeIds::IRON_GOLEM, Builder::of(MobCategory::MISC).sized(1.4, 2.7).client_tracking_range(10));
    pub const ITEM: EntityType = register(registry, EntityTypeIds::ITEM, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).eye_height(0.2125).client_tracking_range(6).update_interval(20));
    pub const ITEM_DISPLAY: EntityType = register(registry, EntityTypeIds::ITEM_DISPLAY, Builder::of(MobCategory::MISC).no_loot_table().sized(0.0, 0.0).client_tracking_range(10).update_interval(1));
    pub const ITEM_FRAME: EntityType = register(registry, EntityTypeIds::ITEM_FRAME, Builder::of(MobCategory::MISC).no_loot_table().sized(0.5, 0.5).eye_height(0.0).client_tracking_range(10).no_update_interval().dont_track_deltas());
    pub const JUNGLE_BOAT: EntityType = register(registry, EntityTypeIds::JUNGLE_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const JUNGLE_CHEST_BOAT: EntityType = register(registry, EntityTypeIds::JUNGLE_CHEST_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const LEASH_KNOT: EntityType = register(registry, EntityTypeIds::LEASH_KNOT, Builder::of(MobCategory::MISC).no_loot_table().no_save().sized(0.375, 0.5).eye_height(0.0625).client_tracking_range(10).no_update_interval().dont_track_deltas());
    pub const LIGHTNING_BOLT: EntityType = register(registry, EntityTypeIds::LIGHTNING_BOLT, Builder::of(MobCategory::MISC).no_loot_table().no_save().sized(0.0, 0.0).client_tracking_range(16).no_update_interval());
    pub const LLAMA: EntityType = register(registry, EntityTypeIds::LLAMA, Builder::of(MobCategory::CREATURE).sized(0.9, 1.87).eye_height(1.7765).client_tracking_range(10));
    pub const LLAMA_SPIT: EntityType = register(registry, EntityTypeIds::LLAMA_SPIT, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).client_tracking_range(4).update_interval(10).dont_track_deltas());
    pub const MAGMA_CUBE: EntityType = register(registry, EntityTypeIds::MAGMA_CUBE, Builder::of(MobCategory::MONSTER).fire_immune().sized(0.52, 0.52).eye_height(0.325).spawn_dimensions_scale(4.0).client_tracking_range(8).not_in_peaceful());
    pub const MANGROVE_BOAT: EntityType = register(registry, EntityTypeIds::MANGROVE_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const MANGROVE_CHEST_BOAT: EntityType = register(registry, EntityTypeIds::MANGROVE_CHEST_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const MANNEQUIN: EntityType = register(registry, EntityTypeIds::MANNEQUIN, Builder::of(MobCategory::MISC).sized(0.6, 1.8).eye_height(1.62).client_tracking_range(32).update_interval(2));
    pub const MARKER: EntityType = register(registry, EntityTypeIds::MARKER, Builder::of(MobCategory::MISC).no_loot_table().sized(0.0, 0.0).client_tracking_range(0));
    pub const MINECART: EntityType = register(registry, EntityTypeIds::MINECART, Builder::of(MobCategory::MISC).no_loot_table().sized(0.98, 0.7).client_tracking_range(8));
    pub const MOOSHROOM: EntityType = register(registry, EntityTypeIds::MOOSHROOM, Builder::of(MobCategory::CREATURE).sized(0.9, 1.4).eye_height(1.3).client_tracking_range(10));
    pub const MULE: EntityType = register(registry, EntityTypeIds::MULE, Builder::of(MobCategory::CREATURE).sized(1.3964844, 1.6).eye_height(1.52).client_tracking_range(8));
    pub const NAUTILUS: EntityType = register(registry, EntityTypeIds::NAUTILUS, Builder::of(MobCategory::WATER_CREATURE).sized(0.875, 0.95).eye_height(0.2751).client_tracking_range(10));
    pub const OAK_BOAT: EntityType = register(registry, EntityTypeIds::OAK_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const OAK_CHEST_BOAT: EntityType = register(registry, EntityTypeIds::OAK_CHEST_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const OCELOT: EntityType = register(registry, EntityTypeIds::OCELOT, Builder::of(MobCategory::CREATURE).sized(0.6, 0.7).client_tracking_range(10));
    pub const OMINOUS_ITEM_SPAWNER: EntityType = register(registry, EntityTypeIds::OMINOUS_ITEM_SPAWNER, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).client_tracking_range(8));
    pub const PAINTING: EntityType = register(registry, EntityTypeIds::PAINTING, Builder::of(MobCategory::MISC).no_loot_table().sized(0.5, 0.5).client_tracking_range(10).no_update_interval().dont_track_deltas());
    pub const PALE_OAK_BOAT: EntityType = register(registry, EntityTypeIds::PALE_OAK_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const PALE_OAK_CHEST_BOAT: EntityType = register(registry, EntityTypeIds::PALE_OAK_CHEST_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const PANDA: EntityType = register(registry, EntityTypeIds::PANDA, Builder::of(MobCategory::CREATURE).sized(1.3, 1.25).client_tracking_range(10));
    pub const PARCHED: EntityType = register(registry, EntityTypeIds::PARCHED, Builder::of(MobCategory::MONSTER).sized(0.6, 1.99).eye_height(1.74).client_tracking_range(8).not_in_peaceful());
    pub const PARROT: EntityType = register(registry, EntityTypeIds::PARROT, Builder::of(MobCategory::CREATURE).sized(0.5, 0.9).eye_height(0.54).client_tracking_range(8));
    pub const PHANTOM: EntityType = register(registry, EntityTypeIds::PHANTOM, Builder::of(MobCategory::MONSTER).sized(0.9, 0.5).eye_height(0.175).client_tracking_range(8).not_in_peaceful());
    pub const PIG: EntityType = register(registry, EntityTypeIds::PIG, Builder::of(MobCategory::CREATURE).sized(0.9, 0.9).client_tracking_range(10));
    pub const PIGLIN: EntityType = register(registry, EntityTypeIds::PIGLIN, Builder::of(MobCategory::MONSTER).sized(0.6, 1.95).eye_height(1.79).client_tracking_range(8));
    pub const PIGLIN_BRUTE: EntityType = register(registry, EntityTypeIds::PIGLIN_BRUTE, Builder::of(MobCategory::MONSTER).sized(0.6, 1.95).eye_height(1.79).client_tracking_range(8).not_in_peaceful());
    pub const PILLAGER: EntityType = register(registry, EntityTypeIds::PILLAGER, Builder::of(MobCategory::MONSTER).can_spawn_far_from_player().sized(0.6, 1.95).client_tracking_range(8).not_in_peaceful());
    pub const POLAR_BEAR: EntityType = register(registry, EntityTypeIds::POLAR_BEAR, Builder::of(MobCategory::CREATURE).immune_to(BlockTags::POLAR_BEAR_IMMUNE_TO).sized(1.4, 1.4).client_tracking_range(10));
    pub const POPLAR_BOAT: EntityType = register(registry, EntityTypeIds::POPLAR_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const POPLAR_CHEST_BOAT: EntityType = register(registry, EntityTypeIds::POPLAR_CHEST_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const SPLASH_POTION: EntityType = register(registry, EntityTypeIds::SPLASH_POTION, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).client_tracking_range(4).update_interval(10));
    pub const LINGERING_POTION: EntityType = register(registry, EntityTypeIds::LINGERING_POTION, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).client_tracking_range(4).update_interval(10));
    pub const PUFFERFISH: EntityType = register(registry, EntityTypeIds::PUFFERFISH, Builder::of(MobCategory::WATER_AMBIENT).sized(0.7, 0.7).eye_height(0.455).client_tracking_range(4));
    pub const RABBIT: EntityType = register(registry, EntityTypeIds::RABBIT, Builder::of(MobCategory::CREATURE).sized(0.49, 0.6).eye_height(0.59).client_tracking_range(8));
    pub const RAVAGER: EntityType = register(registry, EntityTypeIds::RAVAGER, Builder::of(MobCategory::MONSTER).sized(1.95, 2.2).client_tracking_range(10).not_in_peaceful());
    pub const SALMON: EntityType = register(registry, EntityTypeIds::SALMON, Builder::of(MobCategory::WATER_AMBIENT).sized(0.7, 0.4).eye_height(0.26).client_tracking_range(4));
    pub const SHEEP: EntityType = register(registry, EntityTypeIds::SHEEP, Builder::of(MobCategory::CREATURE).sized(0.9, 1.3).eye_height(1.235).client_tracking_range(10));
    pub const SHULKER: EntityType = register(registry, EntityTypeIds::SHULKER, Builder::of(MobCategory::MONSTER).fire_immune().can_spawn_far_from_player().sized(1.0, 1.0).eye_height(0.5).client_tracking_range(10));
    pub const SHULKER_BULLET: EntityType = register(registry, EntityTypeIds::SHULKER_BULLET, Builder::of(MobCategory::MISC).no_loot_table().sized(0.3125, 0.3125).client_tracking_range(8));
    pub const SILVERFISH: EntityType = register(registry, EntityTypeIds::SILVERFISH, Builder::of(MobCategory::MONSTER).sized(0.4, 0.3).eye_height(0.13).client_tracking_range(8).not_in_peaceful());
    pub const SKELETON: EntityType = register(registry, EntityTypeIds::SKELETON, Builder::of(MobCategory::MONSTER).sized(0.6, 1.99).eye_height(1.74).client_tracking_range(8).not_in_peaceful());
    pub const SKELETON_HORSE: EntityType = register(registry, EntityTypeIds::SKELETON_HORSE, Builder::of(MobCategory::CREATURE).sized(1.3964844, 1.6).eye_height(1.52).client_tracking_range(10));
    pub const SLIME: EntityType = register(registry, EntityTypeIds::SLIME, Builder::of(MobCategory::MONSTER).sized(0.52, 0.52).eye_height(0.325).spawn_dimensions_scale(4.0).client_tracking_range(10).not_in_peaceful());
    pub const SMALL_FIREBALL: EntityType = register(registry, EntityTypeIds::SMALL_FIREBALL, Builder::of(MobCategory::MISC).no_loot_table().sized(0.3125, 0.3125).client_tracking_range(4).update_interval(10));
    pub const SNIFFER: EntityType = register(registry, EntityTypeIds::SNIFFER, Builder::of(MobCategory::CREATURE).sized(1.9, 1.75).eye_height(1.05).client_tracking_range(10));
    pub const SNOWBALL: EntityType = register(registry, EntityTypeIds::SNOWBALL, Builder::of(MobCategory::MISC).no_loot_table().sized(0.25, 0.25).client_tracking_range(4).update_interval(10));
    pub const SNOW_GOLEM: EntityType = register(registry, EntityTypeIds::SNOW_GOLEM, Builder::of(MobCategory::MISC).immune_to(BlockTags::SNOW_GOLEM_IMMUNE_TO).sized(0.7, 1.9).eye_height(1.7).client_tracking_range(8));
    pub const SPAWNER_MINECART: EntityType = register(registry, EntityTypeIds::SPAWNER_MINECART, Builder::of(MobCategory::MISC).no_loot_table().sized(0.98, 0.7).client_tracking_range(8));
    pub const SPECTRAL_ARROW: EntityType = register(registry, EntityTypeIds::SPECTRAL_ARROW, Builder::of(MobCategory::MISC).no_loot_table().sized(0.5, 0.5).eye_height(0.13).client_tracking_range(4).update_interval(20));
    pub const SPIDER: EntityType = register(registry, EntityTypeIds::SPIDER, Builder::of(MobCategory::MONSTER).sized(1.4, 0.9).eye_height(0.65).client_tracking_range(8).not_in_peaceful());
    pub const SPRUCE_BOAT: EntityType = register(registry, EntityTypeIds::SPRUCE_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const SPRUCE_CHEST_BOAT: EntityType = register(registry, EntityTypeIds::SPRUCE_CHEST_BOAT, Builder::of(MobCategory::MISC).no_loot_table().sized(1.375, 0.5625).eye_height(0.5625).client_tracking_range(10));
    pub const SQUID: EntityType = register(registry, EntityTypeIds::SQUID, Builder::of(MobCategory::WATER_CREATURE).sized(0.8, 0.8).eye_height(0.4).client_tracking_range(8));
    pub const STRAY: EntityType = register(registry, EntityTypeIds::STRAY, Builder::of(MobCategory::MONSTER).sized(0.6, 1.99).eye_height(1.74).immune_to(BlockTags::STRAY_IMMUNE_TO).client_tracking_range(8).not_in_peaceful());
    pub const STRIDER: EntityType = register(registry, EntityTypeIds::STRIDER, Builder::of(MobCategory::CREATURE).fire_immune().sized(0.9, 1.7).client_tracking_range(10));
    pub const SULFUR_CUBE: EntityType = register(registry, EntityTypeIds::SULFUR_CUBE, Builder::of(MobCategory::MONSTER).sized(0.49, 0.49).eye_height(0.175).spawn_dimensions_scale(2.0).client_tracking_range(10));
    pub const TADPOLE: EntityType = register(registry, EntityTypeIds::TADPOLE, Builder::of(MobCategory::CREATURE).sized(0.4, 0.3).eye_height(0.19500001).client_tracking_range(10));
    pub const TEXT_DISPLAY: EntityType = register(registry, EntityTypeIds::TEXT_DISPLAY, Builder::of(MobCategory::MISC).no_loot_table().sized(0.0, 0.0).client_tracking_range(10).update_interval(1));
    pub const TNT: EntityType = register(registry, EntityTypeIds::TNT, Builder::of(MobCategory::MISC).no_loot_table().fire_immune().sized(0.98, 0.98).eye_height(0.15).client_tracking_range(10).update_interval(10));
    pub const TNT_MINECART: EntityType = register(registry, EntityTypeIds::TNT_MINECART, Builder::of(MobCategory::MISC).no_loot_table().sized(0.98, 0.7).client_tracking_range(8));
    pub const TRADER_LLAMA: EntityType = register(registry, EntityTypeIds::TRADER_LLAMA, Builder::of(MobCategory::CREATURE).sized(0.9, 1.87).eye_height(1.7765).client_tracking_range(10));
    pub const TRIDENT: EntityType = register(registry, EntityTypeIds::TRIDENT, Builder::of(MobCategory::MISC).no_loot_table().sized(0.5, 0.5).eye_height(0.13).client_tracking_range(4).update_interval(20));
    pub const TROPICAL_FISH: EntityType = register(registry, EntityTypeIds::TROPICAL_FISH, Builder::of(MobCategory::WATER_AMBIENT).sized(0.5, 0.4).eye_height(0.26).client_tracking_range(4));
    pub const TURTLE: EntityType = register(registry, EntityTypeIds::TURTLE, Builder::of(MobCategory::CREATURE).sized(1.2, 0.4).client_tracking_range(10));
    pub const VEX: EntityType = register(registry, EntityTypeIds::VEX, Builder::of(MobCategory::MONSTER).fire_immune().sized(0.4, 0.8).eye_height(0.51875).client_tracking_range(8).not_in_peaceful());
    pub const VILLAGER: EntityType = register(registry, EntityTypeIds::VILLAGER, Builder::of(MobCategory::MISC).sized(0.6, 1.95).eye_height(1.62).client_tracking_range(10));
    pub const VINDICATOR: EntityType = register(registry, EntityTypeIds::VINDICATOR, Builder::of(MobCategory::MONSTER).sized(0.6, 1.95).client_tracking_range(8).not_in_peaceful());
    pub const WANDERING_TRADER: EntityType = register(registry, EntityTypeIds::WANDERING_TRADER, Builder::of(MobCategory::CREATURE).sized(0.6, 1.95).eye_height(1.62).client_tracking_range(10));
    pub const WARDEN: EntityType = register(registry, EntityTypeIds::WARDEN, Builder::of(MobCategory::MONSTER).sized(0.9, 2.9).client_tracking_range(16).fire_immune().not_in_peaceful());
    pub const WIND_CHARGE: EntityType = register(registry, EntityTypeIds::WIND_CHARGE, Builder::of(MobCategory::MISC).no_loot_table().sized(0.3125, 0.3125).eye_height(0.0).client_tracking_range(4).update_interval(10));
    pub const WITCH: EntityType = register(registry, EntityTypeIds::WITCH, Builder::of(MobCategory::MONSTER).sized(0.6, 1.95).eye_height(1.62).client_tracking_range(8).not_in_peaceful());
    pub const WITHER: EntityType = register(registry, EntityTypeIds::WITHER, Builder::of(MobCategory::MONSTER).fire_immune().immune_to(BlockTags::WITHER_IMMUNE_TO).sized(0.9, 3.5).client_tracking_range(10).not_in_peaceful().dont_track_deltas());
    pub const WITHER_SKELETON: EntityType = register(registry, EntityTypeIds::WITHER_SKELETON, Builder::of(MobCategory::MONSTER).fire_immune().immune_to(BlockTags::WITHER_SKELETON_IMMUNE_TO).sized(0.7, 2.4).eye_height(2.1).client_tracking_range(8).not_in_peaceful());
    pub const WITHER_SKULL: EntityType = register(registry, EntityTypeIds::WITHER_SKULL, Builder::of(MobCategory::MISC).no_loot_table().sized(0.3125, 0.3125).client_tracking_range(4).update_interval(10));
    pub const WOLF: EntityType = register(registry, EntityTypeIds::WOLF, Builder::of(MobCategory::CREATURE).sized(0.6, 0.85).eye_height(0.68).client_tracking_range(10));
    pub const ZOGLIN: EntityType = register(registry, EntityTypeIds::ZOGLIN, Builder::of(MobCategory::MONSTER).fire_immune().sized(1.3964844, 1.4).client_tracking_range(8).not_in_peaceful());
    pub const ZOMBIE: EntityType = register(registry, EntityTypeIds::ZOMBIE, Builder::of(MobCategory::MONSTER).sized(0.6, 1.95).eye_height(1.74).client_tracking_range(8).not_in_peaceful());
    pub const ZOMBIE_HORSE: EntityType = register(registry, EntityTypeIds::ZOMBIE_HORSE, Builder::of(MobCategory::MONSTER).sized(1.3964844, 1.6).eye_height(1.52).client_tracking_range(10));
    pub const ZOMBIE_NAUTILUS: EntityType = register(registry, EntityTypeIds::ZOMBIE_NAUTILUS, Builder::of(MobCategory::MONSTER).sized(0.875, 0.95).eye_height(0.2751).client_tracking_range(10));
    pub const ZOMBIE_VILLAGER: EntityType = register(registry, EntityTypeIds::ZOMBIE_VILLAGER, Builder::of(MobCategory::MONSTER).sized(0.6, 1.95).eye_height(1.74).client_tracking_range(8).not_in_peaceful());
    pub const ZOMBIFIED_PIGLIN: EntityType = register(registry, EntityTypeIds::ZOMBIFIED_PIGLIN, Builder::of(MobCategory::MONSTER).fire_immune().sized(0.6, 1.95).eye_height(1.79).client_tracking_range(8).not_in_peaceful());
    pub const PLAYER: EntityType = register(registry, EntityTypeIds::PLAYER, Builder::create_nothing(MobCategory::MISC).no_save().no_summon().sized(0.6, 1.8).eye_height(1.62).client_tracking_range(32).update_interval(2).dont_track_deltas());
    pub const FISHING_BOBBER: EntityType = register(registry, EntityTypeIds::FISHING_BOBBER, Builder::of(MobCategory::MISC).no_loot_table().no_save().no_summon().sized(0.25, 0.25).client_tracking_range(4).update_interval(5));
}

#[cfg(test)]
mod tests {
    use super::FIELDS;
    use crate::golden::Golden;
    use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::world::entity::entity_type::{self, EntityType};

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.EntityTypes");
        let registry = &BuiltInRegistries::ENTITY_TYPE;
        for &(name, r#type) in FIELDS {
            golden.field(name, entity_type::tests::values(r#type));
        }
        golden.field("size", registry.size());
        golden.field("defaultKey", registry.get_default_key().to_string());
        golden.check("byId", |id: i32| registry.by_id(id));
        golden.check("getValue", |key: String| {
            registry.get_value(&Identifier::parse(&key))
        });
        golden.check("getOptional", |key: String| {
            registry.get_optional(&Identifier::parse(&key))
        });
        golden.finish();
        assert!(registry.iter().eq(FIELDS.iter().map(|&(_, r#type)| r#type)));
        assert!(
            registry
                .iter()
                .all(|r#type| registry.get_value(EntityType::get_key(r#type)) == r#type)
        );
    }
}
