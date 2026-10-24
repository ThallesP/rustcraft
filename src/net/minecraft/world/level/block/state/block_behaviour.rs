//! net/minecraft/world/level/block/state/BlockBehaviour.java
//!
//! What decides a block's states and what they keep from its `Properties`: `Properties` itself,
//! the fields `BlockBehaviour` copies from it, and what `BlockStateBase` keeps of each state and
//! answers without a level (`getLightEmission`, `isAir`, `canOcclude`, `getPistonPushReaction`,
//! `getDestroySpeed`, `getMapColor`, `emissiveRendering`, `isRandomlyTicking`,
//! `getPostProcessPos`…), with its predicates (`isRedstoneConductor`, `isSuffocating`,
//! `isViewBlocking`).
//!
//! The shapes and what's worked out of them, each a `BlockBehaviour` method a class may override
//! (`BlockClass`'s fields), dispatched down the class chain like Java's: `getShape`,
//! `getCollisionShape`, `getVisualShape`, `getBlockSupportShape`, `getInteractionShape`,
//! `getOcclusionShape`, `isCollisionShapeFullBlock`, `useShapeForLightOcclusion`,
//! `propagatesSkylightDown`, `getLightDampening`, and what the renderer reads,
//! `getShadeBrightness`, `getRenderShape`, `skipRendering`, `getSeed` and the offset function
//! with each class's `getMaxHorizontalOffset` and `getMaxVerticalOffset`. `initCache` keeps what
//! it does of them (the collision shape's `Cache` with `isFaceSturdy`, `isSolid`, the occlusion
//! shape and its faces, `isSolidRender`, the light), worked out the first time it's asked for
//! (`StateShapes`). Every class's overrides are ported.
//!
//! Not ported yet: every other behaviour method (`useWithoutItem`, `tick`, `updateShape`,
//! `animateTick`…), which come with the blocks' classes; `Properties.sound` and `SoundType`,
//! `isValidSpawn` (it takes an `EntityType`) and `requiredFeatures` (`FeatureFlag`), which
//! Blocks' registrations leave out until they come; `getEntityInsideCollisionShape`, which
//! comes with what's inside a block.
//!
//! Java's state predicates are lambdas, so `Properties` holds them as values that say which they
//! are: Properties' defaults, which read the block's shapes or tags, Blocks' named ones, or a
//! function of the state alone. A function a level is passed to (`getPostProcessPos`,
//! `isRedstoneConductor`…) leaves the level out where none of vanilla's read it.

use std::sync::{Arc, OnceLock};

use crate::net::minecraft::world::level::material::fluid_state::FluidState;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::util::Util;
use crate::net::minecraft::world::item::dye_color::DyeColor;
use crate::net::minecraft::world::level::block::block::{Block, BlockClass, BlockData};
use crate::net::minecraft::world::level::block::blocks::{self, REGISTRY};
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::note_block_instrument::NoteBlockInstrument;
use crate::net::minecraft::world::level::block::support_type::SupportType;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::map_color::MapColor;
use crate::net::minecraft::world::level::material::push_reaction::PushReaction;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::collision_context::{self, CollisionContext};
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// `BlockBehaviour.StatePredicate`, as `Properties` holds one for `isRedstoneConductor` and
/// `isSuffocating`.
#[derive(Clone, Copy)]
pub enum StatePredicate {
    /// What `Properties` starts with for `isRedstoneConductor`: `isCollisionShapeFullBlock`.
    Default,
    /// What `Properties` starts with for `isSuffocating`: that and the `causes_suffocation` tag.
    DefaultSuffocating,
    /// `Blocks.NOT_CLOSED_SHULKER`, which asks the shulker box's block entity.
    NotClosedShulker,
    /// One that answers from the state alone, with its name in `Blocks` if it has one
    /// (`NOT_EXTENDED_PISTON`): `Blocks::never`, `Blocks::always`.
    State(Option<&'static str>, fn(BlockState) -> bool),
}

/// `BlockBehaviour.StateArgumentPredicate<AABB>`, as `Properties` holds one for `isViewBlocking`.
#[derive(Clone, Copy)]
pub enum ViewBlocking {
    /// What `Properties` starts with, `isSuffocating` of the `Properties` it was made for: its own
    /// (`None`), or the one it was copied from with `ofFullCopy`, whose `isSuffocating` it keeps
    /// (`Some`).
    Default(Option<StatePredicate>),
    /// `Blocks.NEAR_PLANE_INTERSECTS_OUTLINE`, which reads the state's occlusion shape.
    NearPlaneIntersectsOutline,
    /// One that answers from the state alone.
    State(fn(BlockState) -> bool),
}

/// `BlockBehaviour.OffsetType`: how a state's model is moved from the middle of its block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OffsetType {
    NONE,
    XZ,
    XYZ,
}

/// A function of the state, `ToIntFunction<BlockState>` or `Function<BlockState, MapColor>`.
pub type StateFunction<T> = Arc<dyn Fn(BlockState) -> T + Send + Sync>;

/// `DependantName<Block, T>`: worked out from the block's id, or fixed.
#[derive(Clone)]
enum DependantName<T> {
    FromId,
    Fixed(T),
}

/// `BlockBehaviour.Properties`: what a block is made with. Its setters return it, to chain like
/// Java's.
#[derive(Clone)]
pub struct Properties {
    map_color: StateFunction<MapColor>,
    has_collision: bool,
    light_emission: StateFunction<i32>,
    explosion_resistance: f32,
    destroy_time: f32,
    requires_correct_tool_for_drops: bool,
    is_randomly_ticking: bool,
    friction: f32,
    speed_factor: f32,
    jump_factor: f32,
    bounce_restitution: f32,
    id: Option<ResourceKey>,
    drops: DependantName<Option<ResourceKey>>,
    description_id: DependantName<String>,
    can_occlude: bool,
    is_air: bool,
    ignited_by_lava: bool,
    liquid: bool,
    force_solid_off: bool,
    force_solid_on: bool,
    push_reaction: PushReaction,
    spawn_terrain_particles: bool,
    instrument: NoteBlockInstrument,
    replaceable: bool,
    is_redstone_conductor: StatePredicate,
    is_suffocating: StatePredicate,
    is_view_blocking: ViewBlocking,
    post_process: fn(BlockState, BlockPos) -> Option<BlockPos>,
    emissive_rendering: fn(BlockState) -> bool,
    dynamic_shape: bool,
    offset_type: OffsetType,
    fall_distance_reduction: f32,
}

impl Properties {
    pub fn of() -> Properties {
        Properties {
            map_color: Arc::new(|_| MapColor::NONE),
            has_collision: true,
            light_emission: Arc::new(|_| 0),
            explosion_resistance: 0.0,
            destroy_time: 0.0,
            requires_correct_tool_for_drops: false,
            is_randomly_ticking: false,
            friction: 0.6,
            speed_factor: 1.0,
            jump_factor: 1.0,
            bounce_restitution: 0.0,
            id: None,
            drops: DependantName::FromId,
            description_id: DependantName::FromId,
            can_occlude: true,
            is_air: false,
            ignited_by_lava: false,
            liquid: false,
            force_solid_off: false,
            force_solid_on: false,
            push_reaction: PushReaction::PUSH_PULL,
            spawn_terrain_particles: true,
            instrument: NoteBlockInstrument::HARP,
            replaceable: false,
            is_redstone_conductor: StatePredicate::Default,
            is_suffocating: StatePredicate::DefaultSuffocating,
            is_view_blocking: ViewBlocking::Default(None),
            post_process: |_, _| None,
            emissive_rendering: |_| false,
            dynamic_shape: false,
            offset_type: OffsetType::NONE,
            fall_distance_reduction: 0.0,
        }
    }

    /// `ofFullCopy`: `ofLegacyCopy`, and the jump factor, the predicates, the post-processing,
    /// the loot table and the description id too.
    pub fn of_full_copy(block: &BlockBehaviour) -> Properties {
        let mut copy_to = Properties::of_legacy_copy(block);
        let copy_from = &block.properties;
        copy_to.jump_factor = copy_from.jump_factor;
        copy_to.is_redstone_conductor = copy_from.is_redstone_conductor;
        copy_to.post_process = copy_from.post_process;
        copy_to.is_suffocating = copy_from.is_suffocating;
        // A default isViewBlocking asks the Properties it was made for, which the copy keeps
        // pointing at.
        copy_to.is_view_blocking = match copy_from.is_view_blocking {
            ViewBlocking::Default(None) => ViewBlocking::Default(Some(copy_from.is_suffocating)),
            other => other,
        };
        copy_to.drops = copy_from.drops.clone();
        copy_to.description_id = copy_from.description_id.clone();
        copy_to
    }

    /// `ofLegacyCopy`: what a block was made with, but its loot table, description id, jump
    /// factor, predicates and post-processing.
    pub fn of_legacy_copy(block: &BlockBehaviour) -> Properties {
        let mut copy_to = Properties::of();
        let copy_from = &block.properties;
        copy_to.destroy_time = copy_from.destroy_time;
        copy_to.explosion_resistance = copy_from.explosion_resistance;
        copy_to.has_collision = copy_from.has_collision;
        copy_to.is_randomly_ticking = copy_from.is_randomly_ticking;
        copy_to.light_emission = copy_from.light_emission.clone();
        copy_to.map_color = copy_from.map_color.clone();
        copy_to.friction = copy_from.friction;
        copy_to.speed_factor = copy_from.speed_factor;
        copy_to.bounce_restitution = copy_from.bounce_restitution;
        copy_to.dynamic_shape = copy_from.dynamic_shape;
        copy_to.can_occlude = copy_from.can_occlude;
        copy_to.is_air = copy_from.is_air;
        copy_to.ignited_by_lava = copy_from.ignited_by_lava;
        copy_to.liquid = copy_from.liquid;
        copy_to.force_solid_off = copy_from.force_solid_off;
        copy_to.force_solid_on = copy_from.force_solid_on;
        copy_to.push_reaction = copy_from.push_reaction;
        copy_to.requires_correct_tool_for_drops = copy_from.requires_correct_tool_for_drops;
        copy_to.offset_type = copy_from.offset_type;
        copy_to.spawn_terrain_particles = copy_from.spawn_terrain_particles;
        copy_to.emissive_rendering = copy_from.emissive_rendering;
        copy_to.instrument = copy_from.instrument;
        copy_to.replaceable = copy_from.replaceable;
        copy_to
    }

    /// `mapColor(DyeColor)`.
    pub fn map_color_dye(self, dye_color: DyeColor) -> Properties {
        self.map_color(dye_color.get_map_color())
    }

    pub fn map_color(mut self, map_color: MapColor) -> Properties {
        self.map_color = Arc::new(move |_| map_color);
        self
    }

    /// `mapColor(Function<BlockState, MapColor>)`.
    pub fn map_color_fn(
        mut self,
        map_color: impl Fn(BlockState) -> MapColor + Send + Sync + 'static,
    ) -> Properties {
        self.map_color = Arc::new(map_color);
        self
    }

    pub fn no_collision(mut self) -> Properties {
        self.has_collision = false;
        self.can_occlude = false;
        self
    }

    pub fn no_occlusion(mut self) -> Properties {
        self.can_occlude = false;
        self
    }

    pub fn friction(mut self, friction: f32) -> Properties {
        self.friction = friction;
        self
    }

    pub fn speed_factor(mut self, speed_factor: f32) -> Properties {
        self.speed_factor = speed_factor;
        self
    }

    pub fn jump_factor(mut self, jump_factor: f32) -> Properties {
        self.jump_factor = jump_factor;
        self
    }

    pub fn bounce_restitution(mut self, bounce_restitution: f32) -> Properties {
        self.bounce_restitution = bounce_restitution;
        self
    }

    pub fn fall_distance_reduction(mut self, fall_distance_reduction: f32) -> Properties {
        self.fall_distance_reduction = fall_distance_reduction;
        self
    }

    pub fn light_level(
        mut self,
        light_emission: impl Fn(BlockState) -> i32 + Send + Sync + 'static,
    ) -> Properties {
        self.light_emission = Arc::new(light_emission);
        self
    }

    /// `strength(float, float)`.
    pub fn strength_and_resistance(
        self,
        destroy_time: f32,
        explosion_resistance: f32,
    ) -> Properties {
        self.destroy_time(destroy_time)
            .explosion_resistance(explosion_resistance)
    }

    pub fn instabreak(self) -> Properties {
        self.strength(0.0)
    }

    /// `strength(float)`: as hard to break as to blow up.
    pub fn strength(self, destroy_time: f32) -> Properties {
        self.strength_and_resistance(destroy_time, destroy_time)
    }

    pub fn random_ticks(mut self) -> Properties {
        self.is_randomly_ticking = true;
        self
    }

    pub fn dynamic_shape(mut self) -> Properties {
        self.dynamic_shape = true;
        self
    }

    pub fn no_loot_table(mut self) -> Properties {
        self.drops = DependantName::Fixed(None);
        self
    }

    pub fn override_loot_table(mut self, table: Option<ResourceKey>) -> Properties {
        self.drops = DependantName::Fixed(table);
        self
    }

    /// The loot table: `blocks/<path>` in the block's namespace, unless it was overridden.
    /// Panics without an id, where Java throws `NullPointerException`.
    pub(crate) fn effective_drops(&self) -> Option<ResourceKey> {
        match &self.drops {
            DependantName::Fixed(table) => table.clone(),
            DependantName::FromId => Some(ResourceKey::create(
                Registries::LOOT_TABLE,
                self.id().identifier().with_prefix("blocks/"),
            )),
        }
    }

    pub fn ignited_by_lava(mut self) -> Properties {
        self.ignited_by_lava = true;
        self
    }

    pub fn liquid(mut self) -> Properties {
        self.liquid = true;
        self
    }

    pub fn force_solid_on(mut self) -> Properties {
        self.force_solid_on = true;
        self
    }

    pub fn force_solid_off(mut self) -> Properties {
        self.force_solid_off = true;
        self
    }

    pub fn push_reaction(mut self, push_reaction: PushReaction) -> Properties {
        self.push_reaction = push_reaction;
        self
    }

    pub fn air(mut self) -> Properties {
        self.is_air = true;
        self
    }

    pub fn is_redstone_conductor(mut self, is_redstone_conductor: StatePredicate) -> Properties {
        self.is_redstone_conductor = is_redstone_conductor;
        self
    }

    pub fn is_suffocating(mut self, is_suffocating: StatePredicate) -> Properties {
        self.is_suffocating = is_suffocating;
        self
    }

    pub fn is_view_blocking(mut self, is_view_blocking: ViewBlocking) -> Properties {
        self.is_view_blocking = is_view_blocking;
        self
    }

    pub fn post_process(
        mut self,
        post_process: fn(BlockState, BlockPos) -> Option<BlockPos>,
    ) -> Properties {
        self.post_process = post_process;
        self
    }

    pub fn emissive_rendering(mut self, emissive_rendering: fn(BlockState) -> bool) -> Properties {
        self.emissive_rendering = emissive_rendering;
        self
    }

    pub fn requires_correct_tool_for_drops(mut self) -> Properties {
        self.requires_correct_tool_for_drops = true;
        self
    }

    pub fn destroy_time(mut self, destroy_time: f32) -> Properties {
        self.destroy_time = destroy_time;
        self
    }

    /// Never below 0: `Math.max(0.0F, explosionResistance)`.
    pub fn explosion_resistance(mut self, explosion_resistance: f32) -> Properties {
        self.explosion_resistance =
            crate::java::lang::math::Math::max_f32(0.0, explosion_resistance);
        self
    }

    /// `offsetType`, which picks the offset function (`BlockState::get_offset`).
    pub fn offset_type(mut self, offset_type: OffsetType) -> Properties {
        self.offset_type = offset_type;
        self
    }

    pub fn no_terrain_particles(mut self) -> Properties {
        self.spawn_terrain_particles = false;
        self
    }

    pub fn instrument(mut self, instrument: NoteBlockInstrument) -> Properties {
        self.instrument = instrument;
        self
    }

    pub fn replaceable(mut self) -> Properties {
        self.replaceable = true;
        self
    }

    pub fn set_id(mut self, id: ResourceKey) -> Properties {
        self.id = Some(id);
        self
    }

    pub fn override_description(mut self, description_id: String) -> Properties {
        self.description_id = DependantName::Fixed(description_id);
        self
    }

    /// `block.<namespace>.<path>`, unless it was overridden. Panics without an id.
    pub(crate) fn effective_description_id(&self) -> String {
        match &self.description_id {
            DependantName::Fixed(description_id) => description_id.clone(),
            DependantName::FromId => {
                Util::make_description_id("block", Some(self.id().identifier()))
            }
        }
    }

    fn id(&self) -> &ResourceKey {
        self.id
            .as_ref()
            .expect("NullPointerException: Block id not set")
    }

    /// `mapColor`, the function of the state.
    pub fn map_color_of(&self) -> &StateFunction<MapColor> {
        &self.map_color
    }

    pub fn get_is_redstone_conductor(&self) -> StatePredicate {
        self.is_redstone_conductor
    }

    pub fn get_is_suffocating(&self) -> StatePredicate {
        self.is_suffocating
    }

    pub fn get_is_view_blocking(&self) -> ViewBlocking {
        self.is_view_blocking
    }

    pub fn get_offset_type(&self) -> OffsetType {
        self.offset_type
    }

    pub fn get_force_solid_on(&self) -> bool {
        self.force_solid_on
    }

    pub fn get_force_solid_off(&self) -> bool {
        self.force_solid_off
    }
}

/// What `BlockBehaviour` keeps of the block's `Properties`, and the `Properties` themselves.
#[derive(Clone)]
pub struct BlockBehaviour {
    pub(crate) has_collision: bool,
    pub(crate) explosion_resistance: f32,
    pub(crate) is_randomly_ticking: bool,
    pub(crate) friction: f32,
    pub(crate) speed_factor: f32,
    pub(crate) jump_factor: f32,
    pub(crate) bounce_restitution: f32,
    pub(crate) fall_distance_reduction: f32,
    pub(crate) dynamic_shape: bool,
    pub(crate) properties: Properties,
    pub(crate) drops: Option<ResourceKey>,
    pub(crate) description_id: String,
}

impl BlockBehaviour {
    pub fn new(properties: Properties) -> BlockBehaviour {
        BlockBehaviour {
            has_collision: properties.has_collision,
            drops: properties.effective_drops(),
            description_id: properties.effective_description_id(),
            explosion_resistance: properties.explosion_resistance,
            is_randomly_ticking: properties.is_randomly_ticking,
            friction: properties.friction,
            speed_factor: properties.speed_factor,
            jump_factor: properties.jump_factor,
            bounce_restitution: properties.bounce_restitution,
            fall_distance_reduction: properties.fall_distance_reduction,
            dynamic_shape: properties.dynamic_shape,
            properties,
        }
    }

    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    /// The protected `hasCollision`, which a block's collision shape is empty without.
    pub fn has_collision(&self) -> bool {
        self.has_collision
    }

    pub fn get_loot_table(&self) -> Option<&ResourceKey> {
        self.drops.as_ref()
    }

    pub fn get_description_id(&self) -> &str {
        &self.description_id
    }

    pub fn default_destroy_time(&self) -> f32 {
        self.properties.destroy_time
    }
}

/// What `BlockStateBase` keeps of a state: from its block's `Properties` when it's made, and from
/// its block when the registry's cache is made (`initCache`).
pub struct BlockStateBase {
    light_emission: i32,
    is_air: bool,
    ignited_by_lava: bool,
    liquid: bool,
    push_reaction: PushReaction,
    map_color: MapColor,
    destroy_speed: f32,
    requires_correct_tool_for_drops: bool,
    can_occlude: bool,
    is_redstone_conductor: StatePredicate,
    is_suffocating: StatePredicate,
    is_view_blocking: ViewBlocking,
    post_process: fn(BlockState, BlockPos) -> Option<BlockPos>,
    emissive_rendering: bool,
    offset_type: OffsetType,
    spawn_terrain_particles: bool,
    instrument: NoteBlockInstrument,
    replaceable: bool,
    fluid_state: FluidState,
    is_randomly_ticking: bool,
    shapes: OnceLock<StateShapes>,
}

/// What `initCache` keeps of a state's shapes, and the constructor's `useShapeForLightOcclusion`.
/// Worked out the first time any of it is asked for, once every block is registered, where Java
/// works it out right after: the same values, since a state's shapes never change and none of
/// them reads another state's cache.
struct StateShapes {
    use_shape_for_light_occlusion: bool,
    cache: Option<Cache>,
    legacy_solid: bool,
    occlusion_shape: VoxelShape,
    occlusion_shapes_by_face: [VoxelShape; 6],
    solid_render: bool,
    propagates_skylight_down: bool,
    light_dampening: i32,
}

/// `BlockStateBase.Cache`: the collision shape with no level around it and what's worked out of
/// it, for a block without a dynamic shape.
struct Cache {
    collision_shape: VoxelShape,
    large_collision_shape: bool,
    /// By `direction * 3 + support type`.
    face_sturdy: [bool; 18],
    is_collision_shape_full_block: bool,
}

impl BlockStateBase {
    /// The constructor's part, which copies from the block's `Properties` and works out the
    /// state's light and map colour.
    pub(crate) fn new(properties: &Properties, state: BlockState) -> BlockStateBase {
        BlockStateBase {
            light_emission: (properties.light_emission)(state),
            is_air: properties.is_air,
            ignited_by_lava: properties.ignited_by_lava,
            liquid: properties.liquid,
            push_reaction: properties.push_reaction,
            map_color: (properties.map_color)(state),
            destroy_speed: properties.destroy_time,
            requires_correct_tool_for_drops: properties.requires_correct_tool_for_drops,
            can_occlude: properties.can_occlude,
            is_redstone_conductor: properties.is_redstone_conductor,
            is_suffocating: properties.is_suffocating,
            is_view_blocking: properties.is_view_blocking,
            post_process: properties.post_process,
            emissive_rendering: (properties.emissive_rendering)(state),
            offset_type: properties.offset_type,
            spawn_terrain_particles: properties.spawn_terrain_particles,
            instrument: properties.instrument,
            replaceable: properties.replaceable,
            fluid_state: FluidState::EMPTY,
            is_randomly_ticking: false,
            shapes: OnceLock::new(),
        }
    }

    /// `initCache`'s part that needs no shapes: the state's fluid and whether it ticks randomly.
    /// The rest waits for the shapes (`StateShapes`).
    pub(crate) fn init_cache(&mut self, block: &BlockData, state: BlockState) {
        self.fluid_state = block.get_fluid_state_of(state);
        self.is_randomly_ticking = block.is_randomly_ticking_state(state);
    }
}

impl StateShapes {
    /// `initCache`'s shapes, in its order.
    fn new(block: &BlockData, state: BlockState) -> StateShapes {
        let cache = (!block.has_dynamic_shape()).then(|| Cache::new(state));
        let legacy_solid = calculate_solid(block, cache.as_ref());
        let occlusion_shape = if state.can_occlude() {
            get_occlusion_shape_of(block, state)
        } else {
            Shapes::empty()
        };
        let solid_render = Block::is_shape_full_block(&occlusion_shape);
        let occlusion_shapes_by_face = if occlusion_shape.is_empty() {
            std::array::from_fn(|_| Shapes::empty())
        } else if solid_render {
            std::array::from_fn(|_| Shapes::block())
        } else {
            Direction::VALUES.map(|direction| occlusion_shape.get_face_shape(direction))
        };
        let propagates_skylight_down = block.propagates_skylight_down_of(state);
        StateShapes {
            use_shape_for_light_occlusion: block.use_shape_for_light_occlusion_of(state),
            cache,
            legacy_solid,
            occlusion_shape,
            occlusion_shapes_by_face,
            solid_render,
            propagates_skylight_down,
            light_dampening: block.get_light_dampening_of(
                state,
                solid_render,
                propagates_skylight_down,
            ),
        }
    }
}

impl Cache {
    fn new(state: BlockState) -> Cache {
        let collision_shape = state.get_collision_shape_in(
            &EmptyBlockGetter,
            &BlockPos::ZERO,
            &collision_context::empty(),
        );
        assert!(
            collision_shape.is_empty() || !state.has_offset_function(),
            "IllegalStateException: {} has a collision shape and an offset type, but is not marked as dynamicShape in its properties.",
            blocks::registered_name(state.get_block())
        );
        let large_collision_shape = Axis::VALUES
            .into_iter()
            .any(|axis| collision_shape.min(axis) < 0.0 || collision_shape.max(axis) > 1.0);
        let mut face_sturdy = [false; 18];
        for direction in Direction::VALUES {
            for support_type in SupportType::VALUES {
                face_sturdy[Cache::face_support_index(direction, support_type)] = support_type
                    .is_supporting(state, &EmptyBlockGetter, &BlockPos::ZERO, direction);
            }
        }
        // Java asks the state again, which finds no cache yet and asks the block: the same shape.
        let is_collision_shape_full_block = Block::is_shape_full_block(&collision_shape);
        Cache {
            collision_shape,
            large_collision_shape,
            face_sturdy,
            is_collision_shape_full_block,
        }
    }

    fn face_support_index(direction: Direction, support_type: SupportType) -> usize {
        direction as usize * SupportType::VALUES.len() + support_type as usize
    }
}

/// `calculateSolid`, the deprecated `isSolid`: forced either way, or a collision shape at least
/// as big as a 0.9-block cube, or as tall as a block.
fn calculate_solid(block: &BlockData, cache: Option<&Cache>) -> bool {
    if block.properties().force_solid_on {
        return true;
    }
    if block.properties().force_solid_off {
        return false;
    }
    let Some(cache) = cache else {
        return false;
    };
    let shape = &cache.collision_shape;
    if shape.is_empty() {
        return false;
    }
    let bounds = shape.bounds();
    bounds.get_size() >= 0.7291666666666666 || bounds.get_ysize() >= 1.0
}

/// `BlockStateBase`'s methods, which read the state's entry in the registry's table.
impl BlockState {
    fn base(&self) -> &'static BlockStateBase {
        &REGISTRY.states[self.id() as usize]
    }

    fn shapes(&self) -> &'static StateShapes {
        self.base()
            .shapes
            .get_or_init(|| StateShapes::new(Block::data(self.get_block()), *self))
    }

    pub fn get_light_emission(&self) -> i32 {
        self.base().light_emission
    }

    pub fn is_air(&self) -> bool {
        self.base().is_air
    }

    pub fn ignited_by_lava(&self) -> bool {
        self.base().ignited_by_lava
    }

    /// Deprecated in Java: whether the block was made `liquid()`.
    pub fn liquid(&self) -> bool {
        self.base().liquid
    }

    /// `getMapColor(BlockGetter, BlockPos)`, which reads neither.
    pub fn get_map_color(&self) -> MapColor {
        self.base().map_color
    }

    pub fn emissive_rendering(&self) -> bool {
        self.base().emissive_rendering
    }

    /// `getDestroySpeed(BlockGetter, BlockPos)`, which reads neither.
    pub fn get_destroy_speed(&self) -> f32 {
        self.base().destroy_speed
    }

    pub fn get_piston_push_reaction(&self) -> PushReaction {
        self.base().push_reaction
    }

    pub fn can_occlude(&self) -> bool {
        self.base().can_occlude
    }

    pub fn has_offset_function(&self) -> bool {
        self.base().offset_type != OffsetType::NONE
    }

    /// `getPostProcessPos(BlockGetter, BlockPos)`, which none of vanilla's read the level for.
    pub fn get_post_process_pos(&self, pos: BlockPos) -> Option<BlockPos> {
        (self.base().post_process)(*self, pos)
    }

    pub fn can_be_replaced(&self) -> bool {
        self.base().replaceable
    }

    pub fn is_randomly_ticking(&self) -> bool {
        self.base().is_randomly_ticking
    }

    pub fn get_fluid_state(&self) -> FluidState {
        self.base().fluid_state
    }

    pub fn requires_correct_tool_for_drops(&self) -> bool {
        self.base().requires_correct_tool_for_drops
    }

    pub fn should_spawn_terrain_particles(&self) -> bool {
        self.base().spawn_terrain_particles
    }

    pub fn instrument(&self) -> NoteBlockInstrument {
        self.base().instrument
    }

    /// `hasBlockEntity()`: whether the block's class is an `EntityBlock`.
    pub fn has_block_entity(&self) -> bool {
        Block::data(self.get_block()).class().is_entity_block()
    }

    /// `isRedstoneConductor(BlockGetter, BlockPos)` of a predicate that answers from the state
    /// alone, as the `BlockState` suite asks it. Panics on another.
    pub fn is_redstone_conductor(&self) -> bool {
        self.base().is_redstone_conductor.test(*self)
    }

    /// `isSuffocating(BlockGetter, BlockPos)` of a predicate that answers from the state alone.
    pub fn is_suffocating(&self) -> bool {
        self.base().is_suffocating.test(*self)
    }

    /// `isViewBlocking(BlockGetter, BlockPos, AABB)` of a predicate that answers from the state
    /// alone.
    pub fn is_view_blocking(&self) -> bool {
        match self.base().is_view_blocking {
            ViewBlocking::State(test) => test(*self),
            ViewBlocking::Default(_) | ViewBlocking::NearPlaneIntersectsOutline => {
                panic!("this isViewBlocking needs the level, which this asks without")
            }
        }
    }

    /// `isRedstoneConductor(BlockGetter, BlockPos)`: Properties' default is whether the
    /// collision shape is the whole block.
    pub fn is_redstone_conductor_at(&self, level: &dyn BlockGetter, pos: &BlockPos) -> bool {
        self.base().is_redstone_conductor.test_at(*self, level, pos)
    }

    /// `isSuffocating(BlockGetter, BlockPos)`: Properties' default is a block in the
    /// `causes_suffocation` tag whose collision shape is the whole block.
    pub fn is_suffocating_at(&self, level: &dyn BlockGetter, pos: &BlockPos) -> bool {
        self.base().is_suffocating.test_at(*self, level, pos)
    }

    /// `isViewBlocking(BlockGetter, BlockPos, AABB)`: Properties' default is its `isSuffocating`.
    pub fn is_view_blocking_at(
        &self,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        near_plane_box: &AABB,
    ) -> bool {
        match self.base().is_view_blocking {
            ViewBlocking::State(test) => test(*self),
            ViewBlocking::Default(None) => self.is_suffocating_at(level, pos),
            ViewBlocking::Default(Some(is_suffocating)) => {
                is_suffocating.test_at(*self, level, pos)
            }
            // `Blocks.NEAR_PLANE_INTERSECTS_OUTLINE`.
            ViewBlocking::NearPlaneIntersectsOutline => self
                .get_occlusion_shape()
                .to_aabbs()
                .iter()
                .any(|outline_box| outline_box.move_block_pos(pos).intersects(near_plane_box)),
        }
    }

    pub fn use_shape_for_light_occlusion(&self) -> bool {
        self.shapes().use_shape_for_light_occlusion
    }

    pub fn propagates_skylight_down(&self) -> bool {
        self.shapes().propagates_skylight_down
    }

    pub fn get_light_dampening(&self) -> i32 {
        self.shapes().light_dampening
    }

    /// `getFaceOcclusionShape`: nothing and the whole block are the shared constants, which the
    /// renderer tells apart by reference.
    pub fn get_face_occlusion_shape(&self, direction: Direction) -> VoxelShape {
        self.face_occlusion_shape(direction).clone()
    }

    /// `getFaceOcclusionShape` borrowed, which `Block.shouldRenderFace` compares by reference.
    pub fn face_occlusion_shape(&self, direction: Direction) -> &'static VoxelShape {
        &self.shapes().occlusion_shapes_by_face[direction as usize]
    }

    /// `getOcclusionShape()`: the block's, or nothing for a block that can't occlude.
    pub fn get_occlusion_shape(&self) -> VoxelShape {
        self.shapes().occlusion_shape.clone()
    }

    /// `isSolidRender()`: whether the occlusion shape is the whole block.
    pub fn is_solid_render(&self) -> bool {
        self.shapes().solid_render
    }

    /// The deprecated `isSolid()`, `calculateSolid`'s.
    pub fn is_solid(&self) -> bool {
        self.shapes().legacy_solid
    }

    pub fn is_light_permeable(&self) -> bool {
        !self.is_solid_render() || self.get_light_dampening() == 0
    }
}

/// `getOcclusionShape(BlockState)`, as `initCache` asks it: the class's, or `BlockBehaviour`'s,
/// the shape in an empty level.
fn get_occlusion_shape_of(block: &BlockData, state: BlockState) -> VoxelShape {
    match block.class().find(|class| class.get_occlusion_shape) {
        Some(get_occlusion_shape) => get_occlusion_shape(block, state),
        None => get_occlusion_shape(block, state),
    }
}

/// `BlockBehaviour.getOcclusionShape`: the shape in an empty level.
pub fn get_occlusion_shape(_block: &BlockData, state: BlockState) -> VoxelShape {
    state.get_shape(&EmptyBlockGetter, &BlockPos::ZERO)
}

/// `BlockBehaviour.getCollisionShape`: the shape, or nothing for a block without collision.
pub fn get_collision_shape(
    block: &BlockData,
    state: BlockState,
    level: &dyn BlockGetter,
    pos: &BlockPos,
    _context: &dyn CollisionContext,
) -> VoxelShape {
    if block.has_collision() {
        state.get_shape(level, pos)
    } else {
        Shapes::empty()
    }
}

/// `BlockBehaviour.getVisualShape`: the collision shape.
pub fn get_visual_shape(
    _block: &BlockData,
    state: BlockState,
    level: &dyn BlockGetter,
    pos: &BlockPos,
    context: &dyn CollisionContext,
) -> VoxelShape {
    state.get_collision_shape_in(level, pos, context)
}

/// `BlockBehaviour.getBlockSupportShape`: the collision shape for no one.
pub fn get_block_support_shape(
    _block: &BlockData,
    state: BlockState,
    level: &dyn BlockGetter,
    pos: &BlockPos,
) -> VoxelShape {
    state.get_collision_shape_in(level, pos, &collision_context::empty())
}

/// `BlockBehaviour.getShadeBrightness`: dark under a whole block's collision shape.
pub fn get_shade_brightness(
    _block: &BlockData,
    state: BlockState,
    level: &dyn BlockGetter,
    pos: &BlockPos,
) -> f32 {
    if state.is_collision_shape_full_block(level, pos) {
        0.2
    } else {
        1.0
    }
}

/// The shapes and what's worked out of them, which go by the block's class.
impl BlockState {
    fn class(&self) -> &'static BlockClass {
        Block::data(self.get_block()).class()
    }

    /// `getShape(BlockGetter, BlockPos, CollisionContext)`: the class's, or `BlockBehaviour`'s
    /// whole block.
    pub fn get_shape_in(
        &self,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        match self.class().find(|class| class.get_shape) {
            Some(get_shape) => get_shape(Block::data(self.get_block()), *self, level, pos, context),
            None => Shapes::block(),
        }
    }

    /// `getShape(BlockGetter, BlockPos)`, with an empty context.
    pub fn get_shape(&self, level: &dyn BlockGetter, pos: &BlockPos) -> VoxelShape {
        self.get_shape_in(level, pos, &collision_context::empty())
    }

    /// `getCollisionShape(BlockGetter, BlockPos, CollisionContext)`: the class's, or
    /// `BlockBehaviour`'s.
    pub fn get_collision_shape_in(
        &self,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        let block = Block::data(self.get_block());
        match self.class().find(|class| class.get_collision_shape) {
            Some(get_collision_shape) => get_collision_shape(block, *self, level, pos, context),
            None => get_collision_shape(block, *self, level, pos, context),
        }
    }

    /// `getCollisionShape(BlockGetter, BlockPos)`: the cached one, or with an empty context for a
    /// block with a dynamic shape.
    pub fn get_collision_shape(&self, level: &dyn BlockGetter, pos: &BlockPos) -> VoxelShape {
        match &self.shapes().cache {
            Some(cache) => cache.collision_shape.clone(),
            None => self.get_collision_shape_in(level, pos, &collision_context::empty()),
        }
    }

    /// `getVisualShape(BlockGetter, BlockPos, CollisionContext)`.
    pub fn get_visual_shape(
        &self,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        let block = Block::data(self.get_block());
        match self.class().find(|class| class.get_visual_shape) {
            Some(get_visual_shape) => get_visual_shape(block, *self, level, pos, context),
            None => get_visual_shape(block, *self, level, pos, context),
        }
    }

    /// `getBlockSupportShape(BlockGetter, BlockPos)`.
    pub fn get_block_support_shape(&self, level: &dyn BlockGetter, pos: &BlockPos) -> VoxelShape {
        let block = Block::data(self.get_block());
        match self.class().find(|class| class.get_block_support_shape) {
            Some(get_block_support_shape) => get_block_support_shape(block, *self, level, pos),
            None => get_block_support_shape(block, *self, level, pos),
        }
    }

    /// `getInteractionShape(BlockGetter, BlockPos)`: `BlockBehaviour`'s is nothing.
    pub fn get_interaction_shape(&self, level: &dyn BlockGetter, pos: &BlockPos) -> VoxelShape {
        match self.class().find(|class| class.get_interaction_shape) {
            Some(get_interaction_shape) => {
                get_interaction_shape(Block::data(self.get_block()), *self, level, pos)
            }
            None => Shapes::empty(),
        }
    }

    /// `isCollisionShapeFullBlock(BlockGetter, BlockPos)`: the cached answer, or the class's for
    /// a block with a dynamic shape.
    pub fn is_collision_shape_full_block(&self, level: &dyn BlockGetter, pos: &BlockPos) -> bool {
        if let Some(cache) = &self.shapes().cache {
            return cache.is_collision_shape_full_block;
        }
        match self
            .class()
            .find(|class| class.is_collision_shape_full_block)
        {
            Some(is_collision_shape_full_block) => {
                is_collision_shape_full_block(Block::data(self.get_block()), *self, level, pos)
            }
            None => Block::is_shape_full_block(&self.get_collision_shape(level, pos)),
        }
    }

    /// Whether the collision shape reaches past the block, which is always taken to be so for a
    /// block with a dynamic shape (no cache).
    pub fn has_large_collision_shape(&self) -> bool {
        self.shapes()
            .cache
            .as_ref()
            .is_none_or(|cache| cache.large_collision_shape)
    }

    /// `isFaceSturdy(BlockGetter, BlockPos, Direction)`: whether the face fully supports.
    pub fn is_face_sturdy(
        &self,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction: Direction,
    ) -> bool {
        self.is_face_sturdy_for(level, pos, direction, SupportType::FULL)
    }

    /// `isFaceSturdy(BlockGetter, BlockPos, Direction, SupportType)`: the cached answer, or the
    /// support type's for a block with a dynamic shape.
    pub fn is_face_sturdy_for(
        &self,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction: Direction,
        support_type: SupportType,
    ) -> bool {
        match &self.shapes().cache {
            Some(cache) => cache.face_sturdy[Cache::face_support_index(direction, support_type)],
            None => support_type.is_supporting(*self, level, pos, direction),
        }
    }
}

/// What the renderer reads of a state, from its block's class.
impl BlockState {
    /// `getBlock() instanceof LeavesBlock`.
    pub fn class_is_leaves(&self) -> bool {
        self.class_is("net.minecraft.world.level.block.LeavesBlock")
    }

    /// `getBlock() instanceof HalfTransparentBlock`: glass, ice, slime, honey and the like.
    pub fn class_is_half_transparent(&self) -> bool {
        self.class_is("net.minecraft.world.level.block.HalfTransparentBlock")
    }

    /// Whether the block's class is `name` or extends it.
    pub(crate) fn class_is(&self, name: &str) -> bool {
        let mut class = Some(self.class());
        while let Some(current) = class {
            if current.name == name {
                return true;
            }
            class = current.parent;
        }
        false
    }

    /// `getShadeBrightness(BlockGetter, BlockPos)`.
    pub fn get_shade_brightness(&self, level: &dyn BlockGetter, pos: &BlockPos) -> f32 {
        let block = Block::data(self.get_block());
        match self.class().find(|class| class.get_shade_brightness) {
            Some(get_shade_brightness) => get_shade_brightness(block, *self, level, pos),
            None => get_shade_brightness(block, *self, level, pos),
        }
    }

    /// `getRenderShape()`: `BlockBehaviour`'s is a model.
    pub fn get_render_shape(&self) -> RenderShape {
        match self.class().find(|class| class.get_render_shape) {
            Some(get_render_shape) => get_render_shape(Block::data(self.get_block()), *self),
            None => RenderShape::MODEL,
        }
    }

    /// `skipRendering`: whether a face toward this neighbour is left out whatever the shapes say.
    pub fn skip_rendering(&self, neighbor_state: BlockState, direction: Direction) -> bool {
        match self.class().find(|class| class.skip_rendering) {
            Some(skip_rendering) => skip_rendering(
                Block::data(self.get_block()),
                *self,
                neighbor_state,
                direction,
            ),
            None => false,
        }
    }

    /// `getSeed`: the position's, which the beds, doors and double plants take from the half the
    /// two share.
    pub fn get_seed(&self, pos: &BlockPos) -> i64 {
        match self.class().find(|class| class.get_seed) {
            Some(get_seed) => get_seed(Block::data(self.get_block()), *self, pos),
            None => Mth::get_seed_vec3i(pos),
        }
    }

    /// `getOffset`: where the state's model moves in its block, by the offset type its
    /// `Properties` gave it, or nowhere without one.
    pub fn get_offset(&self, pos: &BlockPos) -> Vec3 {
        let block = Block::data(self.get_block());
        let horizontal = || {
            #[allow(deprecated)]
            let seed = Mth::get_seed(pos.get_x(), 0, pos.get_z());
            let max_horizontal_offset = f64::from(block.get_max_horizontal_offset());
            let x = Mth::clamp_f64(
                (f64::from((seed & 15) as f32 / 15.0) - 0.5) * 0.5,
                -max_horizontal_offset,
                max_horizontal_offset,
            );
            let z = Mth::clamp_f64(
                (f64::from((seed >> 8 & 15) as f32 / 15.0) - 0.5) * 0.5,
                -max_horizontal_offset,
                max_horizontal_offset,
            );
            (seed, x, z)
        };
        match self.base().offset_type {
            OffsetType::NONE => Vec3::ZERO,
            OffsetType::XZ => {
                let (_, x, z) = horizontal();
                Vec3::new(x, 0.0, z)
            }
            OffsetType::XYZ => {
                let (seed, x, z) = horizontal();
                let y = (f64::from((seed >> 4 & 15) as f32 / 15.0) - 1.0)
                    * f64::from(block.get_max_vertical_offset());
                Vec3::new(x, y, z)
            }
        }
    }
}

/// `RenderShape`: whether a state's model is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderShape {
    INVISIBLE,
    MODEL,
}

/// `EmptyBlockGetter.INSTANCE`: air everywhere, which shapes are cached with.
pub struct EmptyBlockGetter;

impl BlockGetter for EmptyBlockGetter {
    fn get_block_state(&self, _pos: &BlockPos) -> BlockState {
        crate::net::minecraft::world::level::block::blocks::Blocks::AIR.default_block_state()
    }

    fn get_fluid_state(&self, _pos: &BlockPos) -> FluidState {
        FluidState::EMPTY
    }
}

impl StatePredicate {
    /// Panics on a predicate that needs the level or the block's shapes.
    pub fn test(&self, state: BlockState) -> bool {
        match self {
            StatePredicate::State(_, test) => test(state),
            StatePredicate::Default
            | StatePredicate::DefaultSuffocating
            | StatePredicate::NotClosedShulker => {
                panic!("this predicate needs the level, which this asks without")
            }
        }
    }

    /// `test(BlockState, BlockGetter, BlockPos)`.
    pub fn test_at(&self, state: BlockState, level: &dyn BlockGetter, pos: &BlockPos) -> bool {
        match self {
            StatePredicate::State(_, test) => test(state),
            // `Properties`' default `isRedstoneConductor`, and its `isSuffocating`, which asks
            // the tag first.
            StatePredicate::Default => state.is_collision_shape_full_block(level, pos),
            StatePredicate::DefaultSuffocating => {
                state.is_tag(BlockTags::CAUSES_SUFFOCATION)
                    && state.is_collision_shape_full_block(level, pos)
            }
            // A shulker box's block entity is closed unless it's opening or closing, which the
            // client's block entities don't do yet, and a level without one answers true too.
            StatePredicate::NotClosedShulker => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use serde_json::{Value, json};

    use super::EmptyBlockGetter;
    use crate::golden::{Decode, Encode, Golden, catch};
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::references::block_item_ids::BlockItemIds;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block::blocks::Blocks;
    use crate::net::minecraft::world::level::block::cross_collision_block::CrossCollisionBlock;
    use crate::net::minecraft::world::level::block::liquid_block::LiquidBlock;
    use crate::net::minecraft::world::level::block::slab_block::SlabBlock;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
    use crate::net::minecraft::world::level::block::support_type::SupportType;
    use crate::net::minecraft::world::phys::aabb::AABB;
    use crate::net::minecraft::world::phys::shapes::collision_context::{self, CollisionContext};
    use crate::net::minecraft::world::phys::shapes::entity_collision_context::EntityCollisionContext;
    use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

    const POS: BlockPos = BlockPos::new(1, 2, 3);

    /// The blocks `BlockStateShapeSuite` asks at besides `BlockPos.ZERO`, in its order.
    const POSITIONS: [BlockPos; 7] = [
        BlockPos::ZERO,
        POS,
        BlockPos::new(-37, 70, 1290),
        BlockPos::new(12345, -60, -9876),
        BlockPos::new(-1, 0, -1),
        BlockPos::new(29999999, 319, -29999999),
        BlockPos::new(7, 64, -300),
    ];

    /// The suite's contexts, in its order: none, none colliding with fluids, and no entity whose
    /// bottom is a block up, a block up descending, halfway, a block down, a block up colliding
    /// with fluids, holding scaffolding, holding a light, and placing a block.
    fn contexts() -> Vec<EntityCollisionContext> {
        let above = || EntityCollisionContext::new(false, false, 1.0, false);
        vec![
            collision_context::empty(),
            collision_context::empty_with_fluid_collisions(),
            above(),
            EntityCollisionContext::new(true, false, 1.0, false),
            EntityCollisionContext::new(false, false, 0.5, false),
            EntityCollisionContext::new(false, false, -1.0, false),
            EntityCollisionContext::new(false, false, 1.0, true),
            above().with_held_item(Some(BlockItemIds::SCAFFOLDING.item().clone())),
            above().with_held_item(Some(BlockItemIds::LIGHT.item().clone())),
            collision_context::placement_context_without_player(),
        ]
    }

    /// The neighbours the suite asks `skipRendering` about, after the state and its block's
    /// default state.
    fn neighbours() -> Vec<BlockState> {
        let default = |block: Block| block.default_block_state();
        let mut bars = default(Blocks::IRON_BARS);
        for (_, side) in CrossCollisionBlock::PROPERTY_BY_DIRECTION {
            bars = bars.set_value(side, true);
        }
        vec![
            default(Blocks::AIR),
            default(Blocks::STONE),
            default(Blocks::GLASS),
            default(Blocks::STAINED_GLASS.as_list()[0]),
            default(Blocks::TINTED_GLASS),
            default(Blocks::ICE),
            default(Blocks::OAK_LEAVES),
            default(Blocks::CHERRY_LEAVES),
            default(Blocks::MANGROVE_ROOTS),
            default(Blocks::IRON_BARS),
            bars,
            default(Blocks::GLASS_PANE)
                .set_value(CrossCollisionBlock::NORTH, true)
                .set_value(CrossCollisionBlock::SOUTH, true),
            default(Blocks::COPPER_BARS.weathering().as_list()[0])
                .set_value(CrossCollisionBlock::EAST, true)
                .set_value(CrossCollisionBlock::WEST, true),
            default(Blocks::WATER),
            default(Blocks::WATER).set_value(LiquidBlock::LEVEL, 3),
            default(Blocks::LAVA),
            default(Blocks::KELP),
            default(Blocks::OAK_SLAB).set_value(SlabBlock::WATERLOGGED, true),
            default(Blocks::POWDER_SNOW),
            default(Blocks::SLIME_BLOCK),
            default(Blocks::HONEY_BLOCK),
            default(Blocks::BARRIER),
            default(Blocks::COPPER_GRATE.weathering().as_list()[0]),
            default(Blocks::BUBBLE_COLUMN),
        ]
    }

    /// Shapes encoded as the suite encodes them, each object once: the encoder keeps each shape
    /// it has seen, so its address isn't another's while the test runs.
    #[derive(Default)]
    struct Shapes(RefCell<HashMap<usize, (VoxelShape, Value)>>);

    impl Shapes {
        fn encode(&self, shape: VoxelShape) -> Value {
            self.0
                .borrow_mut()
                .entry(shape.address())
                .or_insert_with(|| {
                    let encoded = shape.encode();
                    (shape, encoded)
                })
                .1
                .clone()
        }
    }

    /// Bits as `BigInteger.toString(16)` writes them: lowercase, no leading zeros.
    fn hex(bits: &[bool]) -> String {
        let digits: String = bits
            .chunks(4)
            .rev()
            .map(|nibble| {
                let value = nibble
                    .iter()
                    .enumerate()
                    .fold(0, |value, (bit, &set)| value | (u32::from(set) << bit));
                char::from_digit(value, 16).expect("a nibble is a hex digit")
            })
            .collect();
        match digits.trim_start_matches('0') {
            "" => "0".to_owned(),
            trimmed => trimmed.to_owned(),
        }
    }

    // What each element of a state's case is, for the messages.
    const NAMES: [&str; 22] = [
        "getShape",
        "getCollisionShape",
        "getOcclusionShape",
        "getFaceOcclusionShape",
        "getBlockSupportShape",
        "getInteractionShape",
        "getVisualShape",
        "useShapeForLightOcclusion",
        "propagatesSkylightDown",
        "getLightDampening",
        "isSolidRender",
        "isSolid",
        "isCollisionShapeFullBlock",
        "hasLargeCollisionShape",
        "isFaceSturdy",
        "getShadeBrightness",
        "getRenderShape",
        "getSeed",
        "skipRendering",
        "isRedstoneConductor",
        "isSuffocating",
        "isViewBlocking",
    ];

    /// Everything the suite records of a state, with each shape encoded.
    fn state_case(state: BlockState, neighbours: &[BlockState], shapes: &Shapes) -> Value {
        let level = &EmptyBlockGetter;
        let zero = &BlockPos::ZERO;
        let mut sturdy = 0;
        for direction in Direction::VALUES {
            for support_type in SupportType::VALUES {
                if state.is_face_sturdy_for(level, zero, direction, support_type) {
                    sturdy |= 1 << (direction as u32 * 3 + support_type as u32);
                }
            }
        }
        let asked: Vec<BlockState> = [state, state.get_block().default_block_state()]
            .into_iter()
            .chain(neighbours.iter().copied())
            .collect();
        let skips: Vec<bool> = asked
            .iter()
            .flat_map(|&neighbour| {
                Direction::VALUES.map(|direction| state.skip_rendering(neighbour, direction))
            })
            .collect();
        json!([
            shapes.encode(state.get_shape(level, zero)),
            shapes.encode(state.get_collision_shape(level, zero)),
            shapes.encode(state.get_occlusion_shape()),
            Direction::VALUES
                .map(|direction| shapes.encode(state.get_face_occlusion_shape(direction))),
            shapes.encode(state.get_block_support_shape(level, zero)),
            shapes.encode(state.get_interaction_shape(level, zero)),
            shapes.encode(state.get_visual_shape(level, zero, &collision_context::empty())),
            state.use_shape_for_light_occlusion(),
            state.propagates_skylight_down(),
            state.get_light_dampening(),
            state.is_solid_render(),
            state.is_solid(),
            state.is_collision_shape_full_block(level, zero),
            state.has_large_collision_shape(),
            sturdy,
            state.get_shade_brightness(level, zero).encode(),
            format!("{:?}", state.get_render_shape()),
            state.get_seed(&POS),
            hex(&skips),
            state.is_redstone_conductor_at(level, &POS),
            state.is_suffocating_at(level, &POS),
            state.is_view_blocking_at(level, &POS, &AABB::from_block_pos(&POS)),
        ])
    }

    /// What vanilla recorded, each shape's index replaced with the shape.
    fn with_shapes(expected: &Value, shapes: &[Value]) -> Value {
        let shape =
            |index: &Value| shapes[index.as_u64().expect("a shape's index") as usize].clone();
        let elements = expected.as_array().expect("a state's case is an array");
        let mut full = elements.clone();
        for i in [0, 1, 2, 4, 5, 6] {
            full[i] = shape(&elements[i]);
        }
        full[3] = Value::Array(
            elements[3]
                .as_array()
                .expect("the faces are an array")
                .iter()
                .map(shape)
                .collect(),
        );
        Value::Array(full)
    }

    /// A state's shapes asked of its block's methods at a block for a context, as the variants
    /// record them: `[getShape, getCollisionShape, getVisualShape, getInteractionShape,
    /// getBlockSupportShape]`.
    fn asked_of_block(
        state: BlockState,
        pos: &BlockPos,
        context: &dyn CollisionContext,
        shapes: &Shapes,
    ) -> Value {
        let level = &EmptyBlockGetter;
        json!([
            shapes.encode(state.get_shape_in(level, pos, context)),
            shapes.encode(state.get_collision_shape_in(level, pos, context)),
            shapes.encode(state.get_visual_shape(level, pos, context)),
            shapes.encode(state.get_interaction_shape(level, pos)),
            shapes.encode(state.get_block_support_shape(level, pos)),
        ])
    }

    /// Every state's shapes, light, occlusion, support and what the renderer reads, against
    /// `ShapeSuites.BlockStateShapeSuite`: as a state reports them with no level around it, then
    /// asked of its block for every context and block the suite asks at, and its offset.
    #[test]
    fn state_shapes_match_vanilla() {
        let golden =
            Golden::load("net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase");
        let mut shapes = golden.cases("shape");
        shapes.sort_by_key(|(args, _)| args[0].as_u64());
        let shapes: Vec<Value> = shapes.into_iter().map(|(_, shape)| shape).collect();
        let encoder = Shapes::default();
        let neighbours = neighbours();
        let mut failures = Vec::new();

        for (args, expected) in golden.cases("state") {
            let state = BlockState::decode(&args[0]);
            let expected = with_shapes(&expected, &shapes);
            match catch(|| state_case(state, &neighbours, &encoder)) {
                Ok(actual) => {
                    for (i, name) in NAMES.iter().enumerate() {
                        if actual[i] != expected[i] {
                            failures.push(format!(
                                "{state} ({}): {name}: expected {}, got {}",
                                state.id(),
                                expected[i],
                                actual[i]
                            ));
                        }
                    }
                }
                Err(panic) => failures.push(format!("{state} ({}): panicked: {panic}", state.id())),
            }
        }

        let contexts = contexts();
        for (args, expected) in golden.cases("variants") {
            let block = Block::decode(&args[0]);
            let states: Vec<BlockState> =
                block.get_state_definition().get_possible_states().collect();
            let [runs, changed] = &**expected.as_array().expect("a block's variants") else {
                panic!("variants are runs and the changes");
            };
            let base: Vec<Value> = runs
                .as_array()
                .expect("runs")
                .iter()
                .flat_map(|run| {
                    let count = run[1].as_u64().expect("a run's count") as usize;
                    std::iter::repeat_n(run[0].clone(), count)
                })
                .collect();
            let shapes_of = |indexes: &Value| -> Value {
                indexes
                    .as_array()
                    .expect("a variant's shapes")
                    .iter()
                    .map(|index| shapes[index.as_u64().expect("an index") as usize].clone())
                    .collect()
            };
            let variants = contexts.len() + POSITIONS.len() - 1;
            for variant in 0..variants {
                let (context, pos) = if variant < contexts.len() {
                    (&contexts[variant], &BlockPos::ZERO)
                } else {
                    (&contexts[0], &POSITIONS[variant - contexts.len() + 1])
                };
                let differences = changed
                    .as_array()
                    .expect("the changed variants")
                    .iter()
                    .find(|entry| entry[0].as_u64() == Some(variant as u64))
                    .map(|entry| entry[1].as_array().expect("the changed states").clone())
                    .unwrap_or_default();
                for (i, &state) in states.iter().enumerate() {
                    let expected = differences
                        .iter()
                        .find(|entry| entry[0].as_u64() == Some(i as u64))
                        .map_or(&base[i], |entry| &entry[1]);
                    match catch(|| asked_of_block(state, pos, context, &encoder)) {
                        Ok(actual) if actual == shapes_of(expected) => {}
                        Ok(actual) => failures.push(format!(
                            "{state} ({}): variant {variant}: expected {}, got {actual}",
                            state.id(),
                            shapes_of(expected)
                        )),
                        Err(panic) => failures.push(format!(
                            "{state} ({}): variant {variant}: panicked: {panic}",
                            state.id()
                        )),
                    }
                }
            }
        }

        for (args, expected) in golden.cases("getOffset") {
            let state = BlockState::decode(&args[0]);
            let actual = catch(|| {
                POSITIONS
                    .iter()
                    .map(|pos| state.get_offset(pos).encode())
                    .collect::<Vec<_>>()
            });
            match actual {
                Ok(actual) if Value::Array(actual.clone()) == expected => {}
                Ok(actual) => failures.push(format!(
                    "{state} ({}): getOffset: expected {expected}, got {actual:?}",
                    state.id()
                )),
                Err(panic) => failures.push(format!(
                    "{state} ({}): getOffset panicked: {panic}",
                    state.id()
                )),
            }
        }

        let blocks_failing: std::collections::BTreeSet<String> = failures
            .iter()
            .map(|failure| failure.split(['[', ' ']).next().unwrap_or("").to_owned())
            .collect();
        assert!(
            failures.is_empty(),
            "{} mismatches in {} blocks ({}):\n{}",
            failures.len(),
            blocks_failing.len(),
            blocks_failing.into_iter().collect::<Vec<_>>().join(", "),
            failures[..failures.len().min(60)].join("\n")
        );
        golden.finish();
    }
}
