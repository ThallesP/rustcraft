//! net/minecraft/world/level/block/state/BlockBehaviour.java
//!
//! Only what decides a block's states and what they keep from its `Properties`: `Properties`
//! itself, the fields `BlockBehaviour` copies from it, and what `BlockStateBase` keeps of each
//! state and answers without a level (`getLightEmission`, `isAir`, `canOcclude`,
//! `getPistonPushReaction`, `getDestroySpeed`, `getMapColor`, `emissiveRendering`,
//! `isRandomlyTicking`, `getPostProcessPos`, the predicates that answer from the state alone…).
//!
//! The shapes: `getShape` and `getCollisionShape` with `BlockBehaviour`'s defaults (the block,
//! empty without collision) and `AirBlock`'s, `isCollisionShapeFullBlock`, `hasLargeCollisionShape`
//! and `isSuffocating` at a position, worked out when asked rather than cached by `initCache`, which
//! gives the same shapes for a block without a dynamic shape. A block whose class (or a parent)
//! overrides `getShape` or `getCollisionShape` in Java panics when asked, until its override is
//! ported (`SHAPE_OVERRIDES`).
//!
//! `initCache`'s light part, which the light engine reads: `useShapeForLightOcclusion`, the
//! occlusion shape and its faces, `isSolidRender`, `propagatesSkylightDown` and
//! `getLightDampening`, with the classes' overrides. They need the state's shape where a class
//! doesn't answer without one, so a state whose class's shapes aren't ported has none of them yet
//! (`light_properties_ported`), and asking panics, like the shapes themselves. So does the
//! occlusion shape of the classes overriding `getOcclusionShape` whose overrides aren't ported
//! (fence gates, lecterns, sculk shriekers; fences' is).
//!
//! Not ported yet: every other behaviour method (`useWithoutItem`, `tick`, `updateShape`…), which
//! come with the blocks' classes; `Properties.sound` and `SoundType`, `isValidSpawn` (it
//! takes an `EntityType`) and `requiredFeatures` (`FeatureFlag`), which Blocks' registrations
//! leave out until they come; `initCache`'s `isSolid` and collision shape cache; the offset
//! function itself (`getOffset`), which needs each class's `getMaxHorizontalOffset`.
//!
//! Java's state predicates are lambdas, so `Properties` holds them as values that say which they
//! are: Properties' defaults, which read the block's shapes or tags, Blocks' named ones, or a
//! function of the state alone. A function a level is passed to (`getPostProcessPos`,
//! `isRedstoneConductor`…) leaves the level out where none of vanilla's read it.

use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use std::sync::Arc;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::util::Util;
use crate::net::minecraft::world::item::dye_color::DyeColor;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, ShapeMethod,
};
use crate::net::minecraft::world::level::block::blocks::REGISTRY;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::note_block_instrument::NoteBlockInstrument;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::map_color::MapColor;
use crate::net::minecraft::world::level::material::push_reaction::PushReaction;
use crate::net::minecraft::world::phys::shapes::collision_context::{self, CollisionContext};
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// `BlockBehaviour.StatePredicate`, as `Properties` holds one for `isRedstoneConductor` and
/// `isSuffocating`.
#[derive(Clone, Copy)]
pub enum StatePredicate {
    /// What `Properties` starts with: `isCollisionShapeFullBlock` for `isRedstoneConductor`, and
    /// that and the `causes_suffocation` tag for `isSuffocating`. Not ported: they need the
    /// block's shapes.
    Default,
    /// `Blocks.NOT_CLOSED_SHULKER`, which asks the shulker box's block entity. Not ported.
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
    /// `Blocks.NEAR_PLANE_INTERSECTS_OUTLINE`, which reads the state's occlusion shape. Not ported.
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
            is_suffocating: StatePredicate::Default,
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

    /// `offsetType`, which picks the offset function; only whether there is one is ported.
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
    has_offset_function: bool,
    spawn_terrain_particles: bool,
    instrument: NoteBlockInstrument,
    replaceable: bool,
    fluid_state: FluidState,
    is_randomly_ticking: bool,
    use_shape_for_light_occlusion: bool,
    occlusion: Option<Occlusion>,
    propagates_skylight_down: Option<bool>,
    light_dampening: Option<i32>,
}

/// What `initCache` keeps of a state's occlusion shape.
struct Occlusion {
    occlusion_shape: VoxelShape,
    occlusion_shapes_by_face: [VoxelShape; 6],
    solid_render: bool,
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
            has_offset_function: properties.offset_type != OffsetType::NONE,
            spawn_terrain_particles: properties.spawn_terrain_particles,
            instrument: properties.instrument,
            replaceable: properties.replaceable,
            fluid_state: FluidState::EMPTY,
            is_randomly_ticking: false,
            use_shape_for_light_occlusion: false,
            occlusion: None,
            propagates_skylight_down: None,
            light_dampening: None,
        }
    }

    /// `initCache`'s part that's ported: the state's fluid, whether it ticks randomly, its
    /// occlusion shape, and its light, which its block's class may decide. They need the state's
    /// shape where the class doesn't answer without it, which a class whose shapes aren't ported
    /// doesn't have yet: that part is left unknown.
    pub(crate) fn init_cache(&mut self, block: &BlockData, state: BlockState) {
        self.fluid_state = block.get_fluid_state_of(state);
        self.is_randomly_ticking = block.is_randomly_ticking_state(state);
        // The constructor's, in Java.
        self.use_shape_for_light_occlusion = block.use_shape_for_light_occlusion_of(state);

        let occlusion_shape = if self.can_occlude {
            occlusion_shape_of(block, state)
        } else {
            Some(Shapes::empty())
        };
        self.occlusion = occlusion_shape.map(|occlusion_shape| {
            let solid_render = Block::is_shape_full_block(&occlusion_shape);
            let occlusion_shapes_by_face = if occlusion_shape.is_empty() {
                std::array::from_fn(|_| Shapes::empty())
            } else if solid_render {
                std::array::from_fn(|_| Shapes::block())
            } else {
                Direction::VALUES.map(|direction| occlusion_shape.get_face_shape(direction))
            };
            Occlusion {
                occlusion_shape,
                occlusion_shapes_by_face,
                solid_render,
            }
        });
        self.propagates_skylight_down = block.propagates_skylight_down_of(state, &self.fluid_state);
        self.light_dampening = block.get_light_dampening_of(
            state,
            self.occlusion
                .as_ref()
                .map(|occlusion| occlusion.solid_render),
            self.propagates_skylight_down,
        );
    }
}

/// `BlockStateBase`'s methods, which read the state's entry in the registry's table.
impl BlockState {
    fn base(&self) -> &'static BlockStateBase {
        &REGISTRY.states[self.id() as usize]
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
        self.base().has_offset_function
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

    /// `isRedstoneConductor(BlockGetter, BlockPos)`, where the predicate answers from the state
    /// alone. Panics on one that needs the level or the block's shapes, which aren't ported.
    pub fn is_redstone_conductor(&self) -> bool {
        self.base().is_redstone_conductor.test(*self)
    }

    /// `isSuffocating(BlockGetter, BlockPos)`, like `is_redstone_conductor`.
    pub fn is_suffocating(&self) -> bool {
        self.base().is_suffocating.test(*self)
    }

    pub fn use_shape_for_light_occlusion(&self) -> bool {
        self.base().use_shape_for_light_occlusion
    }

    /// Whether the state's occlusion shape and light are known: whether its class's shapes are
    /// ported, where they need them.
    pub fn light_properties_ported(&self) -> bool {
        let base = self.base();
        base.occlusion.is_some()
            && base.propagates_skylight_down.is_some()
            && base.light_dampening.is_some()
    }

    /// Panics for a state whose class's shapes aren't ported, which the value needs.
    fn known<T>(&self, value: Option<T>, what: &str) -> T {
        value.unwrap_or_else(|| {
            panic!(
                "the {what} of {} needs its shapes, which aren't ported ({})",
                self,
                Block::data(self.get_block()).class().name
            )
        })
    }

    fn occlusion(&self) -> &'static Occlusion {
        self.known(self.base().occlusion.as_ref(), "occlusion shape")
    }

    pub fn propagates_skylight_down(&self) -> bool {
        self.known(self.base().propagates_skylight_down, "skylight")
    }

    pub fn get_light_dampening(&self) -> i32 {
        self.known(self.base().light_dampening, "light dampening")
    }

    /// `getFaceOcclusionShape`: nothing and the whole block are the shared constants, which the
    /// renderer tells apart by reference.
    pub fn get_face_occlusion_shape(&self, direction: Direction) -> VoxelShape {
        self.occlusion().occlusion_shapes_by_face[direction as usize].clone()
    }

    /// `getOcclusionShape()`: the block's shape in an empty level, or nothing for a block that
    /// can't occlude.
    pub fn get_occlusion_shape(&self) -> VoxelShape {
        self.occlusion().occlusion_shape.clone()
    }

    /// `isSolidRender()`: whether the occlusion shape is the whole block.
    pub fn is_solid_render(&self) -> bool {
        self.occlusion().solid_render
    }

    /// `isViewBlocking(BlockGetter, BlockPos, AABB)`, like `is_redstone_conductor`.
    pub fn is_view_blocking(&self) -> bool {
        match self.base().is_view_blocking {
            ViewBlocking::State(test) => test(*self),
            ViewBlocking::Default(_) | ViewBlocking::NearPlaneIntersectsOutline => {
                panic!("isViewBlocking needs the block's shapes, which aren't ported")
            }
        }
    }
}

/// `getShape(state, EmptyBlockGetter.INSTANCE, BlockPos.ZERO)`, as `initCache` asks it while the
/// blocks are still registering: the class's, `AirBlock`'s nothing, or `BlockBehaviour`'s whole
/// block. None for a class whose shapes aren't ported yet.
pub(crate) fn shape_in_empty_getter(block: &BlockData, state: BlockState) -> Option<VoxelShape> {
    let mut class = Some(block.class());
    while let Some(current) = class {
        if current.name == "net.minecraft.world.level.block.AirBlock" {
            return Some(Shapes::empty());
        }
        if let Some(get_shape) = current.get_shape {
            return Some(get_shape(
                block,
                state,
                &EmptyBlockGetter,
                &BlockPos::ZERO,
                &collision_context::empty(),
            ));
        }
        if SHAPE_OVERRIDES.contains(&current.name) {
            return None;
        }
        class = current.parent;
    }
    Some(Shapes::block())
}

/// The classes overriding `getOcclusionShape` whose overrides aren't ported yet.
const OCCLUSION_SHAPE_OVERRIDES: &[&str] = &[
    "net.minecraft.world.level.block.FenceGateBlock",
    "net.minecraft.world.level.block.LecternBlock",
    "net.minecraft.world.level.block.SculkShriekerBlock",
];

/// `getOcclusionShape(BlockState)`, as `initCache` asks it: the class's, or `BlockBehaviour`'s,
/// the shape in an empty getter. None for a class whose override or shapes aren't ported yet.
fn occlusion_shape_of(block: &BlockData, state: BlockState) -> Option<VoxelShape> {
    let mut class = Some(block.class());
    while let Some(current) = class {
        if let Some(get_occlusion_shape) = current.get_occlusion_shape {
            return Some(get_occlusion_shape(block, state));
        }
        if OCCLUSION_SHAPE_OVERRIDES.contains(&current.name) {
            return None;
        }
        class = current.parent;
    }
    shape_in_empty_getter(block, state)
}

/// The block classes that override `getShape` or `getCollisionShape` in Java, whose overrides
/// aren't ported yet. `AirBlock`'s is, and the classes' with a `BlockClass::get_shape` or
/// `get_collision_shape` (the player track's, for its physics scenarios): `SoulSandBlock`,
/// `SlabBlock`, `StairBlock` and `CrossCollisionBlock` (fences and panes).
const SHAPE_OVERRIDES: &[&str] = &[
    "net.minecraft.world.level.block.AbstractCauldronBlock",
    "net.minecraft.world.level.block.AmethystClusterBlock",
    "net.minecraft.world.level.block.AnvilBlock",
    "net.minecraft.world.level.block.AttachedStemBlock",
    "net.minecraft.world.level.block.AzaleaBlock",
    "net.minecraft.world.level.block.BambooSaplingBlock",
    "net.minecraft.world.level.block.BambooStalkBlock",
    "net.minecraft.world.level.block.BannerBlock",
    "net.minecraft.world.level.block.BaseCoralFanBlock",
    "net.minecraft.world.level.block.BaseCoralPlantBlock",
    "net.minecraft.world.level.block.BaseCoralPlantTypeBlock",
    "net.minecraft.world.level.block.BaseCoralWallFanBlock",
    "net.minecraft.world.level.block.BaseFireBlock",
    "net.minecraft.world.level.block.BasePressurePlateBlock",
    "net.minecraft.world.level.block.BaseRailBlock",
    "net.minecraft.world.level.block.BaseTorchBlock",
    "net.minecraft.world.level.block.BedBlock",
    "net.minecraft.world.level.block.BeetrootBlock",
    "net.minecraft.world.level.block.BellBlock",
    "net.minecraft.world.level.block.BigDripleafBlock",
    "net.minecraft.world.level.block.BigDripleafStemBlock",
    "net.minecraft.world.level.block.BrewingStandBlock",
    "net.minecraft.world.level.block.BubbleColumnBlock",
    "net.minecraft.world.level.block.BushBlock",
    "net.minecraft.world.level.block.ButtonBlock",
    "net.minecraft.world.level.block.CactusBlock",
    "net.minecraft.world.level.block.CactusFlowerBlock",
    "net.minecraft.world.level.block.CakeBlock",
    "net.minecraft.world.level.block.CampfireBlock",
    "net.minecraft.world.level.block.CandleBlock",
    "net.minecraft.world.level.block.CandleCakeBlock",
    "net.minecraft.world.level.block.CarpetBlock",
    "net.minecraft.world.level.block.CarrotBlock",
    "net.minecraft.world.level.block.CeilingHangingSignBlock",
    "net.minecraft.world.level.block.ChainBlock",
    "net.minecraft.world.level.block.ChestBlock",
    "net.minecraft.world.level.block.CocoaBlock",
    "net.minecraft.world.level.block.ComposterBlock",
    "net.minecraft.world.level.block.ConduitBlock",
    "net.minecraft.world.level.block.CopperGolemStatueBlock",
    "net.minecraft.world.level.block.CoralPlantBlock",
    "net.minecraft.world.level.block.CropBlock",
    "net.minecraft.world.level.block.DaylightDetectorBlock",
    "net.minecraft.world.level.block.DecoratedPotBlock",
    "net.minecraft.world.level.block.DiodeBlock",
    "net.minecraft.world.level.block.DoorBlock",
    "net.minecraft.world.level.block.DragonEggBlock",
    "net.minecraft.world.level.block.DriedGhastBlock",
    "net.minecraft.world.level.block.DryVegetationBlock",
    "net.minecraft.world.level.block.EnchantingTableBlock",
    "net.minecraft.world.level.block.EndPortalBlock",
    "net.minecraft.world.level.block.EndPortalFrameBlock",
    "net.minecraft.world.level.block.EnderChestBlock",
    "net.minecraft.world.level.block.FarmlandBlock",
    "net.minecraft.world.level.block.FenceGateBlock",
    "net.minecraft.world.level.block.FireBlock",
    "net.minecraft.world.level.block.FlowerBedBlock",
    "net.minecraft.world.level.block.FlowerBlock",
    "net.minecraft.world.level.block.FlowerPotBlock",
    "net.minecraft.world.level.block.FrogspawnBlock",
    "net.minecraft.world.level.block.GrindstoneBlock",
    "net.minecraft.world.level.block.GrowingPlantBlock",
    "net.minecraft.world.level.block.HangingMossBlock",
    "net.minecraft.world.level.block.HangingRootsBlock",
    "net.minecraft.world.level.block.HeavyCoreBlock",
    "net.minecraft.world.level.block.HoneyBlock",
    "net.minecraft.world.level.block.HopperBlock",
    "net.minecraft.world.level.block.LadderBlock",
    "net.minecraft.world.level.block.LanternBlock",
    "net.minecraft.world.level.block.LeafLitterBlock",
    "net.minecraft.world.level.block.LecternBlock",
    "net.minecraft.world.level.block.LeverBlock",
    "net.minecraft.world.level.block.LightBlock",
    "net.minecraft.world.level.block.LilyPadBlock",
    "net.minecraft.world.level.block.LiquidBlock",
    "net.minecraft.world.level.block.MangrovePropaguleBlock",
    "net.minecraft.world.level.block.MossyCarpetBlock",
    "net.minecraft.world.level.block.MudBlock",
    "net.minecraft.world.level.block.MultifaceBlock",
    "net.minecraft.world.level.block.MushroomBlock",
    "net.minecraft.world.level.block.NetherFungusBlock",
    "net.minecraft.world.level.block.NetherPortalBlock",
    "net.minecraft.world.level.block.NetherRootsBlock",
    "net.minecraft.world.level.block.NetherSproutsBlock",
    "net.minecraft.world.level.block.NetherWartBlock",
    "net.minecraft.world.level.block.PathBlock",
    "net.minecraft.world.level.block.PiglinWallSkullBlock",
    "net.minecraft.world.level.block.PipeBlock",
    "net.minecraft.world.level.block.PitcherCropBlock",
    "net.minecraft.world.level.block.PotatoBlock",
    "net.minecraft.world.level.block.PowderSnowBlock",
    "net.minecraft.world.level.block.RedstoneWallTorchBlock",
    "net.minecraft.world.level.block.RedstoneWireBlock",
    "net.minecraft.world.level.block.RodBlock",
    "net.minecraft.world.level.block.SaplingBlock",
    "net.minecraft.world.level.block.ScaffoldingBlock",
    "net.minecraft.world.level.block.SculkSensorBlock",
    "net.minecraft.world.level.block.SculkShriekerBlock",
    "net.minecraft.world.level.block.SeaPickleBlock",
    "net.minecraft.world.level.block.SeagrassBlock",
    "net.minecraft.world.level.block.ShelfBlock",
    "net.minecraft.world.level.block.ShelfMushroomBlock",
    "net.minecraft.world.level.block.ShortDryGrassBlock",
    "net.minecraft.world.level.block.ShulkerBoxBlock",
    "net.minecraft.world.level.block.SignBlock",
    "net.minecraft.world.level.block.SkullBlock",
    "net.minecraft.world.level.block.SmallDripleafBlock",
    "net.minecraft.world.level.block.SnifferEggBlock",
    "net.minecraft.world.level.block.SnowLayerBlock",
    "net.minecraft.world.level.block.SpeleothemBlock",
    "net.minecraft.world.level.block.SporeBlossomBlock",
    "net.minecraft.world.level.block.StemBlock",
    "net.minecraft.world.level.block.StonecutterBlock",
    "net.minecraft.world.level.block.StrawBedBlock",
    "net.minecraft.world.level.block.StructureVoidBlock",
    "net.minecraft.world.level.block.SugarCaneBlock",
    "net.minecraft.world.level.block.SweetBerryBushBlock",
    "net.minecraft.world.level.block.TallDryGrassBlock",
    "net.minecraft.world.level.block.TallGrassBlock",
    "net.minecraft.world.level.block.TallSeagrassBlock",
    "net.minecraft.world.level.block.TorchflowerCropBlock",
    "net.minecraft.world.level.block.TrapDoorBlock",
    "net.minecraft.world.level.block.TripWireBlock",
    "net.minecraft.world.level.block.TripWireHookBlock",
    "net.minecraft.world.level.block.TurtleEggBlock",
    "net.minecraft.world.level.block.VineBlock",
    "net.minecraft.world.level.block.WallBannerBlock",
    "net.minecraft.world.level.block.WallBlock",
    "net.minecraft.world.level.block.WallHangingSignBlock",
    "net.minecraft.world.level.block.WallSignBlock",
    "net.minecraft.world.level.block.WallSkullBlock",
    "net.minecraft.world.level.block.WallTorchBlock",
    "net.minecraft.world.level.block.piston.MovingPistonBlock",
    "net.minecraft.world.level.block.piston.PistonBaseBlock",
    "net.minecraft.world.level.block.piston.PistonHeadBlock",
];

/// The shapes, which go by the block's class.
impl BlockState {
    /// The class's override of a shape method, looked up the chain like Java's dispatch, or none
    /// for `BlockBehaviour`'s own. Panics at a class whose shapes aren't ported yet.
    fn shape_override(
        &self,
        method: impl Fn(&BlockClass) -> Option<ShapeMethod>,
    ) -> Option<ShapeMethod> {
        let mut class = Some(Block::data(self.get_block()).class());
        while let Some(current) = class {
            if let Some(found) = method(current) {
                return Some(found);
            }
            assert!(
                !SHAPE_OVERRIDES.contains(&current.name),
                "the shapes of {} ({}) aren't ported",
                self.get_block(),
                current.name
            );
            class = current.parent;
        }
        None
    }

    fn is_air_block(&self) -> bool {
        let mut class = Some(Block::data(self.get_block()).class());
        while let Some(current) = class {
            if current.name == "net.minecraft.world.level.block.AirBlock" {
                return true;
            }
            class = current.parent;
        }
        false
    }

    /// `getShape(BlockGetter, BlockPos, CollisionContext)`: the class's, `AirBlock`'s nothing, or
    /// `BlockBehaviour`'s whole block.
    pub fn get_shape_in(
        &self,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        if self.is_air_block() {
            return Shapes::empty();
        }
        match self.shape_override(|class| class.get_shape) {
            Some(get_shape) => get_shape(Block::data(self.get_block()), *self, level, pos, context),
            None => Shapes::block(),
        }
    }

    /// `getShape(BlockGetter, BlockPos)`, with an empty context.
    pub fn get_shape(&self, level: &dyn BlockGetter, pos: &BlockPos) -> VoxelShape {
        self.get_shape_in(level, pos, &collision_context::empty())
    }

    /// `getCollisionShape(BlockGetter, BlockPos, CollisionContext)`: the class's, or
    /// `BlockBehaviour`'s, the shape (asked with an empty context), or nothing for a block
    /// without collision.
    pub fn get_collision_shape_in(
        &self,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        match self.shape_override(|class| class.get_collision_shape) {
            Some(get_collision_shape) => {
                get_collision_shape(Block::data(self.get_block()), *self, level, pos, context)
            }
            None if Block::data(self.get_block()).has_collision() => self.get_shape(level, pos),
            None => Shapes::empty(),
        }
    }

    /// `getCollisionShape(BlockGetter, BlockPos)`, with an empty context.
    pub fn get_collision_shape(&self, level: &dyn BlockGetter, pos: &BlockPos) -> VoxelShape {
        self.get_collision_shape_in(level, pos, &collision_context::empty())
    }

    pub fn is_collision_shape_full_block(&self, level: &dyn BlockGetter, pos: &BlockPos) -> bool {
        Block::is_shape_full_block(&self.get_collision_shape(level, pos))
    }

    /// Whether the collision shape reaches past the block, which is always taken to be so for a
    /// block with a dynamic shape (no cache).
    pub fn has_large_collision_shape(&self) -> bool {
        if Block::data(self.get_block()).has_dynamic_shape() {
            return true;
        }
        let shape = self.get_collision_shape(&EmptyBlockGetter, &BlockPos::ZERO);
        Axis::VALUES
            .into_iter()
            .any(|axis| shape.min(axis) < 0.0 || shape.max(axis) > 1.0)
    }

    /// `isSuffocating(BlockGetter, BlockPos)`: Properties' default is a block in the
    /// `causes_suffocation` tag whose collision shape is the whole block.
    pub fn is_suffocating_at(&self, level: &dyn BlockGetter, pos: &BlockPos) -> bool {
        match self.base().is_suffocating {
            StatePredicate::State(_, test) => test(*self),
            StatePredicate::Default => {
                self.is_tag(BlockTags::CAUSES_SUFFOCATION)
                    && self.is_collision_shape_full_block(level, pos)
            }
            StatePredicate::NotClosedShulker => {
                panic!("NOT_CLOSED_SHULKER asks the shulker box's block entity, which isn't ported")
            }
        }
    }
}

/// Block classes overriding what the renderer reads of a state in a way not ported yet, by the
/// method: they panic, like the shapes' guard.
const RENDER_OVERRIDES: &[(&str, &[&str])] = &[
    (
        "skipRendering",
        &[
            "net.minecraft.world.level.block.IronBarsBlock",
            "net.minecraft.world.level.block.LiquidBlock",
        ],
    ),
    (
        "getShadeBrightness",
        &["net.minecraft.world.level.block.SnowLayerBlock"],
    ),
    (
        "getRenderShape",
        &[
            "net.minecraft.world.level.block.BubbleColumnBlock",
            "net.minecraft.world.level.block.EndPortalBlock",
            "net.minecraft.world.level.block.EndGatewayBlock",
            "net.minecraft.world.level.block.piston.MovingPistonBlock",
        ],
    ),
];

/// What the renderer reads of a state, worked out from its block's class and shapes each time
/// it's asked, where Java keeps most of it from `initCache`: the same answers, since a state's
/// shapes never change.
impl BlockState {
    /// `getBlock() instanceof LeavesBlock`.
    pub fn class_is_leaves(&self) -> bool {
        self.class_is("net.minecraft.world.level.block.LeavesBlock")
    }

    /// Whether the block's class is `name` or extends it.
    fn class_is(&self, name: &str) -> bool {
        let mut class = Some(Block::data(self.get_block()).class());
        while let Some(current) = class {
            if current.name == name {
                return true;
            }
            class = current.parent;
        }
        false
    }

    fn check_render_ported(&self, method: &str) {
        let overriding = RENDER_OVERRIDES
            .iter()
            .find(|(name, _)| *name == method)
            .map_or(&[][..], |(_, classes)| *classes);
        let mut class = Some(Block::data(self.get_block()).class());
        while let Some(current) = class {
            assert!(
                !overriding.contains(&current.name),
                "{method} of {} ({}) isn't ported",
                self.get_block(),
                current.name
            );
            class = current.parent;
        }
    }

    pub fn is_light_permeable(&self) -> bool {
        !self.is_solid_render() || self.get_light_dampening() == 0
    }

    pub fn get_shade_brightness(&self, level: &dyn BlockGetter, pos: &BlockPos) -> f32 {
        self.check_render_ported("getShadeBrightness");
        if self.class_is("net.minecraft.world.level.block.TransparentBlock")
            || self.class_is("net.minecraft.world.level.block.BarrierBlock")
            || self.class_is("net.minecraft.world.level.block.LightBlock")
            || self.class_is("net.minecraft.world.level.block.StructureVoidBlock")
        {
            return 1.0;
        }
        if self.class_is("net.minecraft.world.level.block.MudBlock")
            || self.class_is("net.minecraft.world.level.block.SoulSandBlock")
        {
            return 0.2;
        }
        if self.is_collision_shape_full_block(level, pos) {
            0.2
        } else {
            1.0
        }
    }

    pub fn get_render_shape(&self) -> RenderShape {
        self.check_render_ported("getRenderShape");
        let invisible = [
            "net.minecraft.world.level.block.AirBlock",
            "net.minecraft.world.level.block.BarrierBlock",
            "net.minecraft.world.level.block.LiquidBlock",
            "net.minecraft.world.level.block.LightBlock",
            "net.minecraft.world.level.block.StructureVoidBlock",
        ];
        if invisible.iter().any(|class| self.class_is(class)) {
            RenderShape::INVISIBLE
        } else {
            RenderShape::MODEL
        }
    }

    /// `skipRendering`: whether a face toward this neighbour is left out whatever the shapes say.
    pub fn skip_rendering(&self, neighbor_state: BlockState, direction: Direction) -> bool {
        self.check_render_ported("skipRendering");
        if self.class_is("net.minecraft.world.level.block.HalfTransparentBlock")
            || self.class_is("net.minecraft.world.level.block.PowderSnowBlock")
        {
            return neighbor_state.is(self.get_block());
        }
        if self.class_is("net.minecraft.world.level.block.LeavesBlock") {
            return !crate::net::minecraft::world::level::block::leaves_block::LeavesBlock::cutout_leaves()
                && neighbor_state.class_is("net.minecraft.world.level.block.LeavesBlock");
        }
        if self.class_is("net.minecraft.world.level.block.MangroveRootsBlock") {
            return neighbor_state
                .is(crate::net::minecraft::world::level::block::blocks::Blocks::MANGROVE_ROOTS)
                && direction.get_axis() == Axis::Y;
        }
        false
    }

    /// `getSeed`: the position's, which the beds, doors and double plants take from the half the
    /// two share.
    pub fn get_seed(&self, pos: &BlockPos) -> i64 {
        assert!(
            !self.class_is("net.minecraft.world.level.block.AbstractBedBlock")
                && !self.class_is("net.minecraft.world.level.block.DoorBlock")
                && !self.class_is("net.minecraft.world.level.block.DoublePlantBlock"),
            "getSeed of {} isn't ported",
            self.get_block()
        );
        crate::net::minecraft::util::mth::Mth::get_seed_vec3i(pos)
    }

    /// `getOffset`: nothing for a block without an offset function, whose functions aren't
    /// ported yet.
    pub fn get_offset(&self, _pos: &BlockPos) -> crate::net::minecraft::world::phys::vec3::Vec3 {
        assert!(
            !self.has_offset_function(),
            "the offset function of {} isn't ported",
            self.get_block()
        );
        crate::net::minecraft::world::phys::vec3::Vec3::ZERO
    }
}

/// `RenderShape`: whether a state's model is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderShape {
    INVISIBLE,
    MODEL,
}

/// `EmptyBlockGetter.INSTANCE`: air everywhere, which shapes are cached with.
struct EmptyBlockGetter;

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
            StatePredicate::Default | StatePredicate::NotClosedShulker => {
                panic!("this predicate needs the level or the block's shapes, which aren't ported")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EmptyBlockGetter;
    use crate::golden::Golden;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;

    /// The shapes of every state of the blocks whose classes' shapes are ported, as a state
    /// reports them with no level around it (`ShapeSuites.BlockStateShapeSuite`).
    #[test]
    fn state_shapes_match_vanilla() {
        let golden =
            Golden::load("net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase");
        golden.check(
            "getShape(Lnet/minecraft/world/level/BlockGetter;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/shapes/VoxelShape;",
            |state: BlockState| state.get_shape(&EmptyBlockGetter, &BlockPos::ZERO),
        );
        golden.check(
            "getCollisionShape(Lnet/minecraft/world/level/BlockGetter;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/shapes/VoxelShape;",
            |state: BlockState| state.get_collision_shape(&EmptyBlockGetter, &BlockPos::ZERO),
        );
        golden.finish();
    }
}
