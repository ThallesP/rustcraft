//! net/minecraft/client/multiplayer/ClientLevel.java
//!
//! Not ported yet: nearly all of it. So far a level is what the login said about it, its chunks
//! (`ClientChunkCache`), reading and setting their blocks, their light (the chunk cache's
//! `LevelLightEngine`, which the queued light updates feed and `update` runs, once a frame), and
//! what an entity's movement asks of it (`CollisionGetter` and the entity's `Level`; its world
//! border is the default one until the border packets set it), and its other entities: the
//! entity storage (`TransientEntitySectionManager`'s lookup, `getEntity`, `entitiesForRendering`)
//! and the ticking ones (`EntityTickList`, `tickEntities`). An entity of a class not ported yet is
//! a plain `Entity` of its type; ticking one is its `commonTick`, its class's `tick` being what isn't
//! ported, so it stays where the server put it. The local player isn't in the storage here:
//! `Minecraft` holds it and ticks it first, as it is in Java's tick list, added at the login. An
//! entity ticks from when it's added, where Java waits for its chunk to tick (the chunk is loaded
//! first in all the server sends). Its block entities are its chunks' (`getBlockEntity`), and their
//! tickers its own (`Level.blockEntityTickers`, which it shares with its chunks), ticked in the
//! order they came by `tickBlockEntities`, as the server's block events are handled
//! (`blockEvent`, a state's `triggerEvent`). Both reach the game through `LevelTicking`: the
//! level, the particle engine and sound manager, and the player, the only entity a block pushes
//! here (`LevelPlayer`: Java's `getEntities` finds the level's other entities too, whose classes
//! aren't ported, so they don't move). Not ported: entity sections and their visibility, other
//! entities' collisions with the player, the block state predictions
//! (`blockStatePredictionHandler`, which the player's own block changes make) and neighbour
//! updates. What the renderer is told of the
//! level's changes (`setBlocksDirty`, `sendBlockUpdated`, `setSectionDirtyWithNeighbors`,
//! `setSectionRangeDirty`) goes to the `LevelExtractor` the level shares with its chunk cache and
//! the level renderer, Java's `Minecraft.levelExtractor`.
//!
//! Its biomes are its chunks' (`getNoiseBiome`, plains where it has no chunk), zoomed by its
//! `BiomeManager`, and their values the synchronized biome registry's, which the level keeps a
//! copy of. Its tints (`getBlockTint`) are `BiomeColors`' four resolvers' blends, each cached in a
//! `BlockTintCache`; the blend radius is the options', which Java reads from the game
//! (`Minecraft.getInstance().options`) and the level is given (`set_biome_blend_radius`).
//!
//! Java's light updates are `Runnable`s that close over the packet listener; here they're closures
//! of the level (`queue_light_update`), since all they touch is the level.
//!
//! The game time is what the server last sent, and each tick adds one and ticks the listener's
//! clocks (`tickTime`, the clocks a parameter, Java's `clockManager()`). The tick also steps the
//! world border's move, the rain's particles and sound around the camera (`tickWeatherEffects`,
//! with what the game hands it: `LevelEffects`) and the sky's flash, after the sky's darkening
//! (`updateSkyBrightness`, its `SKY_LIGHT_LEVEL` a parameter, from the environment attributes the
//! game renderer holds); the rest of `tick` isn't ported: block breaking progress, the End's
//! flashes and the explosions.
//!
//! Each tick the game animates it (`animateTick`): blocks near the player picked with the level's
//! random (`Level.random`), each block's and fluid's `animateTick`, and the drips under fluids,
//! their particles and sounds handed to what the game holds (`LevelEffects`, read by the blocks
//! through `LevelWithEffects`). Not ported: the biomes' ambient particles (`AMBIENT_PARTICLES`,
//! whose type the environment attributes haven't got), which only the nether's biomes have. The weather is the levels the
//! server's game events set (`setRainLevel`, `setThunderLevel`), each with the last tick's, which
//! the server steps by sending them every tick they change. `setRespawnData` keeps the spawn as it
//! came, where Java moves one outside the world border to the heightmap's top at the border's
//! center (`getWorldBorderAdjustedRespawnData`).
//!
//! Its dimension type is the synchronized registry's value of the holder the login named
//! (`dimensionType()`), which gives its height, sky light, cardinal lighting and fast lava.

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::VecDeque;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use crate::java::lang::math::Math;
use crate::net::minecraft::client::client_clock_manager::ClientClockManager;
use crate::net::minecraft::client::color::block::block_colors::BlockColors;
use crate::net::minecraft::client::color::block::block_tint_cache::BlockTintCache;
use crate::net::minecraft::client::multiplayer::client_chunk_cache::ClientChunkCache;
use crate::net::minecraft::client::options::Options;
use crate::net::minecraft::client::particle::particle_engine::ParticleEngine;
use crate::net::minecraft::client::renderer::biome_colors;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::chunk::render_section_region::TintsToGiveBack;
use crate::net::minecraft::client::renderer::extract::level_extractor::LevelExtractor;
use crate::net::minecraft::client::resources::sounds::entity_bound_sound_instance::EntityBoundSoundInstance;
use crate::net::minecraft::client::resources::sounds::simple_sound_instance::SimpleSoundInstance;
use crate::net::minecraft::client::sounds::sound_manager::{SoundInstance, SoundManager};
use crate::net::minecraft::core::block_pos::{BlockPos, MutableBlockPos};
use crate::net::minecraft::core::cursor3d::Cursor3D;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::particles::block_particle_option::BlockParticleOption;
use crate::net::minecraft::core::particles::particle_options::ParticleOptions;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::core::quart_pos::QuartPos;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::core::registry::Registry;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::server::level::particle_status::ParticleStatus;
use crate::net::minecraft::sounds::sound_event::SoundEvent;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::tags::fluid_tags::FluidTags;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::{self, RandomSource};
use crate::net::minecraft::world::attribute::attribute_type::Value;
use crate::net::minecraft::world::attribute::environment_attribute::EnvironmentAttribute;
use crate::net::minecraft::world::attribute::environment_attribute_layer::LayerLevel;
use crate::net::minecraft::world::attribute::environment_attribute_system::EnvironmentAttributeSystem;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::entity::entity::{Entity, EntityClass};
use crate::net::minecraft::world::entity::mover_type::MoverType;
use crate::net::minecraft::world::level::biome::biome::{Biome, Precipitation};
use crate::net::minecraft::world::level::biome::biome_manager::BiomeManager;
use crate::net::minecraft::world::level::biome::biome_resolver::BiomeResolver;
use crate::net::minecraft::world::level::biome::biomes::Biomes;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::campfire_block::CampfireBlock;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntity;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::border::world_border::WorldBorder;
use crate::net::minecraft::world::level::cardinal_lighting::CardinalLighting;
use crate::net::minecraft::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::net::minecraft::world::level::chunk::paletted_container_factory::PalettedContainerFactory;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::level::collision_getter::CollisionGetter;
use crate::net::minecraft::world::level::color_resolver::ColorResolver;
use crate::net::minecraft::world::level::dimension::dimension_type::DimensionType;
use crate::net::minecraft::world::level::entity::entity_lookup::EntityLookup;
use crate::net::minecraft::world::level::entity::entity_tick_list::EntityTickList;
use crate::net::minecraft::world::level::level::{Level, SharedLevelState};
use crate::net::minecraft::world::level::level_height_accessor::{Height, LevelHeightAccessor};
use crate::net::minecraft::world::level::levelgen::heightmap::Types;
use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;
use crate::net::minecraft::world::level::levelgen::random_support::RandomSupport;
use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;
use crate::net::minecraft::world::level::lighting::light_event_listener::LightEventListener;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::push_reaction::PushReaction;
use crate::net::minecraft::world::level::redstone::collecting_neighbor_updater::CollectingNeighborUpdater;
use crate::net::minecraft::world::level::storage::level_data::RespawnData;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// The level the player is in, as the client knows it.
pub struct ClientLevel {
    dimension: ResourceKey,
    dimension_type: Holder,
    dimension_type_value: DimensionType,
    height: Height,
    chunk_source: ClientChunkCache,
    server_chunk_radius: i32,
    server_simulation_distance: i32,
    is_debug: bool,
    biome_zoom_seed: i64,
    sea_level: i32,
    hardcore: bool,
    is_flat: bool,
    world_border: WorldBorder,
    /// `Level`'s `oRainLevel`, `rainLevel`, `oThunderLevel` and `thunderLevel`.
    o_rain_level: f32,
    rain_level: f32,
    o_thunder_level: f32,
    thunder_level: f32,
    sky_flash_time: i32,
    /// `rainSoundTime`: ticks since the rain's sound last played, which make the next likelier.
    rain_sound_time: Cell<i32>,
    /// `Level.skyDarken`: how far the sky's light is down, by the time of day and the weather.
    sky_darken: i32,
    /// `Level.random`, which `animateTick`'s positions, the particles' and sounds' choices and
    /// some blocks draw from.
    random: RefCell<LegacyRandomSource>,
    respawn_data: RespawnData,
    light_update_queue: VecDeque<LightUpdate>,
    /// The biome registry, shared with the section compiles' copies of the level
    /// (`RenderSectionRegion`).
    biomes: Arc<Registry>,
    biome_manager: BiomeManager,
    tint_caches: [(&'static ColorResolver, BlockTintCache); 4],
    biome_blend_radius: i32,
    /// `entityStorage`'s entities and `tickingEntities`, the player's aside.
    entity_storage: EntityLookup<Entity>,
    ticking_entities: EntityTickList,
    /// `Minecraft.levelExtractor`, which the level's changes mark sections dirty in.
    level_extractor: Rc<LevelExtractor>,
    /// `Level.blockEntityTickers` and the pending ones, which the chunks add to, and
    /// `levelData`'s game time, which they read.
    block_entity_tickers: Rc<SharedLevelState>,
    /// `Level.neighborUpdater`, with the client's limit of chained updates.
    neighbor_updater: CollectingNeighborUpdater,
}

/// The player as a block entity's ticker finds it among the level's entities (Java's
/// `getEntities`, `players()`): what it reads of it, and its `move`, which a shulker box's lid
/// pushes it with.
pub trait LevelPlayer {
    fn get_bounding_box(&self) -> AABB;

    fn position(&self) -> Vec3;

    /// `isSpectator()`, which `EntitySelector.NO_SPECTATORS` leaves out.
    fn is_spectating(&self) -> bool;

    /// `isAlive()`.
    fn is_alive(&self) -> bool;

    /// `getPistonPushReaction()`: an entity's, `PUSH_PULL`.
    fn get_piston_push_reaction(&self) -> PushReaction {
        PushReaction::PUSH_PULL
    }

    fn get_delta_movement(&self) -> Vec3;

    fn set_delta_movement(&mut self, delta_movement: Vec3);

    fn on_ground(&self) -> bool;

    /// `isSupportedBy(pos)`: whether it stands on that block.
    fn is_supported_by(&self, pos: &BlockPos) -> bool;

    /// `move(MoverType, Vec3)`.
    fn r#move(&mut self, level: &ClientLevel, mover_type: MoverType, delta: Vec3);
}

/// The level as a block event or a block entity's ticker reaches it, Java's `Level` argument: the
/// level, which they change, what their particles and sounds reach of the game, and the player,
/// which their blocks push.
pub struct LevelTicking<'a> {
    pub level: &'a mut ClientLevel,
    pub effects: &'a LevelEffects<'a>,
    pub player: Option<&'a mut dyn LevelPlayer>,
}

impl LevelTicking<'_> {
    /// The level with its particles and sounds, as `addParticle` and `playLocalSound` read it.
    pub fn effects_level(&self) -> LevelWithEffects<'_> {
        LevelWithEffects::new(self.level, self.effects)
    }

    /// The block entity there, borrowed to change: one a ticker or a block event is sure of.
    pub fn block_entity(&self, pos: &BlockPos) -> RefMut<'_, BlockEntity> {
        self.level
            .get_block_entity_mut(pos)
            .unwrap_or_else(|| panic!("a block entity at {pos}"))
    }

    pub fn player_bounding_box(&self) -> Option<AABB> {
        self.player.as_deref().map(LevelPlayer::get_bounding_box)
    }

    /// `hasNearbyAlivePlayer`: whether a player that's alive and no spectator is nearer than the
    /// range (any distance for a negative one).
    pub fn has_nearby_alive_player(&self, x: f64, y: f64, z: f64, range: f64) -> bool {
        self.player.as_deref().is_some_and(|player| {
            !player.is_spectating()
                && player.is_alive()
                && (range < 0.0 || player.position().distance_to_sqr_xyz(x, y, z) < range * range)
        })
    }

    /// `getNearestPlayer(x, y, z, maxDist, false)`: the position of the nearest player that's no
    /// spectator nearer than the distance (any for a negative one).
    pub fn get_nearest_player(&self, x: f64, y: f64, z: f64, max_dist: f64) -> Option<Vec3> {
        self.player
            .as_deref()
            .filter(|player| {
                !player.is_spectating()
                    && (max_dist < 0.0
                        || player.position().distance_to_sqr_xyz(x, y, z) < max_dist * max_dist)
            })
            .map(LevelPlayer::position)
    }
}

/// A light update queued for the level, Java's `Runnable`.
pub type LightUpdate = Box<dyn FnOnce(&mut ClientLevel)>;

/// What the level's particles and sounds reach of the game, which Java's level reads from its
/// `minecraft`: the particle engine and the sound manager they're handed to, the camera they're
/// near, the options' particles, the player a sound follows, the environment attributes (which
/// the game renderer holds here) and the block colours leaves' particles take.
pub struct LevelEffects<'a> {
    /// `minecraft.particleEngine`.
    pub particle_engine: &'a dyn ParticleEngine,
    /// `minecraft.getSoundManager()`.
    pub sound_manager: &'a dyn SoundManager,
    /// `minecraft.gameRenderer.mainCamera().position()`.
    pub camera_position: Vec3,
    /// `minecraft.options.particles()`.
    pub particles: ParticleStatus,
    /// `minecraft.options.weatherRadius()`.
    pub weather_radius: i32,
    /// `minecraft.player`'s id, which `playPlayerSound` binds its sound to.
    pub player: Option<i32>,
    /// `environmentAttributes()`, read with the level as the layers see it; none before the game
    /// renderer has made them, which reads every attribute's default.
    pub environment: Option<(&'a EnvironmentAttributeSystem, &'a dyn LayerLevel)>,
    /// `Minecraft.getInstance().getBlockColors()`.
    pub block_colors: &'a BlockColors,
}

/// The level as a block's `animateTick` reads it, Java's `Level` argument: the `ClientLevel`,
/// which it derefs to, and what its particles and sounds reach of the game.
pub struct LevelWithEffects<'a> {
    level: &'a ClientLevel,
    effects: &'a LevelEffects<'a>,
}

impl Deref for LevelWithEffects<'_> {
    type Target = ClientLevel;

    fn deref(&self) -> &ClientLevel {
        self.level
    }
}

impl<'a> LevelWithEffects<'a> {
    pub fn new(level: &'a ClientLevel, effects: &'a LevelEffects<'a>) -> LevelWithEffects<'a> {
        LevelWithEffects { level, effects }
    }

    /// `addParticle(ParticleOptions, double…)`: shown however far if its type overrides the
    /// limiter.
    #[allow(clippy::too_many_arguments)]
    pub fn add_particle(
        &self,
        particle: impl Into<ParticleOptions>,
        x: f64,
        y: f64,
        z: f64,
        xd: f64,
        yd: f64,
        zd: f64,
    ) {
        let particle = particle.into();
        let override_limiter = particle.get_type().get_override_limiter();
        self.do_add_particle(&particle, override_limiter, false, x, y, z, xd, yd, zd);
    }

    /// `addParticle(ParticleOptions, boolean, boolean, double…)`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_particle_limited(
        &self,
        particle: impl Into<ParticleOptions>,
        override_limiter: bool,
        always_show: bool,
        x: f64,
        y: f64,
        z: f64,
        xd: f64,
        yd: f64,
        zd: f64,
    ) {
        let particle = particle.into();
        let override_limiter = particle.get_type().get_override_limiter() || override_limiter;
        self.do_add_particle(
            &particle,
            override_limiter,
            always_show,
            x,
            y,
            z,
            xd,
            yd,
            zd,
        );
    }

    /// `addAlwaysVisibleParticle(ParticleOptions, double…)`: shown now and then even with the
    /// fewest particles.
    #[allow(clippy::too_many_arguments)]
    pub fn add_always_visible_particle(
        &self,
        particle: impl Into<ParticleOptions>,
        x: f64,
        y: f64,
        z: f64,
        xd: f64,
        yd: f64,
        zd: f64,
    ) {
        self.do_add_particle(&particle.into(), false, true, x, y, z, xd, yd, zd);
    }

    /// `addAlwaysVisibleParticle(ParticleOptions, boolean, double…)`: shown however far if its
    /// type overrides the limiter or `override_limiter` does.
    #[allow(clippy::too_many_arguments)]
    pub fn add_always_visible_particle_limited(
        &self,
        particle: impl Into<ParticleOptions>,
        override_limiter: bool,
        x: f64,
        y: f64,
        z: f64,
        xd: f64,
        yd: f64,
        zd: f64,
    ) {
        let particle = particle.into();
        let override_limiter = particle.get_type().get_override_limiter() || override_limiter;
        self.do_add_particle(&particle, override_limiter, true, x, y, z, xd, yd, zd);
    }

    /// `doAddParticle`: to the particle engine, unless it's more than 32 blocks from the camera or
    /// the options' particles (with chance) leave it out, either of which overriding the limiter
    /// skips.
    #[allow(clippy::too_many_arguments)]
    fn do_add_particle(
        &self,
        particle: &ParticleOptions,
        override_limiter: bool,
        always_show_particles: bool,
        x: f64,
        y: f64,
        z: f64,
        xd: f64,
        yd: f64,
        zd: f64,
    ) {
        let particle_level = self.calculate_particle_level(always_show_particles);
        // Not past 32 blocks, as Java's negated test reads it (a NaN distance isn't).
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        let near = !(self.effects.camera_position.distance_to_sqr_xyz(x, y, z) > 1024.0);
        if override_limiter || (near && particle_level != ParticleStatus::MINIMAL) {
            self.effects
                .particle_engine
                .create_particle(particle, x, y, z, xd, yd, zd);
        }
    }

    /// `calculateParticleLevel`: the options' particles, one in ten of the fewest made fewer for
    /// a particle always shown, and one in three of fewer made the fewest.
    fn calculate_particle_level(&self, always_show_particles: bool) -> ParticleStatus {
        let mut particle_level = self.effects.particles;
        if always_show_particles
            && particle_level == ParticleStatus::MINIMAL
            && self.level.random.borrow_mut().next_int_bounded(10) == 0
        {
            particle_level = ParticleStatus::DECREASED;
        }

        if particle_level == ParticleStatus::DECREASED
            && self.level.random.borrow_mut().next_int_bounded(3) == 0
        {
            particle_level = ParticleStatus::MINIMAL;
        }

        particle_level
    }

    /// `playLocalSound(double, double, double, …)`: a sound at the position, seeded from the
    /// level's random.
    #[allow(clippy::too_many_arguments)]
    pub fn play_local_sound(
        &self,
        x: f64,
        y: f64,
        z: f64,
        sound: SoundEvent,
        source: SoundSource,
        volume: f32,
        pitch: f32,
        distance_delay: bool,
    ) {
        let seed = self.level.random.borrow_mut().next_long();
        self.play_sound(x, y, z, sound, source, volume, pitch, distance_delay, seed);
    }

    /// `Level.playLocalSound(BlockPos, …)`: at the block's middle.
    #[allow(clippy::too_many_arguments)]
    pub fn play_local_sound_at(
        &self,
        pos: &BlockPos,
        sound: SoundEvent,
        source: SoundSource,
        volume: f32,
        pitch: f32,
        distance_delay: bool,
    ) {
        self.play_local_sound(
            f64::from(pos.get_x()) + 0.5,
            f64::from(pos.get_y()) + 0.5,
            f64::from(pos.get_z()) + 0.5,
            sound,
            source,
            volume,
            pitch,
            distance_delay,
        );
    }

    /// `playPlayerSound`: a sound following the player, if there's one.
    pub fn play_player_sound(
        &self,
        sound: SoundEvent,
        source: SoundSource,
        volume: f32,
        pitch: f32,
    ) {
        if let Some(player) = self.effects.player {
            let seed = self.level.random.borrow_mut().next_long();
            self.effects
                .sound_manager
                .play(SoundInstance::EntityBound(EntityBoundSoundInstance {
                    sound,
                    source,
                    volume,
                    pitch,
                    entity: player,
                    seed,
                }));
        }
    }

    /// `playSound`: now, or after the time sound takes to come from more than 10 blocks away.
    #[allow(clippy::too_many_arguments)]
    fn play_sound(
        &self,
        x: f64,
        y: f64,
        z: f64,
        sound: SoundEvent,
        source: SoundSource,
        volume: f32,
        pitch: f32,
        distance_delay: bool,
        seed: i64,
    ) {
        let distance_to_sqr = self.effects.camera_position.distance_to_sqr_xyz(x, y, z);
        let instance = SoundInstance::Simple(SimpleSoundInstance::new(
            sound, source, volume, pitch, seed, x, y, z,
        ));
        if distance_delay && distance_to_sqr > 100.0 {
            let delay_in_seconds = distance_to_sqr.sqrt() / 40.0;
            self.effects
                .sound_manager
                .play_delayed(instance, (delay_in_seconds * 20.0) as i32);
        } else {
            self.effects.sound_manager.play(instance);
        }
    }

    /// `tickWeatherEffects`: while it rains, raindrops (smoke on lava, magma and lit campfires)
    /// on the blocks the rain falls on around the camera, a tick's from its own random seeded by
    /// the game time, and now and then the rain's sound where the last fell.
    pub fn tick_weather_effects(&self) {
        let particle_status = self.effects.particles;
        let weather_radius = self.effects.weather_radius;
        let rain_level = self.level.get_rain_level(1.0);
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(rain_level <= 0.0) {
            let mut random = random_source::create_thread_local_instance_with_seed(
                self.level.get_game_time().wrapping_mul(312987231),
            );
            let camera_position = BlockPos::containing(&self.effects.camera_position);
            let mut rain_particle_position = None;
            let weather_diameter = 2 * weather_radius + 1;
            let weather_area = weather_diameter * weather_diameter;
            let rain_particles = (0.225_f32 * weather_area as f32 * rain_level * rain_level) as i32
                / if particle_status == ParticleStatus::DECREASED {
                    2
                } else {
                    1
                };

            for _ in 0..rain_particles {
                let x = random.next_int_bounded(weather_diameter) - weather_radius;
                let z = random.next_int_bounded(weather_diameter) - weather_radius;
                let heightmap_position = self.level.get_heightmap_pos(
                    Types::MOTION_BLOCKING,
                    &camera_position.offset_xyz(x, 0, z),
                );
                if heightmap_position.get_y() > LevelHeightAccessor::get_min_y(self.level)
                    && heightmap_position.get_y() <= camera_position.get_y() + 10
                    && heightmap_position.get_y() >= camera_position.get_y() - 10
                    && self.level.get_precipitation_at(&heightmap_position) == Precipitation::RAIN
                {
                    let position = heightmap_position.below();
                    rain_particle_position = Some(position);
                    if particle_status == ParticleStatus::MINIMAL {
                        break;
                    }

                    let block_x = random.next_double();
                    let block_z = random.next_double();
                    let block = self.level.get_block_state(&position);
                    let fluid = self.level.get_fluid_state(&position);
                    let block_shape = block.get_collision_shape(self.level, &position);
                    let block_top = block_shape.max_at(Axis::Y, block_x, block_z);
                    let fluid_top = f64::from(fluid.get_height(self.level, &position));
                    let particle_y = Math::max_f64(block_top, fluid_top);
                    let particle_type = if !fluid.is_tag(FluidTags::LAVA)
                        && !block.is(Blocks::MAGMA_BLOCK)
                        && !CampfireBlock::is_lit_campfire(block)
                    {
                        ParticleTypes::RAIN
                    } else {
                        ParticleTypes::SMOKE
                    };
                    self.add_particle(
                        particle_type,
                        f64::from(position.get_x()) + block_x,
                        f64::from(position.get_y()) + particle_y,
                        f64::from(position.get_z()) + block_z,
                        0.0,
                        0.0,
                        0.0,
                    );
                }
            }

            if let Some(position) = rain_particle_position {
                let rain_sound_time = self.level.rain_sound_time.get();
                self.level.rain_sound_time.set(rain_sound_time + 1);
                if random.next_int_bounded(3) < rain_sound_time {
                    self.level.rain_sound_time.set(0);
                    if position.get_y() > camera_position.get_y() + 1
                        && self
                            .level
                            .get_heightmap_pos(Types::MOTION_BLOCKING, &camera_position)
                            .get_y()
                            > camera_position.get_y()
                    {
                        self.play_local_sound_at(
                            &position,
                            SoundEvents::WEATHER_RAIN_ABOVE,
                            SoundSource::WEATHER,
                            0.1,
                            0.5,
                            false,
                        );
                    } else {
                        self.play_local_sound_at(
                            &position,
                            SoundEvents::WEATHER_RAIN,
                            SoundSource::WEATHER,
                            0.2,
                            1.0,
                            false,
                        );
                    }
                }
            }
        }
    }

    /// `rainSoundTime`, which the golden programs read.
    pub fn rain_sound_time(&self) -> i32 {
        self.level.rain_sound_time.get()
    }

    /// `environmentAttributes().getValue(attribute, BlockPos)`: at the block's middle.
    pub fn get_environment_value<V: Value>(
        &self,
        attribute: &EnvironmentAttribute<V>,
        pos: &BlockPos,
    ) -> V {
        match self.effects.environment {
            Some((system, layer_level)) => {
                system.get_value(attribute, &Vec3::at_center_of(pos), None, layer_level)
            }
            None => {
                V::from_attribute_value(EnvironmentAttributes::info(attribute.id()).default_value)
            }
        }
    }

    /// `getClientLeafTintColor`: the block's first tint in the level, -1 for an untinted one.
    pub fn get_client_leaf_tint_color(&self, pos: &BlockPos) -> i32 {
        let state = self.level.get_block_state(pos);
        match self.effects.block_colors.get_tint_source(state, 0) {
            Some(tint_source) => tint_source.color_in_world(state, self.level, pos),
            None => -1,
        }
    }
}

impl ClientLevel {
    /// Panics on registries without the dimension type's value, or without biomes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        registry_access: &RegistryAccess,
        hardcore: bool,
        is_flat: bool,
        dimension: ResourceKey,
        dimension_type: Holder,
        server_chunk_radius: i32,
        server_simulation_distance: i32,
        is_debug: bool,
        biome_zoom_seed: i64,
        sea_level: i32,
    ) -> ClientLevel {
        let dimension_type_value = registry_access
            .lookup_or_throw(Registries::DIMENSION_TYPE)
            .value::<DimensionType>(&dimension_type)
            .clone();
        let height = dimension_type_value.level_height();
        let factory = PalettedContainerFactory::create(registry_access);
        let biomes = Arc::new(registry_access.lookup_or_throw(Registries::BIOME).clone());
        let has_sky_light = dimension_type_value.has_sky_light();
        let level_extractor = Rc::new(LevelExtractor::new());
        let block_entity_tickers = Rc::new(SharedLevelState::new());
        ClientLevel {
            dimension,
            dimension_type,
            dimension_type_value,
            height,
            chunk_source: ClientChunkCache::new(
                height,
                has_sky_light,
                factory,
                server_chunk_radius,
                Rc::clone(&level_extractor),
                std::sync::Arc::new(registry_access.clone()),
                Rc::clone(&block_entity_tickers),
            ),
            server_chunk_radius,
            server_simulation_distance,
            is_debug,
            biome_zoom_seed,
            sea_level,
            hardcore,
            is_flat,
            world_border: WorldBorder::new(),
            o_rain_level: 0.0,
            rain_level: 0.0,
            o_thunder_level: 0.0,
            thunder_level: 0.0,
            sky_flash_time: 0,
            rain_sound_time: Cell::new(0),
            sky_darken: 0,
            random: RefCell::new(LegacyRandomSource::new(
                RandomSupport::generate_unique_seed(),
            )),
            respawn_data: RespawnData::clone(RespawnData::DEFAULT),
            light_update_queue: VecDeque::new(),
            biomes,
            biome_manager: BiomeManager::new(biome_zoom_seed),
            tint_caches: [
                (&biome_colors::GRASS_COLOR_RESOLVER, BlockTintCache::new()),
                (&biome_colors::FOLIAGE_COLOR_RESOLVER, BlockTintCache::new()),
                (
                    &biome_colors::DRY_FOLIAGE_COLOR_RESOLVER,
                    BlockTintCache::new(),
                ),
                (&biome_colors::WATER_COLOR_RESOLVER, BlockTintCache::new()),
            ],
            biome_blend_radius: Options::DEFAULT_BIOME_BLEND_RADIUS,
            entity_storage: EntityLookup::new(),
            ticking_entities: EntityTickList::new(),
            level_extractor,
            block_entity_tickers,
            neighbor_updater: CollectingNeighborUpdater::new(1000000),
        }
    }

    /// `addEntity(Entity)`: in place of one it had by the id, ticking from now on.
    pub fn add_entity(&mut self, entity: Entity) {
        self.remove_entity(entity.get_id());
        self.ticking_entities.add(entity.get_id());
        self.entity_storage.add(entity);
    }

    /// `removeEntity(int, RemovalReason)`: gone from the storage and the tick list
    /// (`setRemoved`'s callbacks).
    pub fn remove_entity(&mut self, id: i32) {
        if let Some(entity) = self.entity_storage.get_entity(id) {
            let uuid = entity.get_uuid();
            self.ticking_entities.remove(id);
            self.entity_storage.remove(id, uuid);
        }
    }

    /// `getEntity(int)`, the player's aside.
    pub fn get_entity(&self, id: i32) -> Option<&Entity> {
        self.entity_storage.get_entity(id)
    }

    pub fn get_entity_mut(&mut self, id: i32) -> Option<&mut Entity> {
        self.entity_storage.get_entity_mut(id)
    }

    /// `entitiesForRendering()`: the entities in the order they were added, the player's aside.
    pub fn entities_for_rendering(&self) -> impl Iterator<Item = &Entity> {
        self.entity_storage.get_all_entities()
    }

    /// `tick(BooleanSupplier)`, as far as it's ported: the world border's move, the game time and
    /// the clocks, the rain's particles and sound, and the sky's flash.
    pub fn tick(&mut self, clock_manager: &mut ClientClockManager, effects: &LevelEffects<'_>) {
        // `tickRateManager().runsNormally()`: the tick rate isn't ported, so it always does.
        self.world_border.tick();
        self.tick_time(clock_manager);
        LevelWithEffects::new(self, effects).tick_weather_effects();
        if self.sky_flash_time > 0 {
            self.set_sky_flash_time(self.sky_flash_time - 1);
        }
    }

    /// `tickTime()`: a game tick more, which the clocks follow.
    pub(crate) fn tick_time(&mut self, clock_manager: &mut ClientClockManager) {
        let game_time = self.get_game_time().wrapping_add(1);
        self.block_entity_tickers.game_time.set(game_time);
        clock_manager.tick(game_time);
    }

    /// `Level.getRainLevel(float)`: between the last tick's rain and this one's.
    pub fn get_rain_level(&self, a: f32) -> f32 {
        Mth::lerp_f32(a, self.o_rain_level, self.rain_level)
    }

    /// `Level.setRainLevel`: both ticks' rain, clamped to 0 to 1.
    pub fn set_rain_level(&mut self, rain_level: f32) {
        let clamped_rain_level = Mth::clamp_f32(rain_level, 0.0, 1.0);
        self.o_rain_level = clamped_rain_level;
        self.rain_level = clamped_rain_level;
    }

    /// `Level.getThunderLevel(float)`: between the last tick's thunder and this one's, times the
    /// rain.
    pub fn get_thunder_level(&self, a: f32) -> f32 {
        Mth::lerp_f32(a, self.o_thunder_level, self.thunder_level) * self.get_rain_level(a)
    }

    /// `Level.setThunderLevel`: both ticks' thunder, clamped to 0 to 1.
    pub fn set_thunder_level(&mut self, thunder_level: f32) {
        let clamped_thunder_level = Mth::clamp_f32(thunder_level, 0.0, 1.0);
        self.o_thunder_level = clamped_thunder_level;
        self.thunder_level = clamped_thunder_level;
    }

    /// `Level.canHaveWeather()`: sky light, no ceiling, and not the End.
    pub fn can_have_weather(&self) -> bool {
        self.dimension_type_value.has_sky_light()
            && !self.dimension_type_value.has_ceiling()
            && *self.dimension.identifier() != Identifier::with_default_namespace("the_end")
    }

    /// `Level.isThundering()`.
    pub fn is_thundering(&self) -> bool {
        self.can_have_weather() && f64::from(self.get_thunder_level(1.0)) > 0.9
    }

    /// `Level.isRaining()`.
    pub fn is_raining(&self) -> bool {
        self.can_have_weather() && f64::from(self.get_rain_level(1.0)) > 0.2
    }

    /// `getSkyFlashTime()`: the ticks a lightning bolt's flash has left. Java reads it as 0 with
    /// the options' `hideLightningFlash`, which the options don't have yet.
    pub fn get_sky_flash_time(&self) -> i32 {
        self.sky_flash_time
    }

    pub fn set_sky_flash_time(&mut self, sky_flash_time: i32) {
        self.sky_flash_time = sky_flash_time;
    }

    /// `Level.getRandom()`, borrowed: let it go before anything else draws from it.
    pub fn get_random(&self) -> RefMut<'_, LegacyRandomSource> {
        self.random.borrow_mut()
    }

    /// `updateSkyBrightness()`, with the environment attributes' `SKY_LIGHT_LEVEL`, which the game
    /// renderer holds.
    pub fn update_sky_brightness(&mut self, sky_light_level: f32) {
        self.sky_darken = (15.0 - sky_light_level) as i32;
    }

    /// `Level.getSkyDarken()`.
    pub fn get_sky_darken(&self) -> i32 {
        self.sky_darken
    }

    /// `LevelReader.getMaxLocalRawBrightness(BlockPos)`: the block's light, the sky's darkened.
    pub fn get_max_local_raw_brightness(&self, pos: &BlockPos) -> i32 {
        if pos.get_x() >= -30000000
            && pos.get_z() >= -30000000
            && pos.get_x() < 30000000
            && pos.get_z() < 30000000
        {
            self.get_raw_brightness(pos, self.sky_darken)
        } else {
            15
        }
    }

    /// `Level.getHeight(Heightmap.Types, int, int)`: the first free block above the column's
    /// heightmap, the bottom where the level has no chunk (Java's empty chunk), and above the sea
    /// past the world's edge.
    pub fn get_height_at(&self, r#type: Types, x: i32, z: i32) -> i32 {
        if (-30000000..30000000).contains(&x) && (-30000000..30000000).contains(&z) {
            match self.chunk_source.get_chunk(
                SectionPos::block_to_section_coord(x),
                SectionPos::block_to_section_coord(z),
            ) {
                Some(chunk) => chunk
                    .get_heightmap(r#type)
                    .expect("a full chunk keeps its heightmaps")
                    .get_first_available(x & 15, z & 15),
                None => LevelHeightAccessor::get_min_y(self),
            }
        } else {
            self.sea_level + 1
        }
    }

    /// `LevelReader.getHeightmapPos`: the column's first free block above its heightmap.
    pub fn get_heightmap_pos(&self, r#type: Types, pos: &BlockPos) -> BlockPos {
        BlockPos::new(
            pos.get_x(),
            self.get_height_at(r#type, pos.get_x(), pos.get_z()),
            pos.get_z(),
        )
    }

    /// `getPrecipitationAt`: none where the level has no chunk, the biome's otherwise.
    pub fn get_precipitation_at(&self, pos: &BlockPos) -> Precipitation {
        if self
            .chunk_source
            .get_chunk(
                SectionPos::block_to_section_coord(pos.get_x()),
                SectionPos::block_to_section_coord(pos.get_z()),
            )
            .is_none()
        {
            return Precipitation::NONE;
        }

        self.biome_value(&self.get_biome(pos))
            .get_precipitation_at(pos, self.sea_level)
    }

    /// `Level.isRainingAt`: raining, open to the sky with nothing above that stops rain, in a
    /// biome where it rains there.
    pub fn is_raining_at(&self, pos: &BlockPos) -> bool {
        if !self.is_raining() {
            return false;
        }
        if !self.can_see_sky(pos) {
            return false;
        }
        if self.get_heightmap_pos(Types::MOTION_BLOCKING, pos).get_y() > pos.get_y() {
            return false;
        }
        self.biome_value(&self.get_biome(pos))
            .get_precipitation_at(pos, self.sea_level)
            == Precipitation::RAIN
    }

    /// `animateTick(int, int, int)`, each tick around the player: 667 times a block within 16 of
    /// the position, then one within 32, each animated (`doAnimateTick`). The animations' random
    /// source is a new one each tick (`RandomSource.createThreadLocalInstance()`). Java's marker
    /// particles show for a barrier or light block held in creative (`getMarkerParticleTarget`),
    /// which the caller decides, items not being ported.
    pub fn animate_tick(
        &self,
        xt: i32,
        yt: i32,
        zt: i32,
        marker_particle_target: Option<Block>,
        effects: &LevelEffects<'_>,
    ) {
        let mut animate_random = random_source::create_thread_local_instance();
        let mut pos = MutableBlockPos::new();

        for _ in 0..667 {
            self.do_animate_tick(
                xt,
                yt,
                zt,
                16,
                animate_random.as_mut(),
                marker_particle_target,
                &mut pos,
                effects,
            );
            self.do_animate_tick(
                xt,
                yt,
                zt,
                32,
                animate_random.as_mut(),
                marker_particle_target,
                &mut pos,
                effects,
            );
        }
    }

    /// `doAnimateTick`: a block within `r` of the position, picked with the level's random, its
    /// block's and fluid's `animateTick`, a fluid's drip below it, and the marker particle. The
    /// biomes' ambient particles (`AMBIENT_PARTICLES`, the nether's spores and ash) aren't
    /// ported: their attribute's type isn't, so a nether block's draws from the level's random
    /// are missing.
    #[allow(clippy::too_many_arguments)]
    pub fn do_animate_tick(
        &self,
        xt: i32,
        yt: i32,
        zt: i32,
        r: i32,
        animate_random: &mut dyn RandomSource,
        marker_particle_target: Option<Block>,
        pos: &mut MutableBlockPos,
        effects: &LevelEffects<'_>,
    ) {
        let (x, y, z) = {
            let mut random = self.random.borrow_mut();
            let x = xt
                .wrapping_add(random.next_int_bounded(r))
                .wrapping_sub(random.next_int_bounded(r));
            let y = yt
                .wrapping_add(random.next_int_bounded(r))
                .wrapping_sub(random.next_int_bounded(r));
            let z = zt
                .wrapping_add(random.next_int_bounded(r))
                .wrapping_sub(random.next_int_bounded(r));
            (x, y, z)
        };
        pos.set_xyz(x, y, z);
        let block_pos = pos.immutable();
        let level = LevelWithEffects::new(self, effects);
        let state = self.get_block_state(&block_pos);
        state
            .get_block()
            .animate_tick(state, &level, &block_pos, animate_random);
        let fluid_state = self.get_fluid_state(&block_pos);
        if !fluid_state.is_empty() {
            fluid_state.animate_tick(&level, &block_pos, animate_random);
            let drip_particle = fluid_state.get_drip_particle();
            if let Some(drip_particle) = drip_particle
                && self.random.borrow_mut().next_int_bounded(10) == 0
            {
                let has_watertight_bottom = state.is_face_sturdy(self, &block_pos, Direction::DOWN);
                let below = block_pos.below();
                self.try_spawn_drip_particles(
                    &level,
                    &below,
                    self.get_block_state(&below),
                    drip_particle,
                    has_watertight_bottom,
                );
            }
        }

        if marker_particle_target == Some(state.get_block()) {
            level.add_particle(
                BlockParticleOption::new(ParticleTypes::BLOCK_MARKER, state),
                f64::from(x) + 0.5,
                f64::from(y) + 0.5,
                f64::from(z) + 0.5,
                0.0,
                0.0,
                0.0,
            );
        }
    }

    fn try_spawn_drip_particles(
        &self,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        state: BlockState,
        drip_particle: ParticleOptions,
        is_top_solid: bool,
    ) {
        if state.get_fluid_state().is_empty() {
            let collision_shape = state.get_collision_shape(self, pos);
            let top_side_height = collision_shape.max(Axis::Y);
            if top_side_height < 1.0 {
                if is_top_solid {
                    self.spawn_fluid_particle(
                        level,
                        f64::from(pos.get_x()),
                        f64::from(pos.get_x() + 1),
                        f64::from(pos.get_z()),
                        f64::from(pos.get_z() + 1),
                        f64::from(pos.get_y() + 1) - 0.05,
                        drip_particle,
                    );
                }
            } else if !state.is_tag(BlockTags::IMPERMEABLE) {
                let bottom_side_height = collision_shape.min(Axis::Y);
                if bottom_side_height > 0.0 {
                    self.spawn_particle(
                        level,
                        pos,
                        drip_particle,
                        &collision_shape,
                        f64::from(pos.get_y()) + bottom_side_height - 0.05,
                    );
                } else {
                    let below = pos.below();
                    let below_state = self.get_block_state(&below);
                    let below_shape = below_state.get_collision_shape(self, &below);
                    let below_top_side_height = below_shape.max(Axis::Y);
                    if below_top_side_height < 1.0 && below_state.get_fluid_state().is_empty() {
                        self.spawn_particle(
                            level,
                            pos,
                            drip_particle,
                            &collision_shape,
                            f64::from(pos.get_y()) - 0.05,
                        );
                    }
                }
            }
        }
    }

    fn spawn_particle(
        &self,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        drip_particle: ParticleOptions,
        drip_shape: &VoxelShape,
        height: f64,
    ) {
        self.spawn_fluid_particle(
            level,
            f64::from(pos.get_x()) + drip_shape.min(Axis::X),
            f64::from(pos.get_x()) + drip_shape.max(Axis::X),
            f64::from(pos.get_z()) + drip_shape.min(Axis::Z),
            f64::from(pos.get_z()) + drip_shape.max(Axis::Z),
            height,
            drip_particle,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_fluid_particle(
        &self,
        level: &LevelWithEffects<'_>,
        x1: f64,
        x2: f64,
        z1: f64,
        z2: f64,
        y: f64,
        drip_particle: ParticleOptions,
    ) {
        let x = Mth::lerp_f64(self.random.borrow_mut().next_double(), x1, x2);
        let z = Mth::lerp_f64(self.random.borrow_mut().next_double(), z1, z2);
        level.add_particle(drip_particle, x, y, z, 0.0, 0.0, 0.0);
    }

    /// `Level.tickBlockEntities()`: the pending tickers added to the end, then each ticker in
    /// order, a removed one dropped from the list. Those added while they tick wait for the next
    /// tick. Java's `tickRateManager().runsNormally()` and `shouldTickBlocksAt` are always true
    /// here: the tick rate isn't ported, and a client ticks blocks everywhere.
    pub fn tick_block_entities(
        &mut self,
        effects: &LevelEffects<'_>,
        player: Option<&mut dyn LevelPlayer>,
    ) {
        let tickers = Rc::clone(&self.block_entity_tickers);
        tickers.ticking_block_entities.set(true);
        let mut block_entity_tickers =
            std::mem::take(&mut *tickers.block_entity_tickers.borrow_mut());
        block_entity_tickers.append(&mut tickers.pending_block_entity_tickers.borrow_mut());

        let mut level = LevelTicking {
            level: self,
            effects,
            player: player.map(|player| player as &mut dyn LevelPlayer),
        };
        block_entity_tickers.retain(|ticker| {
            if ticker.is_removed(level.level) {
                return false;
            }
            ticker.tick(&mut level);
            true
        });

        *tickers.block_entity_tickers.borrow_mut() = block_entity_tickers;
        tickers.ticking_block_entities.set(false);
    }

    /// The tickers the level ticks, in order: `blockEntityTickers`.
    pub fn block_entity_tickers(&self) -> &SharedLevelState {
        &self.block_entity_tickers
    }

    /// `Level.blockEvent(pos, block, b0, b1)`: the state there's `triggerEvent` (the block the
    /// server named isn't read).
    pub fn block_event(level: &mut LevelTicking<'_>, pos: &BlockPos, b0: i32, b1: i32) {
        let state = level.level.get_block_state(pos);
        state.trigger_event(level, pos, b0, b1);
    }

    /// `tickEntities()` for the entities besides the player, in the tick list's order:
    /// `tickNonPassenger`'s `commonTick`, which steps their interpolation; their classes' `tick`
    /// isn't ported.
    pub fn tick_entities(&mut self) {
        for id in self.ticking_entities.for_each() {
            self.with_entity(id, |entity, level| entity.common_tick_in(level));
        }
    }

    /// Runs `f` on the entity with the level, which it's out of meanwhile, and answers what `f`
    /// does, or none without the entity. Java's entity reaches its level while it's in it.
    pub fn with_entity<R>(
        &mut self,
        id: i32,
        f: impl FnOnce(&mut Entity, &ClientLevel) -> R,
    ) -> Option<R> {
        let (index, mut entity) = self.entity_storage.take(id)?;
        let result = f(&mut entity, self);
        self.entity_storage.put_back(index, entity);
        Some(result)
    }

    /// `isTickingEntity(Entity)`: every entity the port has ticks.
    pub fn is_ticking_entity(&self, id: i32) -> bool {
        self.ticking_entities.contains(id)
    }

    /// The options' `biomeBlendRadius`, which the tints blend over.
    pub fn set_biome_blend_radius(&mut self, biome_blend_radius: i32) {
        self.biome_blend_radius = biome_blend_radius;
    }

    /// `getBiome(pos)`: the biome the level's `BiomeManager` picks for the block.
    pub fn get_biome(&self, pos: &BlockPos) -> Holder {
        self.biome_manager.get_biome(self, pos)
    }

    /// `Holder.value()` of one of the level's biomes.
    pub fn biome_value(&self, biome: &Holder) -> &Biome {
        self.biomes.value::<Biome>(biome)
    }

    /// `getUncachedNoiseBiome`: plains, where the level has no chunk.
    pub fn get_uncached_noise_biome(&self, _quart_x: i32, _quart_y: i32, _quart_z: i32) -> Holder {
        self.biomes.get_or_throw(Biomes::PLAINS)
    }

    /// `onChunkLoaded`: the tints around the chunk are worked out again. Java also starts its
    /// entities ticking, which aren't ported.
    pub fn on_chunk_loaded(&self, pos: ChunkPos) {
        for (_, cache) in &self.tint_caches {
            cache.invalidate_for_chunk(pos.x(), pos.z());
        }
    }

    pub fn clear_tint_caches(&self) {
        for (_, cache) in &self.tint_caches {
            cache.invalidate_all();
        }
    }

    /// The resolver's colour at the block, averaged over the blend radius's square of columns
    /// around it at its height.
    pub fn calculate_block_tint(&self, pos: &BlockPos, color_resolver: &ColorResolver) -> i32 {
        ClientLevel::calculate_block_tint_in(
            self,
            &self.biome_manager,
            &self.biomes,
            self.biome_blend_radius,
            pos,
            color_resolver,
        )
    }

    /// `calculateBlockTint` over the noise biomes `biome_source` hands out, which are the level's
    /// or a copy of some of them (`RenderSectionRegion`'s), zoomed by `biome_manager`.
    pub fn calculate_block_tint_in(
        biome_source: &dyn BiomeResolver,
        biome_manager: &BiomeManager,
        biomes: &Registry,
        dist: i32,
        pos: &BlockPos,
        color_resolver: &ColorResolver,
    ) -> i32 {
        if dist == 0 {
            return color_resolver.get_color(
                biomes.value::<Biome>(&biome_manager.get_biome(biome_source, pos)),
                f64::from(pos.get_x()),
                f64::from(pos.get_z()),
            );
        }

        let count = (dist * 2 + 1) * (dist * 2 + 1);
        let mut total_red = 0;
        let mut total_green = 0;
        let mut total_blue = 0;
        let mut cursor = Cursor3D::new(
            pos.get_x() - dist,
            pos.get_y(),
            pos.get_z() - dist,
            pos.get_x() + dist,
            pos.get_y(),
            pos.get_z() + dist,
        );

        while cursor.advance() {
            let next_pos = BlockPos::new(cursor.next_x(), cursor.next_y(), cursor.next_z());
            let color = color_resolver.get_color(
                biomes.value::<Biome>(&biome_manager.get_biome(biome_source, &next_pos)),
                f64::from(next_pos.get_x()),
                f64::from(next_pos.get_z()),
            );
            total_red += ARGB::red(color);
            total_green += ARGB::green(color);
            total_blue += ARGB::blue(color);
        }

        ARGB::color_rgb(total_red / count, total_green / count, total_blue / count)
    }

    /// What a section compile's copy of the level reads to work out tints the way the level does:
    /// the biome registry, the zoom, the blend radius, and the tint caches by resolver.
    pub fn biome_registry(&self) -> &Arc<Registry> {
        &self.biomes
    }

    pub fn biome_manager(&self) -> BiomeManager {
        self.biome_manager
    }

    pub fn biome_blend_radius(&self) -> i32 {
        self.biome_blend_radius
    }

    pub fn tint_caches(&self) -> &[(&'static ColorResolver, BlockTintCache); 4] {
        &self.tint_caches
    }

    /// The tints a section compile worked out over a copy of the caches, put in them
    /// (`BlockTintCache::give_back`).
    pub fn give_back_tints(&self, tints: TintsToGiveBack) {
        for ((_, cache), (generation, layers)) in self.tint_caches.iter().zip(tints.layers) {
            cache.give_back(
                tints.chunk_x,
                tints.chunk_z,
                tints.min_y,
                generation,
                layers,
            );
        }
    }

    const NORMAL_LIGHT_UPDATES_PER_FRAME: usize = 10;
    const LIGHT_UPDATE_QUEUE_SIZE_THRESHOLD: usize = 1000;

    pub fn queue_light_update(&mut self, update: LightUpdate) {
        self.light_update_queue.push_back(update);
    }

    /// Runs a tenth of the queued light updates (at least 10), or all of them once there are a
    /// thousand.
    pub fn poll_light_updates(&mut self) {
        let size = self.light_update_queue.len();
        let light_updates_per_frame = if size < ClientLevel::LIGHT_UPDATE_QUEUE_SIZE_THRESHOLD {
            ClientLevel::NORMAL_LIGHT_UPDATES_PER_FRAME.max(size / 10)
        } else {
            size
        };

        for _ in 0..light_updates_per_frame {
            let Some(update) = self.light_update_queue.pop_front() else {
                break;
            };

            update(self);
        }
    }

    /// Whether light updates are queued, which `update` runs some of a frame.
    pub fn has_light_updates(&self) -> bool {
        !self.light_update_queue.is_empty()
    }

    /// How many light updates are queued.
    #[cfg(test)]
    pub(crate) fn light_updates_queued(&self) -> usize {
        self.light_update_queue.len()
    }

    /// `update()`, once a frame: the queued light updates, then the light engine's own.
    pub fn update(&mut self) {
        self.poll_light_updates();
        let (light_engine, chunks) = self.chunk_source.get_light_engine_mut();
        light_engine.run_light_updates(chunks);
    }

    /// `getLightEngine()`, to change: the chunk source's, with the chunks it lights.
    pub fn get_light_engine_mut(&mut self) -> (&mut LevelLightEngine, &dyn LightChunkGetter) {
        self.chunk_source.get_light_engine_mut()
    }

    /// `getSectionYFromSectionIndex`, as `LevelHeightAccessor`'s.
    pub fn get_section_y_from_section_index(&self, section_index: i32) -> i32 {
        LevelHeightAccessor::get_section_y_from_section_index(self, section_index)
    }

    pub fn get_world_border_mut(&mut self) -> &mut WorldBorder {
        &mut self.world_border
    }

    pub fn dimension(&self) -> &ResourceKey {
        &self.dimension
    }

    pub fn dimension_type_registration(&self) -> &Holder {
        &self.dimension_type
    }

    /// `dimensionType()`: the registration's value.
    pub fn dimension_type(&self) -> &DimensionType {
        &self.dimension_type_value
    }

    pub fn server_chunk_radius(&self) -> i32 {
        self.server_chunk_radius
    }

    pub fn get_biome_zoom_seed(&self) -> i64 {
        self.biome_zoom_seed
    }

    /// `getLevelData().isHardcore()`.
    pub fn is_hardcore(&self) -> bool {
        self.hardcore
    }

    /// `getLevelData().isFlat()`.
    pub fn is_flat(&self) -> bool {
        self.is_flat
    }

    pub fn get_sea_level(&self) -> i32 {
        self.sea_level
    }

    pub fn is_debug(&self) -> bool {
        self.is_debug
    }

    /// `getLevelData().getGameTime()`.
    pub fn get_game_time(&self) -> i64 {
        self.block_entity_tickers.get_game_time()
    }

    pub fn set_time_from_server(&mut self, game_time: i64) {
        self.block_entity_tickers.game_time.set(game_time);
    }

    pub fn get_respawn_data(&self) -> &RespawnData {
        &self.respawn_data
    }

    pub fn set_respawn_data(&mut self, respawn_data: RespawnData) {
        self.respawn_data = respawn_data;
    }

    pub fn set_server_simulation_distance(&mut self, server_simulation_distance: i32) {
        self.server_simulation_distance = server_simulation_distance;
    }

    pub fn get_chunk_source(&self) -> &ClientChunkCache {
        &self.chunk_source
    }

    pub fn get_chunk_source_mut(&mut self) -> &mut ClientChunkCache {
        &mut self.chunk_source
    }

    /// `Level.getBlockEntity(pos)`: the chunk's block entity there, made if the block makes one
    /// (`IMMEDIATE`); none out of bounds or where the level has no chunk.
    pub fn get_block_entity(&mut self, pos: &BlockPos) -> Option<&mut BlockEntity> {
        if !self.is_in_valid_bounds(pos) {
            return None;
        }
        self.chunk_source
            .get_chunk_mut(
                SectionPos::block_to_section_coord(pos.get_x()),
                SectionPos::block_to_section_coord(pos.get_z()),
            )?
            .get_block_entity_immediate(pos)
    }

    /// `BlockGetter.getBlockEntity(pos, type)`: the block entity there, if it's of the type.
    pub fn get_block_entity_typed(
        &mut self,
        pos: &BlockPos,
        r#type: BlockEntityType,
    ) -> Option<&mut BlockEntity> {
        self.get_block_entity(pos)
            .filter(|block_entity| block_entity.get_type() == r#type)
    }

    /// The block entity there if the chunk has one, without making it (`CHECK`), which what
    /// only reads the level (the shapes, the renderer) asks.
    pub fn get_existing_block_entity(&self, pos: &BlockPos) -> Option<Ref<'_, BlockEntity>> {
        self.chunk_source
            .get_chunk(
                SectionPos::block_to_section_coord(pos.get_x()),
                SectionPos::block_to_section_coord(pos.get_z()),
            )?
            .get_block_entity(pos)
    }

    /// The block entity there to change, without making it: what a ticker changes while it
    /// reads the level.
    pub fn get_block_entity_mut(&self, pos: &BlockPos) -> Option<RefMut<'_, BlockEntity>> {
        self.chunk_source
            .get_chunk(
                SectionPos::block_to_section_coord(pos.get_x()),
                SectionPos::block_to_section_coord(pos.get_z()),
            )?
            .get_block_entity_mut(pos)
    }

    /// `Level.removeBlockEntity(pos)`.
    pub fn remove_block_entity(&mut self, pos: &BlockPos) {
        if !self.is_in_valid_bounds(pos) {
            return;
        }
        if let Some(chunk) = self.chunk_source.get_chunk_mut(
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_z()),
        ) {
            chunk.remove_block_entity(pos);
        }
    }

    /// `Level.setBlockEntity(blockEntity)`: its chunk's (`addAndRegisterBlockEntity`).
    pub fn set_block_entity(&mut self, block_entity: BlockEntity) {
        let pos = block_entity.get_block_pos();
        if !self.is_in_valid_bounds(&pos) {
            return;
        }
        if let Some(chunk) = self.chunk_source.get_chunk_mut(
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_z()),
        ) {
            chunk.add_and_register_block_entity(block_entity);
        }
    }

    /// `Level.isInValidBounds`: in the build height, and in a chunk that's valid.
    pub fn is_in_valid_bounds(&self, pos: &BlockPos) -> bool {
        self.is_inside_build_height(pos.get_y())
            && ChunkPos::is_valid(
                SectionPos::block_to_section_coord(pos.get_x()),
                SectionPos::block_to_section_coord(pos.get_z()),
            )
    }

    /// A block the server says is there. Java checks the player's own predictions first, which
    /// aren't ported, so it's always `Level.setBlock`.
    pub fn set_server_verified_block_state(
        &mut self,
        pos: &BlockPos,
        block_state: BlockState,
        update_flag: i32,
    ) {
        self.set_block(pos, block_state, update_flag, 512);
    }

    /// `Level.setBlock`: whether the chunk took the state, and the renderer told of the change
    /// (`setBlocksDirty`, and `sendBlockUpdated` with `Block.UPDATE_CLIENTS` but not
    /// `UPDATE_INVISIBLE`), and unless the shape is known (`UPDATE_KNOWN_SHAPE`) the neighbours'
    /// shapes updated. `UPDATE_NEIGHBORS`' `updateNeighborsAt` and `updatePOIOnBlockStateChange`
    /// do nothing on the client.
    pub fn set_block(
        &mut self,
        pos: &BlockPos,
        block_state: BlockState,
        update_flags: i32,
        update_limit: i32,
    ) -> bool {
        if !self.is_in_valid_bounds(pos) {
            return false;
        }

        let chunk_x = SectionPos::block_to_section_coord(pos.get_x());
        let chunk_z = SectionPos::block_to_section_coord(pos.get_z());
        // Java's level hands out its empty chunk for one it doesn't have, which takes no blocks.
        let old_state = match self
            .chunk_source
            .get_chunk_and_light_engine_mut(chunk_x, chunk_z)
        {
            Some((chunk, light_engine)) => chunk.set_block_state(pos, block_state, light_engine),
            None => None,
        };
        let Some(old_state) = old_state else {
            return false;
        };

        let new_state = self.get_block_state(pos);
        if new_state == block_state {
            if old_state != new_state {
                self.set_blocks_dirty(pos, old_state, new_state);
            }

            if (update_flags & 2) != 0 && (update_flags & 4) == 0 {
                self.send_block_updated(pos, old_state, block_state, update_flags);
            }

            if (update_flags & 16) == 0 && update_limit > 0 {
                let neighbour_update_flags = update_flags & -34;
                old_state.update_indirect_neighbour_shapes(
                    self,
                    pos,
                    neighbour_update_flags,
                    update_limit - 1,
                );
                block_state.update_neighbour_shapes(
                    self,
                    pos,
                    neighbour_update_flags,
                    update_limit - 1,
                );
                block_state.update_indirect_neighbour_shapes(
                    self,
                    pos,
                    neighbour_update_flags,
                    update_limit - 1,
                );
            }
        }
        true
    }

    /// `Level.neighborUpdater`.
    pub fn neighbor_updater(&self) -> &CollectingNeighborUpdater {
        &self.neighbor_updater
    }

    /// `neighborShapeChanged(direction, pos, neighborPos, neighborState, updateFlags,
    /// updateLimit)`: the block at `pos` told its neighbour that way is now `neighbor_state`,
    /// through the neighbour updater.
    pub fn neighbor_shape_changed(
        &mut self,
        direction: Direction,
        pos: &BlockPos,
        neighbor_pos: &BlockPos,
        neighbor_state: BlockState,
        update_flags: i32,
        update_limit: i32,
    ) {
        CollectingNeighborUpdater::shape_update(
            self,
            direction,
            neighbor_state,
            pos,
            neighbor_pos,
            update_flags,
            update_limit,
        );
    }

    /// `Level.removeBlock(pos, movedByPiston)`: the block there replaced by its fluid alone, with
    /// `UPDATE_ALL` (and `UPDATE_MOVE_BY_PISTON`).
    pub fn remove_block(&mut self, pos: &BlockPos, moved_by_piston: bool) -> bool {
        let fluid_state = self.get_fluid_state(pos);
        self.set_block(
            pos,
            fluid_state.create_legacy_block(),
            3 | if moved_by_piston { 64 } else { 0 },
            512,
        )
    }

    /// `sendBlockUpdated`: the sections around the block are dirty.
    pub fn send_block_updated(
        &self,
        pos: &BlockPos,
        _old: BlockState,
        _current: BlockState,
        update_flags: i32,
    ) {
        self.level_extractor.block_changed(pos, update_flags);
    }

    /// `setBlocksDirty`: the sections around the block are dirty when its look changed.
    pub fn set_blocks_dirty(&self, pos: &BlockPos, old_state: BlockState, new_state: BlockState) {
        self.level_extractor
            .set_block_dirty_states(pos, old_state, new_state);
    }

    pub fn set_section_dirty_with_neighbors(&self, chunk_x: i32, chunk_y: i32, chunk_z: i32) {
        self.level_extractor
            .set_section_dirty_with_neighbors(chunk_x, chunk_y, chunk_z);
    }

    pub fn set_section_range_dirty(
        &self,
        min_section_x: i32,
        min_section_y: i32,
        min_section_z: i32,
        max_section_x: i32,
        max_section_y: i32,
        max_section_z: i32,
    ) {
        self.level_extractor.set_section_range_dirty(
            min_section_x,
            min_section_y,
            min_section_z,
            max_section_x,
            max_section_y,
            max_section_z,
        );
    }

    /// `Minecraft.levelExtractor`'s dirty sections, which the level renderer takes each frame.
    pub fn level_extractor(&self) -> &Rc<LevelExtractor> {
        &self.level_extractor
    }
}

impl LevelHeightAccessor for ClientLevel {
    fn get_height(&self) -> i32 {
        self.height.get_height()
    }

    fn get_min_y(&self) -> i32 {
        self.height.get_min_y()
    }
}

/// `Level.getBlockState` and `getFluidState`: void air (and no fluid) out of the level's bounds
/// and in a chunk it doesn't have (Java's `EmptyLevelChunk`), the chunk's otherwise.
impl BlockGetter for ClientLevel {
    fn get_block_entity(&self, pos: &BlockPos) -> Option<Ref<'_, BlockEntity>> {
        self.get_existing_block_entity(pos)
    }

    #[inline]
    fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        if !self.is_in_valid_bounds(pos) {
            return Blocks::VOID_AIR.default_block_state();
        }

        match self.chunk_source.get_chunk(
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_z()),
        ) {
            Some(chunk) => chunk.get_block_state(pos),
            None => Blocks::VOID_AIR.default_block_state(),
        }
    }

    fn get_fluid_state(&self, pos: &BlockPos) -> FluidState {
        if !self.is_in_valid_bounds(pos) {
            return FluidState::EMPTY;
        }

        match self.chunk_source.get_chunk(
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_z()),
        ) {
            Some(chunk) => chunk.get_fluid_state(pos),
            None => FluidState::EMPTY,
        }
    }
}

impl BlockAndLightGetter for ClientLevel {
    /// `Level.getLightEngine()`: the chunk source's.
    fn get_light_engine(&self) -> &LevelLightEngine {
        self.chunk_source.get_light_engine()
    }
}

impl BlockAndTintGetter for ClientLevel {
    /// `dimensionType().cardinalLightType().get()`.
    fn cardinal_lighting(&self) -> CardinalLighting {
        self.dimension_type_value.cardinal_light_type().get()
    }

    /// The resolver's cached tint. Panics for a resolver that isn't one of `BiomeColors`', where
    /// Java's map has no cache and throws `NullPointerException`.
    fn get_block_tint(&self, pos: &BlockPos, resolver: &'static ColorResolver) -> i32 {
        let (_, cache) = self
            .tint_caches
            .iter()
            .find(|(cached, _)| ColorResolver::same(cached, resolver))
            .unwrap_or_else(|| panic!("no tint cache for the resolver {}", resolver.name()));
        cache.get_color(pos, |pos| self.calculate_block_tint(pos, resolver))
    }
}

/// `LevelReader.getNoiseBiome`: the chunk's biome at the quart, or the uncached one where the
/// level has no chunk.
impl BiomeResolver for ClientLevel {
    fn get_noise_biome(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Holder {
        match self
            .chunk_source
            .get_chunk(QuartPos::to_section(quart_x), QuartPos::to_section(quart_z))
        {
            Some(chunk) => chunk.get_noise_biome(quart_x, quart_y, quart_z),
            None => self.get_uncached_noise_biome(quart_x, quart_y, quart_z),
        }
    }
}

/// What collisions ask: the chunk's blocks, the border, and the entities' shapes, of which there
/// are none yet.
impl CollisionGetter for ClientLevel {
    fn get_world_border(&self) -> &WorldBorder {
        &self.world_border
    }

    /// `getChunkForCollisions`: `getChunk(x, z, FULL, false)`.
    fn get_chunk_for_collisions(&self, chunk_x: i32, chunk_z: i32) -> Option<&dyn BlockGetter> {
        self.chunk_source
            .get_chunk(chunk_x, chunk_z)
            .map(|chunk| chunk as &dyn BlockGetter)
    }

    fn get_entity_collisions(
        &self,
        _source: Option<&dyn EntityClass>,
        _test_area: &AABB,
    ) -> Vec<VoxelShape> {
        Vec::new()
    }
}

impl Level for ClientLevel {
    fn is_client_side(&self) -> bool {
        true
    }

    fn get_min_y(&self) -> i32 {
        LevelHeightAccessor::get_min_y(self)
    }

    /// `hasChunkAt(pos)`: whether the chunk source has the block's chunk.
    fn has_chunk_at(&self, pos: &BlockPos) -> bool {
        self.chunk_source
            .get_chunk(
                SectionPos::block_to_section_coord(pos.get_x()),
                SectionPos::block_to_section_coord(pos.get_z()),
            )
            .is_some()
    }

    /// `EntityFluidInteraction`'s scan of one chunk: whether any of its sections from `section_y0`
    /// to `section_y1` that the level has holds a fluid.
    fn chunk_has_fluid(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        section_y0: i32,
        section_y1: i32,
    ) -> Option<bool> {
        let chunk = self.chunk_source.get_chunk(chunk_x, chunk_z)?;
        let sections = chunk.get_sections();
        let mut has_fluid = false;
        for section_y in section_y0..=section_y1 {
            let section_index = chunk.get_section_index_from_section_y(section_y);
            if section_index >= 0 && (section_index as usize) < sections.len() {
                has_fluid |= sections[section_index as usize].has_fluid();
            }
        }
        Some(has_fluid)
    }

    /// `environmentAttributes().getDimensionValue(FAST_LAVA)`: the dimension type's.
    fn has_fast_lava(&self) -> bool {
        self.dimension_type_value.fast_lava()
    }

    fn get_game_time(&self) -> i64 {
        ClientLevel::get_game_time(self)
    }
}

impl fmt::Debug for ClientLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientLevel")
            .field("dimension", &self.dimension)
            .field("dimension_type", &self.dimension_type)
            .field("chunks", &self.chunk_source.get_loaded_chunks_count())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use serde_json::Value;

    use super::{ClientLevel, LevelEffects};
    use crate::net::minecraft::client::color::block::block_colors::BlockColors;
    use crate::net::minecraft::client::particle::particle_engine::ParticleEngine;
    use crate::net::minecraft::client::renderer::chunk::render_region_cache::LevelEnvironment;
    use crate::net::minecraft::client::sounds::sound_manager::{SoundInstance, SoundManager};
    use crate::net::minecraft::core::block_pos::MutableBlockPos;
    use crate::net::minecraft::core::particles::particle_options::ParticleOptions;
    use crate::net::minecraft::server::level::particle_status::ParticleStatus;
    use crate::net::minecraft::util::random_source::RandomSource;
    use crate::net::minecraft::world::attribute::environment_attribute_system::EnvironmentAttributeSystem;
    use crate::net::minecraft::world::level::biome::biome_manager::BiomeManager;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;
    use crate::net::minecraft::world::level::levelgen::single_threaded_random_source::SingleThreadedRandomSource;
    use crate::net::minecraft::world::level::lighting::light_event_listener::LightEventListener;
    use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
    use crate::net::minecraft::world::phys::vec3::Vec3;
    use crate::golden::{Decode, Encode, Golden, unhex};
    use crate::net::minecraft::client::client_clock_manager::ClientClockManager;
    use crate::net::minecraft::core::registry_access::{self, RegistryAccess};
    use crate::net::minecraft::network::protocol::game::clientbound_set_time_packet::ClockUpdates;
    use crate::net::minecraft::world::clock::clock_network_state::ClockNetworkState;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::client::multiplayer::client_packet_listener::ClientPacketListener;
    use crate::net::minecraft::client::renderer::biome_colors;
    use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::game::clientbound_chunks_biomes_packet::ClientboundChunksBiomesPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::registry_reader;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::world::level::biome::biome::tests::biome_registries;
    use crate::net::minecraft::world::level::color_map_color_util::tests::load_colormaps;
    use crate::net::minecraft::world::level::color_resolver::ColorResolver;

    const HEIGHTS: [i32; 3] = [-61, 0, 70];

    /// The walk's chunks, biomes laid on them by a `chunks_biomes` packet, and every resolver's
    /// tint at each column (at three heights) as vanilla's `calculateBlockTint` blends them: from
    /// the level's tint caches, so read twice (the second time cached), and the second layout
    /// laid on the first's level, whose tints the packet's chunks must forget. Then the level's
    /// ticks (`ticks_match_vanilla`).
    #[test]
    fn matches_vanilla() {
        load_colormaps();
        let chunk_golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket",
        );
        let registries = biome_registries(&chunk_golden.input("BIOMES"));
        let mut packets = Vec::new();
        for (args, _) in chunk_golden.cases("captured") {
            let mut input = registry_reader(unhex(args[0].as_str().expect("hex")));
            packets.push(ClientboundLevelChunkWithLightPacket::STREAM_CODEC.decode(&mut input));
        }
        let resolvers: [&'static ColorResolver; 4] = [
            &biome_colors::GRASS_COLOR_RESOLVER,
            &biome_colors::FOLIAGE_COLOR_RESOLVER,
            &biome_colors::DRY_FOLIAGE_COLOR_RESOLVER,
            &biome_colors::WATER_COLOR_RESOLVER,
        ];

        let golden = Golden::load("net.minecraft.client.multiplayer.ClientLevel");
        let layouts = golden.cases("layouts");
        assert_eq!(layouts.len(), 4);
        let mut level = None;
        let mut level_radius = -1;
        for (number, (args, expected)) in layouts.iter().enumerate() {
            let radius = args[0].as_i64().expect("a radius") as i32;
            if radius != level_radius {
                let mut new_level = ClientLevel::new(
                    &registries,
                    false,
                    true,
                    ResourceKey::create(
                        Registries::DIMENSION,
                        Identifier::with_default_namespace("overworld"),
                    ),
                    Holder::Reference(ResourceKey::create(
                        Registries::DIMENSION_TYPE,
                        Identifier::with_default_namespace("overworld"),
                    )),
                    8,
                    8,
                    false,
                    i64::decode(&golden.input("ZOOM_SEED")),
                    -63,
                );
                new_level.set_biome_blend_radius(radius);
                for packet in &packets {
                    ClientPacketListener::handle_level_chunk_with_light_in(&mut new_level, packet);
                }
                level = Some(new_level);
                level_radius = radius;
            }
            let level = level.as_mut().expect("made above");
            let mut buffer = unpooled::wrapped_buffer(unhex(args[1].as_str().expect("hex")));
            let biomes = ClientboundChunksBiomesPacket::STREAM_CODEC.decode(&mut buffer);
            ClientPacketListener::handle_chunks_biomes_in(level, &biomes);

            let expected = expected.as_array().expect("tints");
            for pass in 0..2 {
                let mut index = 0;
                for column in args[2].as_array().expect("columns") {
                    let x = column[0].as_i64().expect("x") as i32;
                    let z = column[1].as_i64().expect("z") as i32;
                    for y in HEIGHTS {
                        for resolver in resolvers {
                            let pos = BlockPos::new(x, y, z);
                            assert_eq!(
                                i64::from(level.get_block_tint(&pos, resolver)),
                                expected[index].as_i64().expect("a tint"),
                                "layout {number} (pass {pass}): {} at {pos:?}",
                                resolver.name()
                            );
                            index += 1;
                        }
                    }
                }
                assert_eq!(index, expected.len());
            }
        }
        ticks_match_vanilla(&golden, &registries);
        animate_tick_matches_vanilla(&golden, &registries, &packets);
        weather_matches_vanilla(&golden);
        golden.finish();
    }

    /// What `animateTick` hands the particle engine and the sound manager, as the golden writes it.
    #[derive(Default)]
    struct Recorder {
        events: RefCell<Vec<Value>>,
    }

    impl ParticleEngine for Recorder {
        fn create_particle(
            &self,
            options: &ParticleOptions,
            x: f64,
            y: f64,
            z: f64,
            xd: f64,
            yd: f64,
            zd: f64,
        ) {
            let fields = match options {
                ParticleOptions::Simple(_) => Value::Null,
                ParticleOptions::Block(block) => Value::from(block.get_state().id()),
                ParticleOptions::Color(color) => Value::from(color.color()),
                ParticleOptions::Dust(dust) => {
                    Value::from(vec![Value::from(dust.color()), dust.get_scale().encode()])
                }
                ParticleOptions::DustColorTransition(dust) => {
                    let (from, to) = dust.colors();
                    Value::from(vec![
                        Value::from(from),
                        Value::from(to),
                        dust.get_scale().encode(),
                    ])
                }
            };
            self.events.borrow_mut().push(Value::from(vec![
                Value::from("particle"),
                Value::from(options.get_type().key()),
                fields,
                x.encode(),
                y.encode(),
                z.encode(),
                xd.encode(),
                yd.encode(),
                zd.encode(),
            ]));
        }
    }

    impl Recorder {
        fn sound(&self, sound: SoundInstance, delay: i32) {
            let event = match sound {
                SoundInstance::Simple(sound) => Value::from(vec![
                    Value::from("sound"),
                    Value::from(sound.sound.location().to_string()),
                    Value::from(sound.source.get_name()),
                    sound.volume.encode(),
                    sound.pitch.encode(),
                    sound.x.encode(),
                    sound.y.encode(),
                    sound.z.encode(),
                    Value::from(LegacyRandomSource::new(sound.seed).next_long()),
                    Value::from(delay),
                ]),
                SoundInstance::EntityBound(sound) => Value::from(vec![
                    Value::from("entitySound"),
                    Value::from(sound.sound.location().to_string()),
                    Value::from(sound.source.get_name()),
                    sound.volume.encode(),
                    sound.pitch.encode(),
                    Value::from(sound.entity),
                    Value::from(LegacyRandomSource::new(sound.seed).next_long()),
                    Value::from(delay),
                ]),
            };
            self.events.borrow_mut().push(event);
        }
    }

    impl SoundManager for Recorder {
        fn play(&self, sound: SoundInstance) {
            self.sound(sound, -1);
        }

        fn play_delayed(&self, sound: SoundInstance, delay: i32) {
            self.sound(sound, delay);
        }
    }

    /// Runs the queued light updates and the light engine's until neither has anything left.
    fn settle(level: &mut ClientLevel) {
        while level.has_light_updates() || level.get_light_engine().has_light_work() {
            level.update();
        }
    }

    /// `animateTick`: the walk's chunks with the course of animated blocks set on them, and each
    /// program's 667 pairs of `doAnimateTick` from the seeded randoms, near its camera, with its
    /// options' particles, its sky's darkening, time and weather: every particle and sound the
    /// blocks and fluids make, in order, and the randoms' next longs after.
    fn animate_tick_matches_vanilla(
        golden: &Golden,
        registries: &RegistryAccess,
        packets: &[ClientboundLevelChunkWithLightPacket],
    ) {
        let mut level = level_in(registries, "overworld");
        level.biome_zoom_seed = i64::decode(&golden.input("ZOOM_SEED"));
        level.biome_manager = BiomeManager::new(level.biome_zoom_seed);
        for packet in packets {
            ClientPacketListener::handle_level_chunk_with_light_in(&mut level, packet);
        }
        settle(&mut level);
        for placement in golden.input("COURSE").as_array().expect("the course") {
            let placement = Vec::<i32>::decode(placement);
            let pos = BlockPos::new(placement[0], placement[1], placement[2]);
            level.set_server_verified_block_state(&pos, Block::state_by_id(placement[3]), 19);
        }
        settle(&mut level);

        let biomes = registries.lookup_or_throw(Registries::BIOME);
        let biome_keys: Vec<ResourceKey> = (0..biomes.size())
            .map(|id| {
                biomes
                    .holder_by_id_or_throw(id)
                    .unwrap_key()
                    .expect("a registry's holder has its key")
                    .clone()
            })
            .collect();
        let system = EnvironmentAttributeSystem::for_client_level(
            &Identifier::with_default_namespace("overworld"),
            &biome_keys,
            true,
        )
        .expect("the overworld's attributes");
        let synchronized = registry_access::tests::golden();
        let clock_registry = synchronized.lookup_or_throw(Registries::WORLD_CLOCK);
        let block_colors = BlockColors::create_default();

        for (number, (args, expected)) in golden.cases("programs").into_iter().enumerate() {
            let level_seed = i64::decode(&args[0]);
            let animate_seed = i64::decode(&args[1]);
            let particles = match args[2].as_str().expect("a status") {
                "ALL" => ParticleStatus::ALL,
                "DECREASED" => ParticleStatus::DECREASED,
                _ => ParticleStatus::MINIMAL,
            };
            let camera = Vec::<f64>::decode(&args[3]);
            let center = Vec::<i32>::decode(&args[4]);
            let clock_time = i64::decode(&args[7]);
            *level.random.borrow_mut() = LegacyRandomSource::new(level_seed);
            level.sky_darken = i32::decode(&args[5]);
            level.set_time_from_server(i64::decode(&args[6]));
            level.set_rain_level(f32::decode(&args[8]));
            level.set_thunder_level(f32::decode(&args[9]));
            // AttributeSuites.Clock: every clock at the program's time.
            let mut clocks = ClientClockManager::new();
            let mut updates = ClockUpdates::new();
            for id in 0..clock_registry.size() {
                updates.put(
                    clock_registry.holder_by_id_or_throw(id),
                    ClockNetworkState::new(clock_time, 0.0, 1.0),
                );
            }
            clocks.handle_updates(0, &updates);
            system.invalidate_tick_cache();
            let layer_level = LevelEnvironment::new(&level, clocks);
            let recorder = Recorder::default();
            let effects = LevelEffects {
                particle_engine: &recorder,
                sound_manager: &recorder,
                camera_position: Vec3::new(camera[0], camera[1], camera[2]),
                particles,
                weather_radius: 10,
                player: Some(7),
                environment: Some((&system, &layer_level)),
                block_colors: &block_colors,
            };
            let mut animate_random = SingleThreadedRandomSource::new(animate_seed);
            let mut pos = MutableBlockPos::new();
            for _ in 0..667 {
                for r in [16, 32] {
                    level.do_animate_tick(
                        center[0],
                        center[1],
                        center[2],
                        r,
                        &mut animate_random,
                        None,
                        &mut pos,
                        &effects,
                    );
                }
            }
            let mut events = recorder.events.take();
            events.push(Value::from(vec![
                Value::from("end"),
                Value::from(level.random.borrow_mut().next_long()),
                Value::from(animate_random.next_long()),
            ]));
            let expected = expected.as_array().expect("events");
            let first_difference = events
                .iter()
                .zip(expected)
                .position(|(actual, expected)| actual != expected);
            assert!(
                first_difference.is_none() && events.len() == expected.len(),
                "program {number}: {} events, vanilla's {}; the first differing (#{first_difference:?}): {:?} vs vanilla's {:?}",
                events.len(),
                expected.len(),
                first_difference.map(|index| &events[index]),
                first_difference.map(|index| &expected[index]),
            );
        }
    }

    fn level_in(registries: &RegistryAccess, dimension: &str) -> ClientLevel {
        ClientLevel::new(
            registries,
            false,
            false,
            ResourceKey::create(
                Registries::DIMENSION,
                Identifier::with_default_namespace(dimension),
            ),
            Holder::Reference(ResourceKey::create(
                Registries::DIMENSION_TYPE,
                Identifier::with_default_namespace(dimension),
            )),
            8,
            8,
            false,
            0,
            63,
        )
    }

    /// `ticks`: the game time a tick at a time (`tickTime`) and as the server sets it, the clocks,
    /// and the weather the server's game events set, in each dimension.
    fn ticks_match_vanilla(golden: &Golden, registries: &RegistryAccess) {
        // The clocks of the synchronized registries, in id order.
        let synchronized = registry_access::tests::golden();
        let clock_registry = synchronized.lookup_or_throw(Registries::WORLD_CLOCK);
        let clocks: Vec<Holder> = (0..clock_registry.size())
            .map(|id| clock_registry.holder_by_id_or_throw(id))
            .collect();
        golden.check("ticks", |dimension: String, ops: Value| {
            let mut level = level_in(registries, &dimension);
            let mut clock_manager = ClientClockManager::new();
            let outputs = ops
                .as_array()
                .expect("ops")
                .iter()
                .map(|op| {
                    let op = op.as_array().expect("an op is an array");
                    match op[0].as_str().expect("an op's name") {
                        "tickTime" => level.tick_time(&mut clock_manager),
                        "handleSetTime" => {
                            let game_time = i64::decode(&op[1]);
                            let mut updates = ClockUpdates::new();
                            for (clock, state) in Vec::<(Holder, ClockNetworkState)>::decode(&op[2])
                            {
                                updates.put(clock, state);
                            }
                            level.set_time_from_server(game_time);
                            clock_manager.handle_updates(game_time, &updates);
                        }
                        "setRainLevel" => level.set_rain_level(f32::decode(&op[1])),
                        "setThunderLevel" => level.set_thunder_level(f32::decode(&op[1])),
                        "weather" => {
                            let partial = f32::decode(&op[1]);
                            return Value::from(vec![
                                level.get_rain_level(partial).encode(),
                                level.get_thunder_level(partial).encode(),
                                level.is_raining().encode(),
                                level.is_thundering().encode(),
                            ]);
                        }
                        "time" => {
                            let mut state = vec![level.get_game_time().encode()];
                            for clock in &clocks {
                                let instance = clock_manager.get_instance(clock);
                                state.push(Value::from(vec![
                                    instance.total_ticks().encode(),
                                    instance.partial_tick().encode(),
                                    instance.rate().encode(),
                                ]));
                            }
                            return Value::from(state);
                        }
                        name => panic!("unknown op {name}"),
                    }
                    Value::Null
                })
                .collect::<Vec<_>>();
            Value::from(outputs)
        });
    }

    /// `tickWeatherEffects` over 24 ticks of rain at levels and cameras around a course on the
    /// walk's chunks: the raindrops, smoke and sounds, `rainSoundTime` after each tick, and the
    /// level's random, as vanilla's.
    fn weather_matches_vanilla(golden: &Golden) {
        use crate::net::minecraft::world::level::block::entity::ticking_block_entity::tests::{
            Recorder, walk_chunks, walk_level,
        };

        let (registries, packets) = walk_chunks();
        let block_colors = BlockColors::create_default();
        let programs = golden.cases("weather");
        assert_eq!(programs.len(), 64);
        for (number, (args, expected)) in programs.into_iter().enumerate() {
            let mut level = walk_level(&registries, &packets);
            *level.random.borrow_mut() = LegacyRandomSource::new(i64::decode(&args[0]));
            let particles = match args[1].as_str().expect("a status") {
                "ALL" => ParticleStatus::ALL,
                "DECREASED" => ParticleStatus::DECREASED,
                _ => ParticleStatus::MINIMAL,
            };
            let weather_radius = i32::decode(&args[2]);
            let game_time = i64::decode(&args[3]);
            for placement in args[4].as_array().expect("the course") {
                let placement = Vec::<i32>::decode(placement);
                level.set_block(
                    &BlockPos::new(placement[0], placement[1], placement[2]),
                    Block::state_by_id(placement[3]),
                    18,
                    512,
                );
            }

            let recorder = Recorder::default();
            let mut lines = Vec::new();
            for (tick, step) in args[5].as_array().expect("ticks").iter().enumerate() {
                let rain = f32::decode(&step[0]);
                let camera = Vec::<f64>::decode(&step[1]);
                level.set_time_from_server(game_time + tick as i64);
                level.set_rain_level(rain);
                let effects = LevelEffects {
                    particle_engine: &recorder,
                    sound_manager: &recorder,
                    camera_position: Vec3::new(camera[0], camera[1], camera[2]),
                    particles,
                    weather_radius,
                    player: Some(7),
                    environment: None,
                    block_colors: &block_colors,
                };
                let level_with_effects = super::LevelWithEffects::new(&level, &effects);
                level_with_effects.tick_weather_effects();
                lines.push(Value::from(vec![
                    Value::from(recorder.events.take()),
                    Value::from(level_with_effects.rain_sound_time()),
                ]));
            }
            lines.push(Value::from(vec![
                Value::from("end"),
                Value::from(level.random.borrow_mut().next_long()),
            ]));
            let expected = expected.as_array().expect("lines");
            assert_eq!(lines.len(), expected.len(), "weather {number}");
            for (tick, (actual, expected)) in lines.iter().zip(expected).enumerate() {
                assert_eq!(actual, expected, "weather {number}, tick {tick}");
            }
        }
    }
}
