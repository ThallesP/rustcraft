//! net/minecraft/world/level/dimension/DimensionType.java
//!
//! A kind of dimension, an element of the synchronized `dimension_type` registry, as
//! `NETWORK_CODEC` reads it (`RegistryDataLoader`): its height, its light, its sky, its attributes
//! (`EnvironmentAttributeMap.NETWORK_CODEC`) and timelines. `DIRECT_CODEC` and `CODEC` (the
//! server's), `getTeleportationScale`, `getStorageFolder` and `MOON_BRIGHTNESS_PER_PHASE` come
//! with their callers. Its `infiniburn` and `timelines` tags are named sets whose blocks and
//! timelines come with `update_tags`.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{
    self, BOOL, CodecExt, FLOAT, SharedCodec, StaticCodec,
};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, SharedMapCodec, StaticMapCodec};
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::holder_set::HolderSet;
use crate::net::minecraft::core::registries::codec::registry_codecs::RegistryCodecs;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};
use crate::net::minecraft::util::valueproviders::int_provider::IntProvider;
use crate::net::minecraft::util::valueproviders::int_providers::IntProviders;
use crate::net::minecraft::world::attribute::environment_attribute_map::EnvironmentAttributeMap;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::level::cardinal_lighting;
use crate::net::minecraft::world::level::level_height_accessor::{self, Height};

static NETWORK_CODEC: StaticCodec<DimensionType> = LazyLock::new(|| {
    // createDirectCodec(EnvironmentAttributeMap.NETWORK_CODEC).
    ExtraCodecs::catch_decoder_exception(record_codec_builder::create(
        (
            BOOL.optional_field_of_default("has_fixed_time", false)
                .for_getter(|o: &DimensionType| o.has_fixed_time),
            BOOL.field_of("has_skylight")
                .for_getter(|o: &DimensionType| o.has_sky_light),
            BOOL.field_of("has_ceiling")
                .for_getter(|o: &DimensionType| o.has_ceiling),
            BOOL.field_of("has_ender_dragon_fight")
                .for_getter(|o: &DimensionType| o.has_ender_dragon_fight),
            // 1.0E-5F, a float widened.
            codec::double_range(f64::from(1.0E-5_f32), 3.0E7)
                .field_of("coordinate_scale")
                .for_getter(|o: &DimensionType| o.coordinate_scale),
            codec::int_range(DimensionType::MIN_Y, DimensionType::MAX_Y)
                .field_of("min_y")
                .for_getter(|o: &DimensionType| o.min_y),
            codec::int_range(16, DimensionType::Y_SIZE)
                .field_of("height")
                .for_getter(|o: &DimensionType| o.height),
            codec::int_range(0, DimensionType::Y_SIZE)
                .field_of("logical_height")
                .for_getter(|o: &DimensionType| o.logical_height),
            RegistryCodecs::holder_set(Registries::BLOCK)
                .field_of("infiniburn")
                .for_getter(|o: &DimensionType| o.infiniburn.clone()),
            FLOAT
                .field_of("ambient_light")
                .for_getter(|o: &DimensionType| o.ambient_light),
            MONSTER_SETTINGS_CODEC
                .shared()
                .for_getter(|o: &DimensionType| o.monster_settings),
            Skybox::CODEC
                .shared()
                .optional_field_of_default("skybox", Skybox::OVERWORLD)
                .for_getter(|o: &DimensionType| o.skybox),
            cardinal_lighting::Type::CODEC
                .shared()
                .optional_field_of_default("cardinal_light", cardinal_lighting::Type::DEFAULT)
                .for_getter(|o: &DimensionType| o.cardinal_light_type),
            EnvironmentAttributeMap::NETWORK_CODEC
                .shared()
                .optional_field_of_default("attributes", EnvironmentAttributeMap::empty())
                .for_getter(|o: &DimensionType| o.attributes.clone()),
            RegistryCodecs::holder_set(Registries::TIMELINE)
                .optional_field_of_default("timelines", HolderSet::empty())
                .for_getter(|o: &DimensionType| o.timelines.clone()),
            // WorldClock.CODEC.
            RegistryCodecs::holder(Registries::WORLD_CLOCK)
                .optional_field_of("default_clock")
                .for_getter(|o: &DimensionType| o.default_clock.clone()),
        ),
        DimensionType::new,
    ))
    .boxed()
});

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
    LazyLock::new(|| Box::new(ByteBufCodecs::holder_registry(Registries::DIMENSION_TYPE)));

/// The kind of a dimension: how high it is, its light, its sky.
#[derive(Clone, Debug, PartialEq)]
pub struct DimensionType {
    has_fixed_time: bool,
    has_sky_light: bool,
    has_ceiling: bool,
    has_ender_dragon_fight: bool,
    coordinate_scale: f64,
    min_y: i32,
    height: i32,
    logical_height: i32,
    infiniburn: HolderSet,
    ambient_light: f32,
    monster_settings: MonsterSettings,
    skybox: Skybox,
    cardinal_light_type: cardinal_lighting::Type,
    attributes: EnvironmentAttributeMap,
    timelines: HolderSet,
    default_clock: Option<Holder>,
}

impl DimensionType {
    pub const BITS_FOR_Y: i32 = BlockPos::PACKED_Y_LENGTH;
    pub const MIN_HEIGHT: i32 = 16;
    pub const Y_SIZE: i32 = (1 << DimensionType::BITS_FOR_Y) - 32;
    pub const MAX_Y: i32 = (DimensionType::Y_SIZE >> 1) - 1;
    pub const MIN_Y: i32 = DimensionType::MAX_Y - DimensionType::Y_SIZE + 1;
    pub const WAY_ABOVE_MAX_Y: i32 = DimensionType::MAX_Y << 4;
    pub const WAY_BELOW_MIN_Y: i32 = DimensionType::MIN_Y << 4;

    pub const NETWORK_CODEC: &'static StaticCodec<DimensionType> = &NETWORK_CODEC;

    /// A dimension type of the synchronized registry, by its id.
    pub const STREAM_CODEC: &'static StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
        &STREAM_CODEC;

    /// The record's constructor, which panics where Java's throws `IllegalStateException`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        has_fixed_time: bool,
        has_sky_light: bool,
        has_ceiling: bool,
        has_ender_dragon_fight: bool,
        coordinate_scale: f64,
        min_y: i32,
        height: i32,
        logical_height: i32,
        infiniburn: HolderSet,
        ambient_light: f32,
        monster_settings: MonsterSettings,
        skybox: Skybox,
        cardinal_light_type: cardinal_lighting::Type,
        attributes: EnvironmentAttributeMap,
        timelines: HolderSet,
        default_clock: Option<Holder>,
    ) -> DimensionType {
        if height < 16 {
            panic!("height has to be at least 16");
        }
        if min_y + height > DimensionType::MAX_Y + 1 {
            panic!(
                "min_y + height cannot be higher than: {}",
                DimensionType::MAX_Y + 1
            );
        }
        if logical_height > height {
            panic!("logical_height cannot be higher than height");
        }
        if height % 16 != 0 {
            panic!("height has to be multiple of 16");
        }
        if min_y % 16 != 0 {
            panic!("min_y has to be a multiple of 16");
        }
        DimensionType {
            has_fixed_time,
            has_sky_light,
            has_ceiling,
            has_ender_dragon_fight,
            coordinate_scale,
            min_y,
            height,
            logical_height,
            infiniburn,
            ambient_light,
            monster_settings,
            skybox,
            cardinal_light_type,
            attributes,
            timelines,
            default_clock,
        }
    }

    pub fn has_fixed_time(&self) -> bool {
        self.has_fixed_time
    }

    pub fn has_sky_light(&self) -> bool {
        self.has_sky_light
    }

    pub fn has_ceiling(&self) -> bool {
        self.has_ceiling
    }

    pub fn has_ender_dragon_fight(&self) -> bool {
        self.has_ender_dragon_fight
    }

    pub fn coordinate_scale(&self) -> f64 {
        self.coordinate_scale
    }

    pub fn min_y(&self) -> i32 {
        self.min_y
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn logical_height(&self) -> i32 {
        self.logical_height
    }

    pub fn infiniburn(&self) -> &HolderSet {
        &self.infiniburn
    }

    pub fn ambient_light(&self) -> f32 {
        self.ambient_light
    }

    pub fn monster_settings(&self) -> MonsterSettings {
        self.monster_settings
    }

    pub fn monster_spawn_light_test(&self) -> IntProvider {
        self.monster_settings.monster_spawn_light_test
    }

    pub fn monster_spawn_block_light_limit(&self) -> i32 {
        self.monster_settings.monster_spawn_block_light_limit
    }

    pub fn skybox(&self) -> Skybox {
        self.skybox
    }

    pub fn has_end_flashes(&self) -> bool {
        self.skybox == Skybox::END
    }

    pub fn cardinal_light_type(&self) -> cardinal_lighting::Type {
        self.cardinal_light_type
    }

    pub fn attributes(&self) -> &EnvironmentAttributeMap {
        &self.attributes
    }

    pub fn timelines(&self) -> &HolderSet {
        &self.timelines
    }

    pub fn default_clock(&self) -> Option<&Holder> {
        self.default_clock.as_ref()
    }

    /// The level's height (`LevelHeightAccessor`): `min_y` and `height`.
    pub fn level_height(&self) -> Height {
        level_height_accessor::create(self.min_y, self.height)
    }

    /// Its attributes' `gameplay/fast_lava`, a dimension value: the attribute's default unless
    /// the type sets it. `ClientLevel` asks it for `Level.environmentAttributes()`'s dimension
    /// value, which only the type's own layer changes.
    pub fn fast_lava(&self) -> bool {
        self.attributes
            .apply_modifier(
                EnvironmentAttributes::FAST_LAVA.id(),
                EnvironmentAttributes::FAST_LAVA.info().default_value,
            )
            .as_boolean()
    }
}

static MONSTER_SETTINGS_CODEC: StaticMapCodec<MonsterSettings> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            IntProviders::codec(0, 15)
                .field_of("monster_spawn_light_level")
                .for_getter(|o: &MonsterSettings| o.monster_spawn_light_test),
            codec::int_range(0, 15)
                .field_of("monster_spawn_block_light_limit")
                .for_getter(|o: &MonsterSettings| o.monster_spawn_block_light_limit),
        ),
        MonsterSettings::new,
    )
    .boxed()
});

/// `DimensionType.MonsterSettings`: the light monsters spawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonsterSettings {
    monster_spawn_light_test: IntProvider,
    monster_spawn_block_light_limit: i32,
}

impl MonsterSettings {
    pub const CODEC: &'static StaticMapCodec<MonsterSettings> = &MONSTER_SETTINGS_CODEC;

    pub fn new(
        monster_spawn_light_test: IntProvider,
        monster_spawn_block_light_limit: i32,
    ) -> MonsterSettings {
        MonsterSettings {
            monster_spawn_light_test,
            monster_spawn_block_light_limit,
        }
    }

    pub fn monster_spawn_light_test(&self) -> IntProvider {
        self.monster_spawn_light_test
    }

    pub fn monster_spawn_block_light_limit(&self) -> i32 {
        self.monster_spawn_block_light_limit
    }
}

static SKYBOX_CODEC: StaticCodec<Skybox> =
    LazyLock::new(|| string_representable::from_values(&Skybox::VALUES));

/// `DimensionType.Skybox`: the sky a dimension draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Skybox {
    NONE,
    OVERWORLD,
    END,
}

impl Skybox {
    pub const VALUES: [Skybox; 3] = [Skybox::NONE, Skybox::OVERWORLD, Skybox::END];
    pub const CODEC: &'static StaticCodec<Skybox> = &SKYBOX_CODEC;
}

impl StringRepresentable for Skybox {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            Skybox::NONE => "none",
            Skybox::OVERWORLD => "overworld",
            Skybox::END => "end",
        }
    }
}
