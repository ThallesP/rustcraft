//! net/minecraft/world/level/block/Block.java
//!
//! Only what makes a block and its states, and what it keeps from its `Properties`: its state
//! definition and default state, `BLOCK_STATE_REGISTRY` with `getId` and `stateById`, the
//! getters of what `BlockBehaviour` copied (`getFriction`, `getExplosionResistance`…),
//! `toString`, and the shapes' helpers (`box`, `column`, `boxZ`, `cube`, `isShapeFullBlock`,
//! `getShapeForEachState`: the player track's), `stepOn`, which a class overrides like the
//! shapes, and what shape updates use (`updateOrDestroy`, `updateFromNeighbourShapes`,
//! `isExceptionForConnection`, and the dispatch of `updateShape`, `canSurvive` and the like to
//! each class's). Not ported yet: the other behaviour methods (`playerDestroy`,
//! `getStateForPlacement`…), `shouldRenderFace`, `asItem`, `getName` and `withPropertiesOf`,
//! which come with what calls them. `fallOn` and `bounceOn` change nothing the client has: a
//! landing's damage is the server's (`Entity.hurt` does nothing on the client), and their sounds
//! aren't ported.
//!
//! Java's blocks are objects the game compares by reference, so here a `Block` is its id in
//! `BuiltInRegistries.BLOCK`: a `Copy` handle that derefs to what Java's object holds, a
//! `BlockData`, once `Blocks` has registered them all. A class extending `Block` is a
//! `BlockClass`: its name, its parent and the methods it overrides (so far only those that decide
//! what a state keeps, like `isRandomlyTicking`), which a call looks up the chain like Java's
//! dispatch. Its constructors make a `BlockData`, the way Java's run their parents' first: the
//! most derived class and its `createBlockStateDefinition` go down the chain to `Block::new_with`.

use std::any::Any;
use std::fmt;
use std::ops::Deref;

use crate::net::minecraft::client::multiplayer::client_level::{
    ClientLevel, LevelTicking, LevelWithEffects,
};
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::id_mapper::IdMapper;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::entity::inside_block_effect_applier::InsideBlockEffectApplier;
use crate::net::minecraft::world::level::block::blocks::{self, Blocks, REGISTRY};
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::state::block_behaviour::{
    BlockBehaviour, EmptyBlockGetter, Properties, RenderShape,
};
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::property::Property;
use crate::net::minecraft::world::level::block::state::state_definition::{
    Builder, StateDefinition,
};
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::level::material::map_color::MapColor;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// A block, by its id in `BuiltInRegistries.BLOCK`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block(pub(crate) u16);

/// A class's `createBlockStateDefinition`.
pub type CreateBlockStateDefinition = fn(&mut Builder);

/// A class's `getShape`, `getCollisionShape` or `getVisualShape`: the shape of its block's state at
/// a position of a level, for what asks.
pub type ShapeMethod =
    fn(&BlockData, BlockState, &dyn BlockGetter, &BlockPos, &dyn CollisionContext) -> VoxelShape;

/// A class's `isCollisionShapeFullBlock`.
pub type PositionPredicate = fn(&BlockData, BlockState, &dyn BlockGetter, &BlockPos) -> bool;

/// A class's `getShadeBrightness`.
pub type ShadeBrightnessMethod = fn(&BlockData, BlockState, &dyn BlockGetter, &BlockPos) -> f32;

/// A class's `skipRendering`: of the state, against its neighbour that way.
pub type SkipRenderingMethod = fn(&BlockData, BlockState, BlockState, Direction) -> bool;

/// A class's `getBlockSupportShape` or `getInteractionShape`, which nothing asks for.
pub type PositionShapeMethod =
    fn(&BlockData, BlockState, &dyn BlockGetter, &BlockPos) -> VoxelShape;

/// A class's `animateTick`: the particles and sounds its block makes now and then, from the
/// level's animations' random source.
pub type AnimateTickMethod =
    fn(&BlockData, BlockState, &LevelWithEffects<'_>, &BlockPos, &mut dyn RandomSource);

/// A class's `triggerEvent`: what a block event does to its block, whether it was handled.
pub type TriggerEventMethod =
    fn(&BlockData, BlockState, &mut LevelTicking<'_>, &BlockPos, i32, i32) -> bool;

/// A class's `updateShape`: the state, given that its neighbour that way is now the neighbour's
/// state, with the level's random. Java also hands it the level's scheduled ticks, which on the
/// client go nowhere (`BlackholeTickAccess`), so they aren't passed: an override's
/// `scheduleTick` is left out, and only what it draws from the random for one is kept.
pub type UpdateShapeMethod = fn(
    &BlockData,
    BlockState,
    &dyn BlockGetter,
    &BlockPos,
    Direction,
    &BlockPos,
    BlockState,
    &mut dyn RandomSource,
) -> BlockState;

/// A class's `canSurvive`, or `VegetationBlock`'s `mayPlaceOn`: whether its block's state can stay
/// at a position of a level (for `mayPlaceOn`, on the state there).
pub type PositionStatePredicate = fn(&BlockData, BlockState, &dyn BlockGetter, &BlockPos) -> bool;

/// A class's `shouldRedstoneWireConnectTo`: whether wire beside its block's state, that way (none
/// for wire above or below), joins it.
pub type ShouldRedstoneWireConnectToMethod =
    fn(&BlockData, BlockState, &dyn BlockGetter, &BlockPos, Option<Direction>) -> bool;

/// A class's `updateIndirectNeighbourShapes`: the shape updates its block sends past its six
/// neighbours, through the level.
pub type UpdateIndirectNeighbourShapesMethod =
    fn(&BlockData, BlockState, &mut ClientLevel, &BlockPos, i32, i32);

/// A class's `getTicker(level, state, type)` on the client, an `EntityBlock`'s: what its block
/// entity of the type does each tick there, if anything.
pub type GetTickerMethod = fn(&BlockData, BlockState, BlockEntityType) -> Option<BlockEntityTicker>;

/// A class's `getEntityInsideCollisionShape`: what of the block an entity has to touch to be in
/// it.
pub type EntityInsideShapeMethod =
    fn(&BlockData, BlockState, &dyn BlockGetter, &BlockPos, &dyn EntityClass) -> VoxelShape;

/// A class's `entityInside`: what the block does to an entity inside it, directly or through the
/// applier, and whether the entity is precisely in it (`isPrecise`).
pub type EntityInsideMethod = fn(
    &BlockData,
    BlockState,
    &dyn Level,
    &BlockPos,
    &mut dyn EntityClass,
    &mut dyn InsideBlockEffectApplier,
    bool,
);

/// A class's `stepOn`: what the block does to an entity on the ground on it.
pub type StepOnMethod = fn(&BlockData, &dyn Level, &BlockPos, BlockState, &mut dyn EntityClass);

/// A block class: Java's `Class<? extends Block>` and the overrides that matter so far.
pub struct BlockClass {
    /// `getName()`, the binary name.
    pub name: &'static str,
    pub parent: Option<&'static BlockClass>,
    /// Whether it implements `EntityBlock` itself.
    pub entity_block: bool,
    /// `isRandomlyTicking(BlockState)`, if it overrides it.
    pub is_randomly_ticking: Option<fn(&BlockData, BlockState) -> bool>,
    /// `getFluidState(BlockState)`, if it overrides it.
    pub get_fluid_state: Option<fn(&BlockData, BlockState) -> FluidState>,
    /// `getShape(BlockState, BlockGetter, BlockPos, CollisionContext)`, if it overrides it.
    pub get_shape: Option<ShapeMethod>,
    /// `getCollisionShape(BlockState, BlockGetter, BlockPos, CollisionContext)`, if it overrides
    /// it.
    pub get_collision_shape: Option<ShapeMethod>,
    /// `useShapeForLightOcclusion(BlockState)`, if it overrides it.
    pub use_shape_for_light_occlusion: Option<fn(&BlockData, BlockState) -> bool>,
    /// `propagatesSkylightDown(BlockState)`, if it overrides it.
    pub propagates_skylight_down: Option<fn(&BlockData, BlockState) -> bool>,
    /// `getLightDampening(BlockState)`, if it overrides it.
    pub get_light_dampening: Option<fn(&BlockData, BlockState) -> i32>,
    /// `getOcclusionShape(BlockState)`, if it overrides it.
    pub get_occlusion_shape: Option<fn(&BlockData, BlockState) -> VoxelShape>,
    /// `getBlockSupportShape(BlockState, BlockGetter, BlockPos)`, if it overrides it.
    pub get_block_support_shape: Option<PositionShapeMethod>,
    /// `getInteractionShape(BlockState, BlockGetter, BlockPos)`, if it overrides it.
    pub get_interaction_shape: Option<PositionShapeMethod>,
    /// `getVisualShape(BlockState, BlockGetter, BlockPos, CollisionContext)`, if it overrides it.
    pub get_visual_shape: Option<ShapeMethod>,
    /// `isCollisionShapeFullBlock(BlockState, BlockGetter, BlockPos)`, if it overrides it.
    pub is_collision_shape_full_block: Option<PositionPredicate>,
    /// `getShadeBrightness(BlockState, BlockGetter, BlockPos)`, if it overrides it.
    pub get_shade_brightness: Option<ShadeBrightnessMethod>,
    /// `skipRendering(BlockState, BlockState, Direction)`, if it overrides it.
    pub skip_rendering: Option<SkipRenderingMethod>,
    /// `getRenderShape(BlockState)`, if it overrides it.
    pub get_render_shape: Option<fn(&BlockData, BlockState) -> RenderShape>,
    /// `getMaxHorizontalOffset()`, if it overrides it.
    pub get_max_horizontal_offset: Option<fn(&BlockData) -> f32>,
    /// `getMaxVerticalOffset()`, if it overrides it.
    pub get_max_vertical_offset: Option<fn(&BlockData) -> f32>,
    /// `getSeed(BlockState, BlockPos)`, if it overrides it.
    pub get_seed: Option<fn(&BlockData, BlockState, &BlockPos) -> i64>,
    /// `animateTick(BlockState, Level, BlockPos, RandomSource)`, if it overrides it.
    pub animate_tick: Option<AnimateTickMethod>,
    /// `triggerEvent(BlockState, Level, BlockPos, int, int)`, if it overrides it.
    pub trigger_event: Option<TriggerEventMethod>,
    /// `EntityBlock.getTicker(Level, BlockState, BlockEntityType)`, if it overrides it.
    pub get_ticker: Option<GetTickerMethod>,
    /// `updateShape(…)`, if it overrides it and the override is ported (`UPDATE_SHAPE_CLASSES`
    /// names those that override it in Java).
    pub update_shape: Option<UpdateShapeMethod>,
    /// `updateIndirectNeighbourShapes(...)`, if it overrides it.
    pub update_indirect_neighbour_shapes: Option<UpdateIndirectNeighbourShapesMethod>,
    /// `canSurvive(BlockState, LevelReader, BlockPos)`, if it overrides it and the override is
    /// ported (`CAN_SURVIVE_CLASSES` names those that override it in Java).
    pub can_survive: Option<PositionStatePredicate>,
    /// `VegetationBlock.mayPlaceOn(BlockState, BlockGetter, BlockPos)`, if it overrides it and
    /// the override is ported (`MAY_PLACE_ON_CLASSES` names those that override it in Java).
    pub may_place_on: Option<PositionStatePredicate>,
    /// `isSignalSource(BlockState)`, if it overrides it.
    pub is_signal_source: Option<fn(&BlockData, BlockState) -> bool>,
    /// `shouldRedstoneWireConnectTo(...)`, if it overrides it.
    pub should_redstone_wire_connect_to: Option<ShouldRedstoneWireConnectToMethod>,
    /// `getEntityInsideCollisionShape(BlockState, BlockGetter, BlockPos, Entity)`, if it overrides
    /// it.
    pub get_entity_inside_collision_shape: Option<EntityInsideShapeMethod>,
    /// `entityInside(BlockState, Level, BlockPos, Entity, InsideBlockEffectApplier, boolean)`, if
    /// it overrides it with something the client does.
    pub entity_inside: Option<EntityInsideMethod>,
    /// `stepOn(Level, BlockPos, BlockState, Entity)`, if it overrides it with something the client
    /// does.
    pub step_on: Option<StepOnMethod>,
}

impl BlockClass {
    /// A class overriding nothing so far.
    pub const fn new(name: &'static str, parent: &'static BlockClass) -> BlockClass {
        BlockClass {
            name,
            parent: Some(parent),
            entity_block: false,
            is_randomly_ticking: None,
            get_fluid_state: None,
            get_shape: None,
            get_collision_shape: None,
            use_shape_for_light_occlusion: None,
            propagates_skylight_down: None,
            get_light_dampening: None,
            get_occlusion_shape: None,
            get_block_support_shape: None,
            get_interaction_shape: None,
            get_visual_shape: None,
            is_collision_shape_full_block: None,
            get_shade_brightness: None,
            skip_rendering: None,
            get_render_shape: None,
            get_max_horizontal_offset: None,
            get_max_vertical_offset: None,
            get_seed: None,
            animate_tick: None,
            trigger_event: None,
            get_ticker: None,
            update_shape: None,
            update_indirect_neighbour_shapes: None,
            can_survive: None,
            may_place_on: None,
            is_signal_source: None,
            should_redstone_wire_connect_to: None,
            get_entity_inside_collision_shape: None,
            entity_inside: None,
            step_on: None,
        }
    }

    /// `instanceof EntityBlock`: it or a parent implements it.
    pub fn is_entity_block(&self) -> bool {
        self.entity_block || self.parent.is_some_and(BlockClass::is_entity_block)
    }

    /// The override of a method nearest this class, looked up the chain like Java's dispatch: a
    /// class's own, else its parent's. None for `BlockBehaviour`'s own.
    pub fn find<T>(&'static self, method: impl Fn(&'static BlockClass) -> Option<T>) -> Option<T> {
        method(self).or_else(|| self.parent.and_then(|parent| parent.find(method)))
    }

    /// The parent's override of a method, for an override calling `super`'s.
    pub fn find_super<T>(
        &'static self,
        method: impl Fn(&'static BlockClass) -> Option<T>,
    ) -> Option<T> {
        self.parent.and_then(|parent| parent.find(method))
    }
}

/// The classes that override `updateShape` in Java, by binary name: where the port has no
/// override, `update_shape` says it isn't ported.
const UPDATE_SHAPE_CLASSES: [&str; 109] = [
    "net.minecraft.world.level.block.AbstractBedBlock",
    "net.minecraft.world.level.block.AmethystClusterBlock",
    "net.minecraft.world.level.block.AttachedStemBlock",
    "net.minecraft.world.level.block.BambooSaplingBlock",
    "net.minecraft.world.level.block.BambooStalkBlock",
    "net.minecraft.world.level.block.BannerBlock",
    "net.minecraft.world.level.block.BarrierBlock",
    "net.minecraft.world.level.block.BaseCoralPlantTypeBlock",
    "net.minecraft.world.level.block.BaseCoralWallFanBlock",
    "net.minecraft.world.level.block.BasePressurePlateBlock",
    "net.minecraft.world.level.block.BaseRailBlock",
    "net.minecraft.world.level.block.BaseTorchBlock",
    "net.minecraft.world.level.block.BeehiveBlock",
    "net.minecraft.world.level.block.BellBlock",
    "net.minecraft.world.level.block.BigDripleafBlock",
    "net.minecraft.world.level.block.BigDripleafStemBlock",
    "net.minecraft.world.level.block.BrushableBlock",
    "net.minecraft.world.level.block.BubbleColumnBlock",
    "net.minecraft.world.level.block.CactusBlock",
    "net.minecraft.world.level.block.CakeBlock",
    "net.minecraft.world.level.block.CampfireBlock",
    "net.minecraft.world.level.block.CandleBlock",
    "net.minecraft.world.level.block.CandleCakeBlock",
    "net.minecraft.world.level.block.CarpetBlock",
    "net.minecraft.world.level.block.CeilingHangingSignBlock",
    "net.minecraft.world.level.block.ChainBlock",
    "net.minecraft.world.level.block.ChestBlock",
    "net.minecraft.world.level.block.ChorusFlowerBlock",
    "net.minecraft.world.level.block.ChorusPlantBlock",
    "net.minecraft.world.level.block.CocoaBlock",
    "net.minecraft.world.level.block.ComparatorBlock",
    "net.minecraft.world.level.block.ConcretePowderBlock",
    "net.minecraft.world.level.block.ConduitBlock",
    "net.minecraft.world.level.block.CopperChestBlock",
    "net.minecraft.world.level.block.CopperGolemStatueBlock",
    "net.minecraft.world.level.block.CoralBlock",
    "net.minecraft.world.level.block.CoralFanBlock",
    "net.minecraft.world.level.block.CoralPlantBlock",
    "net.minecraft.world.level.block.CoralWallFanBlock",
    "net.minecraft.world.level.block.CreakingHeartBlock",
    "net.minecraft.world.level.block.DecoratedPotBlock",
    "net.minecraft.world.level.block.DoorBlock",
    "net.minecraft.world.level.block.DoublePlantBlock",
    "net.minecraft.world.level.block.DriedGhastBlock",
    "net.minecraft.world.level.block.EnderChestBlock",
    "net.minecraft.world.level.block.FaceAttachedHorizontalDirectionalBlock",
    "net.minecraft.world.level.block.FallingBlock",
    "net.minecraft.world.level.block.FarmlandBlock",
    "net.minecraft.world.level.block.FenceBlock",
    "net.minecraft.world.level.block.FenceGateBlock",
    "net.minecraft.world.level.block.FireBlock",
    "net.minecraft.world.level.block.FlowerPotBlock",
    "net.minecraft.world.level.block.FrogspawnBlock",
    "net.minecraft.world.level.block.GrowingPlantBodyBlock",
    "net.minecraft.world.level.block.GrowingPlantHeadBlock",
    "net.minecraft.world.level.block.HangingMossBlock",
    "net.minecraft.world.level.block.HangingRootsBlock",
    "net.minecraft.world.level.block.HeavyCoreBlock",
    "net.minecraft.world.level.block.HugeMushroomBlock",
    "net.minecraft.world.level.block.IronBarsBlock",
    "net.minecraft.world.level.block.LadderBlock",
    "net.minecraft.world.level.block.LanternBlock",
    "net.minecraft.world.level.block.LeavesBlock",
    "net.minecraft.world.level.block.LightBlock",
    "net.minecraft.world.level.block.LightningRodBlock",
    "net.minecraft.world.level.block.LiquidBlock",
    "net.minecraft.world.level.block.MangrovePropaguleBlock",
    "net.minecraft.world.level.block.MangroveRootsBlock",
    "net.minecraft.world.level.block.MossyCarpetBlock",
    "net.minecraft.world.level.block.MultifaceBlock",
    "net.minecraft.world.level.block.NetherPortalBlock",
    "net.minecraft.world.level.block.NoteBlock",
    "net.minecraft.world.level.block.ObserverBlock",
    "net.minecraft.world.level.block.PathBlock",
    "net.minecraft.world.level.block.PitcherCropBlock",
    "net.minecraft.world.level.block.PotentSulfurBlock",
    "net.minecraft.world.level.block.RedstoneWallTorchBlock",
    "net.minecraft.world.level.block.RedstoneWireBlock",
    "net.minecraft.world.level.block.RepeaterBlock",
    "net.minecraft.world.level.block.ScaffoldingBlock",
    "net.minecraft.world.level.block.SculkSensorBlock",
    "net.minecraft.world.level.block.SculkShriekerBlock",
    "net.minecraft.world.level.block.SeaPickleBlock",
    "net.minecraft.world.level.block.SeagrassBlock",
    "net.minecraft.world.level.block.ShelfBlock",
    "net.minecraft.world.level.block.ShelfMushroomBlock",
    "net.minecraft.world.level.block.SignBlock",
    "net.minecraft.world.level.block.SlabBlock",
    "net.minecraft.world.level.block.SmallDripleafBlock",
    "net.minecraft.world.level.block.SnowLayerBlock",
    "net.minecraft.world.level.block.SnowyBlock",
    "net.minecraft.world.level.block.SoulFireBlock",
    "net.minecraft.world.level.block.SpeleothemBlock",
    "net.minecraft.world.level.block.SporeBlossomBlock",
    "net.minecraft.world.level.block.StairBlock",
    "net.minecraft.world.level.block.StandingSignBlock",
    "net.minecraft.world.level.block.SugarCaneBlock",
    "net.minecraft.world.level.block.TrapDoorBlock",
    "net.minecraft.world.level.block.TripWireBlock",
    "net.minecraft.world.level.block.TripWireHookBlock",
    "net.minecraft.world.level.block.VegetationBlock",
    "net.minecraft.world.level.block.VineBlock",
    "net.minecraft.world.level.block.WallBannerBlock",
    "net.minecraft.world.level.block.WallBlock",
    "net.minecraft.world.level.block.WallHangingSignBlock",
    "net.minecraft.world.level.block.WallSignBlock",
    "net.minecraft.world.level.block.WallTorchBlock",
    "net.minecraft.world.level.block.WaterloggedTransparentBlock",
    "net.minecraft.world.level.block.piston.PistonHeadBlock",
];

/// The classes that override `canSurvive` in Java, by binary name: where the port has no
/// override, `can_survive` says it isn't ported.
const CAN_SURVIVE_CLASSES: [&str; 62] = [
    "net.minecraft.world.level.block.AmethystClusterBlock",
    "net.minecraft.world.level.block.BambooSaplingBlock",
    "net.minecraft.world.level.block.BambooStalkBlock",
    "net.minecraft.world.level.block.BannerBlock",
    "net.minecraft.world.level.block.BaseCoralPlantTypeBlock",
    "net.minecraft.world.level.block.BaseCoralWallFanBlock",
    "net.minecraft.world.level.block.BasePressurePlateBlock",
    "net.minecraft.world.level.block.BaseRailBlock",
    "net.minecraft.world.level.block.BaseTorchBlock",
    "net.minecraft.world.level.block.BellBlock",
    "net.minecraft.world.level.block.BigDripleafBlock",
    "net.minecraft.world.level.block.BigDripleafStemBlock",
    "net.minecraft.world.level.block.BubbleColumnBlock",
    "net.minecraft.world.level.block.CactusBlock",
    "net.minecraft.world.level.block.CakeBlock",
    "net.minecraft.world.level.block.CandleBlock",
    "net.minecraft.world.level.block.CandleCakeBlock",
    "net.minecraft.world.level.block.CarpetBlock",
    "net.minecraft.world.level.block.CeilingHangingSignBlock",
    "net.minecraft.world.level.block.ChorusFlowerBlock",
    "net.minecraft.world.level.block.ChorusPlantBlock",
    "net.minecraft.world.level.block.CocoaBlock",
    "net.minecraft.world.level.block.CropBlock",
    "net.minecraft.world.level.block.DiodeBlock",
    "net.minecraft.world.level.block.DoorBlock",
    "net.minecraft.world.level.block.DoublePlantBlock",
    "net.minecraft.world.level.block.FaceAttachedHorizontalDirectionalBlock",
    "net.minecraft.world.level.block.FarmlandBlock",
    "net.minecraft.world.level.block.FireBlock",
    "net.minecraft.world.level.block.FrogspawnBlock",
    "net.minecraft.world.level.block.GrindstoneBlock",
    "net.minecraft.world.level.block.GrowingPlantBlock",
    "net.minecraft.world.level.block.HangingMossBlock",
    "net.minecraft.world.level.block.HangingRootsBlock",
    "net.minecraft.world.level.block.LadderBlock",
    "net.minecraft.world.level.block.LanternBlock",
    "net.minecraft.world.level.block.LeafLitterBlock",
    "net.minecraft.world.level.block.MangrovePropaguleBlock",
    "net.minecraft.world.level.block.MossyCarpetBlock",
    "net.minecraft.world.level.block.MultifaceBlock",
    "net.minecraft.world.level.block.MushroomBlock",
    "net.minecraft.world.level.block.PathBlock",
    "net.minecraft.world.level.block.RedstoneWallTorchBlock",
    "net.minecraft.world.level.block.RedstoneWireBlock",
    "net.minecraft.world.level.block.ScaffoldingBlock",
    "net.minecraft.world.level.block.SeaPickleBlock",
    "net.minecraft.world.level.block.ShelfMushroomBlock",
    "net.minecraft.world.level.block.SmallDripleafBlock",
    "net.minecraft.world.level.block.SnowLayerBlock",
    "net.minecraft.world.level.block.SoulFireBlock",
    "net.minecraft.world.level.block.SpeleothemBlock",
    "net.minecraft.world.level.block.SporeBlossomBlock",
    "net.minecraft.world.level.block.StandingSignBlock",
    "net.minecraft.world.level.block.SugarCaneBlock",
    "net.minecraft.world.level.block.TallSeagrassBlock",
    "net.minecraft.world.level.block.TripWireHookBlock",
    "net.minecraft.world.level.block.VegetationBlock",
    "net.minecraft.world.level.block.VineBlock",
    "net.minecraft.world.level.block.WallBannerBlock",
    "net.minecraft.world.level.block.WallSignBlock",
    "net.minecraft.world.level.block.WallTorchBlock",
    "net.minecraft.world.level.block.piston.PistonHeadBlock",
];

/// The classes that override `VegetationBlock.mayPlaceOn` in Java, by binary name: where the port
/// has no override, `VegetationBlock::may_place_on` says it isn't ported.
pub(crate) const MAY_PLACE_ON_CLASSES: [&str; 19] = [
    "net.minecraft.world.level.block.AttachedStemBlock",
    "net.minecraft.world.level.block.AzaleaBlock",
    "net.minecraft.world.level.block.CactusFlowerBlock",
    "net.minecraft.world.level.block.CropBlock",
    "net.minecraft.world.level.block.DryVegetationBlock",
    "net.minecraft.world.level.block.LilyPadBlock",
    "net.minecraft.world.level.block.MangrovePropaguleBlock",
    "net.minecraft.world.level.block.MushroomBlock",
    "net.minecraft.world.level.block.NetherFungusBlock",
    "net.minecraft.world.level.block.NetherRootsBlock",
    "net.minecraft.world.level.block.NetherSproutsBlock",
    "net.minecraft.world.level.block.NetherWartBlock",
    "net.minecraft.world.level.block.PitcherCropBlock",
    "net.minecraft.world.level.block.SeaPickleBlock",
    "net.minecraft.world.level.block.SeagrassBlock",
    "net.minecraft.world.level.block.SmallDripleafBlock",
    "net.minecraft.world.level.block.StemBlock",
    "net.minecraft.world.level.block.TallSeagrassBlock",
    "net.minecraft.world.level.block.WitherRoseBlock",
];

/// `Block` itself.
pub static CLASS: BlockClass = BlockClass {
    parent: None,
    ..BlockClass::new("net.minecraft.world.level.block.Block", &CLASS)
};

/// What a Java `Block` object holds: `BlockBehaviour`'s fields, which it derefs to, its state
/// definition and default state, its class, and the fields its class adds that are ported.
pub struct BlockData {
    behaviour: BlockBehaviour,
    class: &'static BlockClass,
    state_definition: &'static StateDefinition,
    default_block_state: BlockState,
    fields: Box<dyn Any + Send + Sync>,
}

impl BlockData {
    pub fn class(&self) -> &'static BlockClass {
        self.class
    }

    pub fn get_state_definition(&self) -> &'static StateDefinition {
        self.state_definition
    }

    pub fn default_block_state(&self) -> BlockState {
        self.default_block_state
    }

    pub(crate) fn register_default_state(&mut self, state: BlockState) {
        self.default_block_state = state;
    }

    /// The fields its class adds, which it was made with (`set_fields`).
    pub fn fields<T: 'static>(&self) -> &T {
        self.fields
            .downcast_ref()
            .expect("a block's fields are its class's")
    }

    pub(crate) fn set_fields(&mut self, fields: impl Any + Send + Sync) {
        self.fields = Box::new(fields);
    }

    pub fn get_explosion_resistance(&self) -> f32 {
        self.explosion_resistance
    }

    pub fn get_bounce_restitution(&self) -> f32 {
        self.bounce_restitution
    }

    pub fn get_fall_distance_reduction(&self) -> f32 {
        self.fall_distance_reduction
    }

    pub fn get_friction(&self) -> f32 {
        self.friction
    }

    pub fn get_speed_factor(&self) -> f32 {
        self.speed_factor
    }

    pub fn get_jump_factor(&self) -> f32 {
        self.jump_factor
    }

    pub fn has_dynamic_shape(&self) -> bool {
        self.dynamic_shape
    }

    /// `BlockBehaviour.defaultMapColor()`: the default state's.
    pub fn default_map_color(&self) -> MapColor {
        (self.properties().map_color_of())(self.default_block_state)
    }

    /// `BlockBehaviour.getFluidState(BlockState)`, which a class may override: none.
    pub fn get_fluid_state_of(&self, state: BlockState) -> FluidState {
        match self.class.find(|class| class.get_fluid_state) {
            Some(get_fluid_state) => get_fluid_state(self, state),
            None => Fluids::EMPTY.default_fluid_state(),
        }
    }

    /// `BlockBehaviour.isRandomlyTicking(BlockState)`, which a class may override.
    pub fn is_randomly_ticking_state(&self, state: BlockState) -> bool {
        match self.class.find(|class| class.is_randomly_ticking) {
            Some(is_randomly_ticking) => is_randomly_ticking(self, state),
            None => self.is_randomly_ticking,
        }
    }

    /// `BlockBehaviour.useShapeForLightOcclusion(BlockState)`, which a class may override: false.
    pub fn use_shape_for_light_occlusion_of(&self, state: BlockState) -> bool {
        match self.class.find(|class| class.use_shape_for_light_occlusion) {
            Some(use_shape_for_light_occlusion) => use_shape_for_light_occlusion(self, state),
            None => false,
        }
    }

    /// `BlockBehaviour.propagatesSkylightDown(BlockState)`, which a class may override: whether
    /// the shape isn't the whole block and the state holds no fluid.
    pub fn propagates_skylight_down_of(&self, state: BlockState) -> bool {
        match self.class.find(|class| class.propagates_skylight_down) {
            Some(propagates_skylight_down) => propagates_skylight_down(self, state),
            None => {
                !Block::is_shape_full_block(&state.get_shape(&EmptyBlockGetter, &BlockPos::ZERO))
                    && state.get_fluid_state().is_empty()
            }
        }
    }

    /// `BlockBehaviour.getLightDampening(BlockState)`, which a class may override: 15 for a
    /// solid render, else 0 if skylight goes down through it and 1 if not. `initCache` asks it
    /// with the state's `isSolidRender` and `propagatesSkylightDown` worked out just before.
    pub fn get_light_dampening_of(
        &self,
        state: BlockState,
        solid_render: bool,
        propagates_skylight_down: bool,
    ) -> i32 {
        match self.class.find(|class| class.get_light_dampening) {
            Some(get_light_dampening) => get_light_dampening(self, state),
            None if solid_render => 15,
            None if propagates_skylight_down => 0,
            None => 1,
        }
    }

    /// `getMaxHorizontalOffset()`, which a class may override: a quarter of a block.
    pub fn get_max_horizontal_offset(&self) -> f32 {
        match self.class.find(|class| class.get_max_horizontal_offset) {
            Some(get_max_horizontal_offset) => get_max_horizontal_offset(self),
            None => 0.25,
        }
    }

    /// `getMaxVerticalOffset()`, which a class may override: a fifth of a block.
    pub fn get_max_vertical_offset(&self) -> f32 {
        match self.class.find(|class| class.get_max_vertical_offset) {
            Some(get_max_vertical_offset) => get_max_vertical_offset(self),
            None => 0.2,
        }
    }

    /// `stepOn(Level, BlockPos, BlockState, Entity)`, which a class may override: nothing.
    pub fn step_on(
        &self,
        level: &dyn Level,
        pos: &BlockPos,
        on_state: BlockState,
        entity: &mut dyn EntityClass,
    ) {
        if let Some(step_on) = self.class.find(|class| class.step_on) {
            step_on(self, level, pos, on_state, entity);
        }
    }
}

impl Deref for BlockData {
    type Target = BlockBehaviour;

    fn deref(&self) -> &BlockBehaviour {
        &self.behaviour
    }
}

impl Block {
    /// How many blocks a field of `Blocks` holding one takes.
    pub(crate) const COUNT: u16 = 1;

    /// `BlockBehaviour.UPDATE_SHAPE_ORDER`.
    pub const UPDATE_SHAPE_ORDER: [Direction; 6] = [
        Direction::WEST,
        Direction::EAST,
        Direction::NORTH,
        Direction::SOUTH,
        Direction::DOWN,
        Direction::UP,
    ];

    /// `animateTick(BlockState, Level, BlockPos, RandomSource)`: its class's, or `Block`'s, which
    /// does nothing.
    pub fn animate_tick(
        &self,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if let Some(animate_tick) = self.class().find(|class| class.animate_tick) {
            animate_tick(self, state, level, pos, random);
        }
    }

    /// `triggerEvent(BlockState, Level, BlockPos, int, int)`: its class's, or `BlockBehaviour`'s,
    /// which handles nothing.
    pub fn trigger_event(
        &self,
        state: BlockState,
        level: &mut LevelTicking<'_>,
        pos: &BlockPos,
        b0: i32,
        b1: i32,
    ) -> bool {
        match self.class().find(|class| class.trigger_event) {
            Some(trigger_event) => trigger_event(self, state, level, pos, b0, b1),
            None => false,
        }
    }

    /// `getTicker(Level, BlockEntityType)` of a state on the client: its class's, for an
    /// `EntityBlock`, or none.
    pub fn get_ticker(
        &self,
        state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        if !self.class().is_entity_block() {
            return None;
        }
        self.class()
            .find(|class| class.get_ticker)
            .and_then(|get_ticker| get_ticker(self, state, r#type))
    }

    /// `updateShape(state, level, ticks, pos, direction, neighbourPos, neighbourState, random)`:
    /// its class's, or `BlockBehaviour`'s, the state as it is. A class whose override isn't
    /// ported says so and keeps the state.
    #[allow(clippy::too_many_arguments)]
    pub fn update_shape(
        &self,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        random: &mut dyn RandomSource,
    ) -> BlockState {
        Block::update_shape_from(
            self.class(),
            self,
            state,
            level,
            pos,
            direction_to_neighbour,
            neighbour_pos,
            neighbour_state,
            random,
        )
    }

    /// `super.updateShape(...)` from `class`'s override: its parent's, or `BlockBehaviour`'s.
    #[allow(clippy::too_many_arguments)]
    pub fn super_update_shape(
        block: &BlockData,
        class: &'static BlockClass,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        random: &mut dyn RandomSource,
    ) -> BlockState {
        match class.parent {
            Some(parent) => Block::update_shape_from(
                parent,
                block,
                state,
                level,
                pos,
                direction_to_neighbour,
                neighbour_pos,
                neighbour_state,
                random,
            ),
            None => state,
        }
    }

    /// The `updateShape` nearest `class` up its chain.
    #[allow(clippy::too_many_arguments)]
    fn update_shape_from(
        class: &'static BlockClass,
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        random: &mut dyn RandomSource,
    ) -> BlockState {
        let mut class = Some(class);
        while let Some(current) = class {
            if let Some(update_shape) = current.update_shape {
                return update_shape(
                    block,
                    state,
                    level,
                    pos,
                    direction_to_neighbour,
                    neighbour_pos,
                    neighbour_state,
                    random,
                );
            }
            if UPDATE_SHAPE_CLASSES.contains(&current.name) {
                eprintln!("{}.updateShape isn't ported", current.name);
                return state;
            }
            class = current.parent;
        }
        state
    }

    /// `canSurvive(state, level, pos)`: its class's, or `BlockBehaviour`'s, true. A class whose
    /// override isn't ported says so and answers true.
    pub fn can_survive(&self, state: BlockState, level: &dyn BlockGetter, pos: &BlockPos) -> bool {
        let mut class = Some(self.class());
        while let Some(current) = class {
            if let Some(can_survive) = current.can_survive {
                return can_survive(self, state, level, pos);
            }
            if CAN_SURVIVE_CLASSES.contains(&current.name) {
                eprintln!("{}.canSurvive isn't ported", current.name);
                return true;
            }
            class = current.parent;
        }
        true
    }

    /// `isSignalSource(state)`: its class's, or `BlockBehaviour`'s, false.
    pub fn is_signal_source(&self, state: BlockState) -> bool {
        self.class()
            .find(|class| class.is_signal_source)
            .is_some_and(|is_signal_source| is_signal_source(self, state))
    }

    /// `shouldRedstoneWireConnectTo(state, level, pos, direction)`: its class's, or
    /// `BlockBehaviour`'s, a signal source's beside it.
    pub fn should_redstone_wire_connect_to(
        &self,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction: Option<Direction>,
    ) -> bool {
        match self
            .class()
            .find(|class| class.should_redstone_wire_connect_to)
        {
            Some(should_redstone_wire_connect_to) => {
                should_redstone_wire_connect_to(self, state, level, pos, direction)
            }
            None => state.is_signal_source() && direction.is_some(),
        }
    }

    /// `updateIndirectNeighbourShapes(state, level, pos, updateFlags, updateLimit)`: its class's,
    /// or `BlockBehaviour`'s, none.
    pub fn update_indirect_neighbour_shapes(
        &self,
        state: BlockState,
        level: &mut ClientLevel,
        pos: &BlockPos,
        update_flags: i32,
        update_limit: i32,
    ) {
        if let Some(update_indirect_neighbour_shapes) = self
            .class()
            .find(|class| class.update_indirect_neighbour_shapes)
        {
            update_indirect_neighbour_shapes(self, state, level, pos, update_flags, update_limit);
        }
    }

    /// `updateFromNeighbourShapes(state, level, pos)`: the state as each neighbour, in
    /// `UPDATE_SHAPE_ORDER`, has it update its shape, with the level's random.
    pub fn update_from_neighbour_shapes(
        state: BlockState,
        level: &ClientLevel,
        pos: &BlockPos,
    ) -> BlockState {
        let mut new_state = state;
        for direction in Block::UPDATE_SHAPE_ORDER {
            let neighbour_pos = pos.relative(direction);
            let neighbour_state = level.get_block_state(&neighbour_pos);
            new_state = new_state.get_block().update_shape(
                new_state,
                level,
                pos,
                direction,
                &neighbour_pos,
                neighbour_state,
                &mut *level.get_random(),
            );
        }
        new_state
    }

    /// `updateOrDestroy(blockState, newState, level, blockPos, updateFlags, updateLimit)`: the new
    /// state set, unless it's air, which only a server destroys the block for.
    pub fn update_or_destroy(
        block_state: BlockState,
        new_state: BlockState,
        level: &mut ClientLevel,
        block_pos: &BlockPos,
        update_flags: i32,
        update_limit: i32,
    ) {
        if new_state != block_state && !new_state.is_air() {
            level.set_block(block_pos, new_state, update_flags & -33, update_limit);
        }
    }

    /// `isExceptionForConnection(state)`: what fences, walls and panes don't join to however
    /// sturdy its face.
    pub fn is_exception_for_connection(state: BlockState) -> bool {
        state.class_is_leaves()
            || state.is(Blocks::BARRIER)
            || state.is(Blocks::CARVED_PUMPKIN)
            || state.is(Blocks::JACK_O_LANTERN)
            || state.is(Blocks::MELON)
            || state.is(Blocks::PUMPKIN)
            || state.is_tag(BlockTags::SHULKER_BOXES)
    }

    /// `super.animateTick(...)` from `class`'s override: its parent's, if one overrides it.
    pub fn super_animate_tick(
        block: &BlockData,
        class: &'static BlockClass,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if let Some(animate_tick) = class.find_super(|class| class.animate_tick) {
            animate_tick(block, state, level, pos, random);
        }
    }

    /// The `Block` constructor, for `class`, whose state definition `definition` makes: Java's
    /// `this.createBlockStateDefinition`, the most derived class's.
    pub fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let behaviour = BlockBehaviour::new(properties);
        let mut builder = Builder::new();
        definition(&mut builder);
        let state_definition = builder.create();
        BlockData {
            behaviour,
            class,
            state_definition,
            default_block_state: state_definition.any(),
            fields: Box::new(()),
        }
    }

    /// `new Block(properties)`.
    pub fn new(properties: Properties) -> BlockData {
        Block::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    /// None: a plain block has one state.
    pub fn create_block_state_definition(_builder: &mut Builder) {}

    /// `Shapes.box` of sixteenths of a block, the way blocks give their shapes.
    pub fn r#box(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> VoxelShape {
        Shapes::r#box(
            min_x / 16.0,
            min_y / 16.0,
            min_z / 16.0,
            max_x / 16.0,
            max_y / 16.0,
            max_z / 16.0,
        )
    }

    /// `column(double, double, double)`: a box `size_xz` sixteenths across, centered.
    pub fn column(size_xz: f64, min_y: f64, max_y: f64) -> VoxelShape {
        Block::column_xz(size_xz, size_xz, min_y, max_y)
    }

    /// `column(double, double, double, double)`.
    pub fn column_xz(size_x: f64, size_z: f64, min_y: f64, max_y: f64) -> VoxelShape {
        let half_x = size_x / 2.0;
        let half_z = size_z / 2.0;
        Block::r#box(
            8.0 - half_x,
            min_y,
            8.0 - half_z,
            8.0 + half_x,
            max_y,
            8.0 + half_z,
        )
    }

    /// `boxes(int, IntFunction)`: the shapes the factory makes of 0 to `end_inclusive`.
    pub fn boxes(end_inclusive: i32, factory: impl Fn(i32) -> VoxelShape) -> Vec<VoxelShape> {
        (0..=end_inclusive).map(factory).collect()
    }

    /// `cube(double)`: a box centered every way, as big every way.
    pub fn cube(size: f64) -> VoxelShape {
        Block::cube_xyz(size, size, size)
    }

    /// `cube(double, double, double)`: a box centered every way.
    pub fn cube_xyz(size_x: f64, size_y: f64, size_z: f64) -> VoxelShape {
        let half_y = size_y / 2.0;
        Block::column_xz(size_x, size_z, 8.0 - half_y, 8.0 + half_y)
    }

    /// `boxZ(double, double, double)`: a box as wide as it's high, centered on x and y, from
    /// `min_z` to `max_z`.
    pub fn box_z_square(size_xy: f64, min_z: f64, max_z: f64) -> VoxelShape {
        Block::box_z_sized(size_xy, size_xy, min_z, max_z)
    }

    /// `boxZ(double, double, double, double)`: a box centered on x and y, from `min_z` to `max_z`.
    pub fn box_z_sized(size_x: f64, size_y: f64, min_z: f64, max_z: f64) -> VoxelShape {
        let half_y = size_y / 2.0;
        Block::box_z(size_x, 8.0 - half_y, 8.0 + half_y, min_z, max_z)
    }

    /// `boxZ(double, double, double, double, double)`: a box centered along x, from `min_z` to
    /// `max_z`.
    pub fn box_z(size_x: f64, min_y: f64, max_y: f64, min_z: f64, max_z: f64) -> VoxelShape {
        let half_x = size_x / 2.0;
        Block::r#box(8.0 - half_x, min_y, min_z, 8.0 + half_x, max_y, max_z)
    }

    /// `getShapeForEachState(Function)`: the shape of each of the block's states, worked out once
    /// and looked up by the state's index (Java's `ImmutableMap`).
    pub fn get_shape_for_each_state(
        block: &BlockData,
        shape_calculator: impl Fn(BlockState) -> VoxelShape,
    ) -> Vec<VoxelShape> {
        block
            .get_state_definition()
            .get_possible_states()
            .map(shape_calculator)
            .collect()
    }

    /// `getShapeForEachState(Function, Property...)`: the shape of each state with the ignored
    /// properties at their first values, worked out once for those and shared by the states that
    /// only differ from them in the ignored properties.
    pub fn get_shape_for_each_state_ignoring(
        block: &BlockData,
        shape_calculator: impl Fn(BlockState) -> VoxelShape,
        ignored_properties: &[&'static dyn Property],
    ) -> Vec<VoxelShape> {
        let definition = block.get_state_definition();
        let with_defaults = |state: BlockState| {
            ignored_properties.iter().fold(state, |state, property| {
                state.set_comparable(*property, property.possible_value(0))
            })
        };
        let computed: Vec<Option<VoxelShape>> = definition
            .get_possible_states()
            .map(|state| (with_defaults(state) == state).then(|| shape_calculator(state)))
            .collect();
        definition
            .get_possible_states()
            .map(|state| {
                computed[with_defaults(state).index() as usize]
                    .clone()
                    .expect("a state with the ignored properties' first values has its shape")
            })
            .collect()
    }

    /// `instanceof`: whether the block's class is `class` or extends it.
    pub fn is_instance_of(&self, class: &'static BlockClass) -> bool {
        let mut current = Some(Block::data(*self).class());
        while let Some(candidate) = current {
            if std::ptr::eq(candidate, class) {
                return true;
            }
            current = candidate.parent;
        }
        false
    }

    /// Whether a state's face toward `direction` shows past its neighbour there. Java caches the
    /// shapes' answer by the pair of shapes (`OCCLUSION_CACHE`), which only saves the work.
    pub fn should_render_face(
        state: BlockState,
        neighbor_state: BlockState,
        direction: Direction,
    ) -> bool {
        let occluder = neighbor_state.face_occlusion_shape(direction.get_opposite());
        if occluder == Shapes::block_ref() {
            return false;
        }

        if state.skip_rendering(neighbor_state, direction) {
            return false;
        }

        if occluder == Shapes::empty_ref() {
            return true;
        }

        let shape = state.face_occlusion_shape(direction);
        if shape == Shapes::empty_ref() {
            return true;
        }

        Shapes::join_is_not_empty(shape, occluder, BooleanOp::ONLY_FIRST)
    }

    /// `isFaceFull`: whether the shape's face toward `direction` is the whole face.
    pub fn is_face_full(shape: &VoxelShape, direction: Direction) -> bool {
        Block::is_shape_full_block(&shape.get_face_shape(direction))
    }

    /// Whether the shape is the whole block, voxel for voxel.
    pub fn is_shape_full_block(shape: &VoxelShape) -> bool {
        !Shapes::join_is_not_empty(&Shapes::block(), shape, BooleanOp::NOT_SAME)
    }

    /// The block whose id it is, as a field of `Blocks` names it.
    pub(crate) const fn starting_at(id: u16) -> Block {
        Block(id)
    }

    /// Its id in `BuiltInRegistries.BLOCK`.
    pub fn get_id_in_registry(&self) -> i32 {
        i32::from(self.0)
    }

    /// `Block.BLOCK_STATE_REGISTRY`: every state, by id, in the order `Blocks` registers their
    /// blocks.
    pub fn block_state_registry() -> &'static IdMapper<BlockState> {
        &REGISTRY.block_state_registry
    }

    /// `getId(BlockState)`: the state's id, or 0 for none.
    pub fn get_id(block_state: Option<BlockState>) -> i32 {
        match block_state {
            None => 0,
            Some(state) => {
                let id = Block::block_state_registry().get_id(state);
                if id == -1 { 0 } else { id }
            }
        }
    }

    /// `stateById(int)`: the state with the id, or air's default.
    pub fn state_by_id(id_with_data: i32) -> BlockState {
        Block::block_state_registry()
            .by_id(id_with_data)
            .unwrap_or_else(|| Blocks::AIR.default_block_state())
    }

    /// What the block was made with, once `Blocks` has registered every block.
    pub fn data(self) -> &'static BlockData {
        &REGISTRY.blocks[self.0 as usize]
    }
}

impl Deref for Block {
    type Target = BlockData;

    fn deref(&self) -> &'static BlockData {
        self.data()
    }
}

/// `Block{minecraft:stone}`: the registered name.
impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Block{{{}}}", blocks::registered_name(*self))
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::Block;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::world::level::block::blocks;
    use crate::net::minecraft::world::level::block::state::block_behaviour::{StatePredicate, ViewBlocking};
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::net::minecraft::world::level::block::state::properties::block_state_properties::tests::field_name;

    /// A block is its registered name, `minecraft:stone`.
    impl Encode for Block {
        fn encode(&self) -> Value {
            Value::from(blocks::registry().get_key(*self).to_string())
        }
    }

    impl Decode for Block {
        fn decode(value: &Value) -> Block {
            let name = String::decode(value);
            blocks::registry()
                .get_value(&Identifier::parse(&name))
                .unwrap_or_else(|| panic!("no block {name}"))
        }
    }

    /// A predicate's kind as vanilla tells them apart: Properties' default, one of Blocks' named
    /// ones, or another.
    fn kind(predicate: StatePredicate) -> String {
        match predicate {
            StatePredicate::Default | StatePredicate::DefaultSuffocating => "default".to_owned(),
            StatePredicate::NotClosedShulker => "NOT_CLOSED_SHULKER".to_owned(),
            StatePredicate::State(Some(name), _) => name.to_owned(),
            StatePredicate::State(None, _) => "custom".to_owned(),
        }
    }

    fn view_blocking_kind(predicate: ViewBlocking) -> String {
        match predicate {
            ViewBlocking::Default(_) => "default".to_owned(),
            ViewBlocking::NearPlaneIntersectsOutline => "NEAR_PLANE_INTERSECTS_OUTLINE".to_owned(),
            ViewBlocking::State(_) => "custom".to_owned(),
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.block.Block");
        golden.field(
            "BLOCK_STATE_REGISTRY.size()",
            Block::block_state_registry().size(),
        );
        golden.check(
            "stateById(I)Lnet/minecraft/world/level/block/state/BlockState;",
            Block::state_by_id,
        );
        golden.check(
            "getId(Lnet/minecraft/world/level/block/state/BlockState;)I",
            |state: Option<BlockState>| Block::get_id(state),
        );
        golden.check("BuiltInRegistries.BLOCK.getId", |block: Block| {
            block.get_id_in_registry()
        });
        golden.check("getClass()", |block: Block| block.class().name.to_owned());
        golden.check("toString()Ljava/lang/String;", |block: Block| {
            block.to_string()
        });
        golden.check(
            "defaultBlockState()Lnet/minecraft/world/level/block/state/BlockState;",
            |block: Block| block.default_block_state(),
        );
        golden.check("getStateDefinition().getProperties()", |block: Block| {
            block
                .get_state_definition()
                .get_properties()
                .iter()
                .map(|property| field_name(*property).to_owned())
                .collect::<Vec<_>>()
        });
        golden.check(
            "getStateDefinition().getPossibleStates()",
            |block: Block| {
                block
                    .get_state_definition()
                    .get_possible_states()
                    .collect::<Vec<_>>()
            },
        );
        golden.check("getStateDefinition().any()", |block: Block| {
            block.get_state_definition().any()
        });
        golden.check("getStateDefinition().isSingletonState()", |block: Block| {
            block.get_state_definition().is_singleton_state()
        });
        golden.check("getStateDefinition().toString()", |block: Block| {
            block.get_state_definition().to_string()
        });
        golden.check(
            "getStateDefinition().getProperty(Ljava/lang/String;)",
            |block: Block, name: String| {
                block
                    .get_state_definition()
                    .get_property(&name)
                    .map(|property| field_name(property).to_owned())
            },
        );
        golden.check("getDescriptionId()Ljava/lang/String;", |block: Block| {
            block.get_description_id().to_owned()
        });
        golden.check("getLootTable()Ljava/util/Optional;", |block: Block| {
            block
                .get_loot_table()
                .map(|key| key.identifier().to_string())
        });
        golden.check("getExplosionResistance()F", |block: Block| {
            block.get_explosion_resistance()
        });
        golden.check("getFriction()F", |block: Block| block.get_friction());
        golden.check("getSpeedFactor()F", |block: Block| block.get_speed_factor());
        golden.check("getJumpFactor()F", |block: Block| block.get_jump_factor());
        golden.check("getBounceRestitution()F", |block: Block| {
            block.get_bounce_restitution()
        });
        golden.check("getFallDistanceReduction()F", |block: Block| {
            block.get_fall_distance_reduction()
        });
        golden.check("hasDynamicShape()Z", |block: Block| {
            block.has_dynamic_shape()
        });
        golden.check("defaultDestroyTime()F", |block: Block| {
            block.default_destroy_time()
        });
        golden.check(
            "defaultMapColor()Lnet/minecraft/world/level/material/MapColor;",
            |block: Block| block.default_map_color().id,
        );
        golden.check("hasCollision", |block: Block| block.has_collision());
        golden.check("isRandomlyTicking", |block: Block| {
            block.is_randomly_ticking
        });
        golden.check("properties().forceSolidOn", |block: Block| {
            block.properties().get_force_solid_on()
        });
        golden.check("properties().forceSolidOff", |block: Block| {
            block.properties().get_force_solid_off()
        });
        golden.check("properties().isRedstoneConductor", |block: Block| {
            kind(block.properties().get_is_redstone_conductor())
        });
        golden.check("properties().isSuffocating", |block: Block| {
            kind(block.properties().get_is_suffocating())
        });
        golden.check("properties().isViewBlocking", |block: Block| {
            view_blocking_kind(block.properties().get_is_view_blocking())
        });
        golden.finish();
    }
}
