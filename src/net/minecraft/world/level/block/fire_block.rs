//! net/minecraft/world/level/block/FireBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (a sixteenth on each face it burns on, or on the ground) is ported,
//! and what burns (`canBurn`, from `bootStrap`'s ignite odds, which its parent's `animateTick`
//! reads). Not its burn odds, which only its ticking reads.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_fire_block::{self, BaseFireBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::pipe_block::PipeBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct FireBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(FireBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FireBlock",
        &base_fire_block::CLASS,
    )
};

impl FireBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_15;
    pub const NORTH: &'static BooleanProperty = PipeBlock::NORTH;
    pub const EAST: &'static BooleanProperty = PipeBlock::EAST;
    pub const SOUTH: &'static BooleanProperty = PipeBlock::SOUTH;
    pub const WEST: &'static BooleanProperty = PipeBlock::WEST;
    pub const UP: &'static BooleanProperty = PipeBlock::UP;

    pub fn new(properties: Properties) -> BlockData {
        FireBlock::new_with(properties, &CLASS, FireBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseFireBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(FireBlock::AGE, 0)
                .set_value(FireBlock::NORTH, false)
                .set_value(FireBlock::EAST, false)
                .set_value(FireBlock::SOUTH, false)
                .set_value(FireBlock::WEST, false)
                .set_value(FireBlock::UP, false),
        );
        let shapes = FireBlock::make_shapes(&block);
        block.set_fields(FireBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            FireBlock::AGE,
            FireBlock::NORTH,
            FireBlock::EAST,
            FireBlock::SOUTH,
            FireBlock::WEST,
            FireBlock::UP,
        ]);
    }
}

impl FireBlock {
    /// `PROPERTY_BY_DIRECTION`: `PipeBlock`'s, but down.
    pub const PROPERTY_BY_DIRECTION: [(Direction, &'static BooleanProperty); 5] = [
        (Direction::UP, FireBlock::UP),
        (Direction::NORTH, FireBlock::NORTH),
        (Direction::SOUTH, FireBlock::SOUTH),
        (Direction::WEST, FireBlock::WEST),
        (Direction::EAST, FireBlock::EAST),
    ];

    /// `makeShapes`: a sixteenth on each face it burns on, or `BaseFireBlock`'s on the ground,
    /// for each state but for its age. Java's map of properties is a `HashMap` of directions,
    /// whose order changes from run to run and not the union.
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let shapes = Shapes::rotate_all(&Block::box_z_square(16.0, 0.0, 1.0));
        Block::get_shape_for_each_state_ignoring(
            block,
            |state| {
                let mut shape = Shapes::empty();
                for (direction, property) in FireBlock::PROPERTY_BY_DIRECTION {
                    if state.get_value(property) {
                        shape = Shapes::or(&shape, Shapes::get(&shapes, direction));
                    }
                }
                if shape.is_empty() {
                    base_fire_block::SHAPE.clone()
                } else {
                    shape
                }
            },
            &[FireBlock::AGE],
        )
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<FireBlock>().shapes[state.index() as usize].clone()
    }
}

/// `igniteOdds`, as `bootStrap` sets them (`setFlammable`'s first odds), a later one for a block
/// replacing an earlier.
static IGNITE_ODDS: LazyLock<HashMap<Block, i32>> = LazyLock::new(|| {
    let mut ignite_odds = HashMap::new();
    let mut set_flammable = |block: Block, odds: i32| {
        ignite_odds.insert(block, odds);
    };
    set_flammable(Blocks::OAK_PLANKS, 5);
    set_flammable(Blocks::SPRUCE_PLANKS, 5);
    set_flammable(Blocks::BIRCH_PLANKS, 5);
    set_flammable(Blocks::JUNGLE_PLANKS, 5);
    set_flammable(Blocks::ACACIA_PLANKS, 5);
    set_flammable(Blocks::CHERRY_PLANKS, 5);
    set_flammable(Blocks::DARK_OAK_PLANKS, 5);
    set_flammable(Blocks::PALE_OAK_PLANKS, 5);
    set_flammable(Blocks::POPLAR_PLANKS, 5);
    set_flammable(Blocks::MANGROVE_PLANKS, 5);
    set_flammable(Blocks::BAMBOO_PLANKS, 5);
    set_flammable(Blocks::BAMBOO_MOSAIC, 5);
    set_flammable(Blocks::OAK_SLAB, 5);
    set_flammable(Blocks::SPRUCE_SLAB, 5);
    set_flammable(Blocks::BIRCH_SLAB, 5);
    set_flammable(Blocks::JUNGLE_SLAB, 5);
    set_flammable(Blocks::ACACIA_SLAB, 5);
    set_flammable(Blocks::CHERRY_SLAB, 5);
    set_flammable(Blocks::DARK_OAK_SLAB, 5);
    set_flammable(Blocks::PALE_OAK_SLAB, 5);
    set_flammable(Blocks::POPLAR_SLAB, 5);
    set_flammable(Blocks::MANGROVE_SLAB, 5);
    set_flammable(Blocks::BAMBOO_SLAB, 5);
    set_flammable(Blocks::BAMBOO_MOSAIC_SLAB, 5);
    set_flammable(Blocks::OAK_FENCE_GATE, 5);
    set_flammable(Blocks::SPRUCE_FENCE_GATE, 5);
    set_flammable(Blocks::BIRCH_FENCE_GATE, 5);
    set_flammable(Blocks::JUNGLE_FENCE_GATE, 5);
    set_flammable(Blocks::ACACIA_FENCE_GATE, 5);
    set_flammable(Blocks::CHERRY_FENCE_GATE, 5);
    set_flammable(Blocks::DARK_OAK_FENCE_GATE, 5);
    set_flammable(Blocks::PALE_OAK_FENCE_GATE, 5);
    set_flammable(Blocks::POPLAR_FENCE_GATE, 5);
    set_flammable(Blocks::MANGROVE_FENCE_GATE, 5);
    set_flammable(Blocks::BAMBOO_FENCE_GATE, 5);
    set_flammable(Blocks::OAK_FENCE, 5);
    set_flammable(Blocks::SPRUCE_FENCE, 5);
    set_flammable(Blocks::BIRCH_FENCE, 5);
    set_flammable(Blocks::JUNGLE_FENCE, 5);
    set_flammable(Blocks::ACACIA_FENCE, 5);
    set_flammable(Blocks::CHERRY_FENCE, 5);
    set_flammable(Blocks::DARK_OAK_FENCE, 5);
    set_flammable(Blocks::PALE_OAK_FENCE, 5);
    set_flammable(Blocks::POPLAR_FENCE, 5);
    set_flammable(Blocks::MANGROVE_FENCE, 5);
    set_flammable(Blocks::BAMBOO_FENCE, 5);
    set_flammable(Blocks::OAK_STAIRS, 5);
    set_flammable(Blocks::BIRCH_STAIRS, 5);
    set_flammable(Blocks::SPRUCE_STAIRS, 5);
    set_flammable(Blocks::JUNGLE_STAIRS, 5);
    set_flammable(Blocks::ACACIA_STAIRS, 5);
    set_flammable(Blocks::CHERRY_STAIRS, 5);
    set_flammable(Blocks::DARK_OAK_STAIRS, 5);
    set_flammable(Blocks::PALE_OAK_STAIRS, 5);
    set_flammable(Blocks::POPLAR_STAIRS, 5);
    set_flammable(Blocks::MANGROVE_STAIRS, 5);
    set_flammable(Blocks::BAMBOO_STAIRS, 5);
    set_flammable(Blocks::BAMBOO_MOSAIC_STAIRS, 5);
    set_flammable(Blocks::OAK_LOG, 5);
    set_flammable(Blocks::SPRUCE_LOG, 5);
    set_flammable(Blocks::BIRCH_LOG, 5);
    set_flammable(Blocks::JUNGLE_LOG, 5);
    set_flammable(Blocks::ACACIA_LOG, 5);
    set_flammable(Blocks::CHERRY_LOG, 5);
    set_flammable(Blocks::PALE_OAK_LOG, 5);
    set_flammable(Blocks::POPLAR_LOG, 5);
    set_flammable(Blocks::DARK_OAK_LOG, 5);
    set_flammable(Blocks::MANGROVE_LOG, 5);
    set_flammable(Blocks::BAMBOO_BLOCK, 5);
    set_flammable(Blocks::STRIPPED_OAK_LOG, 5);
    set_flammable(Blocks::STRIPPED_SPRUCE_LOG, 5);
    set_flammable(Blocks::STRIPPED_BIRCH_LOG, 5);
    set_flammable(Blocks::STRIPPED_JUNGLE_LOG, 5);
    set_flammable(Blocks::STRIPPED_ACACIA_LOG, 5);
    set_flammable(Blocks::STRIPPED_CHERRY_LOG, 5);
    set_flammable(Blocks::STRIPPED_DARK_OAK_LOG, 5);
    set_flammable(Blocks::STRIPPED_PALE_OAK_LOG, 5);
    set_flammable(Blocks::STRIPPED_POPLAR_LOG, 5);
    set_flammable(Blocks::STRIPPED_MANGROVE_LOG, 5);
    set_flammable(Blocks::STRIPPED_BAMBOO_BLOCK, 5);
    set_flammable(Blocks::STRIPPED_OAK_WOOD, 5);
    set_flammable(Blocks::STRIPPED_SPRUCE_WOOD, 5);
    set_flammable(Blocks::STRIPPED_BIRCH_WOOD, 5);
    set_flammable(Blocks::STRIPPED_JUNGLE_WOOD, 5);
    set_flammable(Blocks::STRIPPED_ACACIA_WOOD, 5);
    set_flammable(Blocks::STRIPPED_CHERRY_WOOD, 5);
    set_flammable(Blocks::STRIPPED_DARK_OAK_WOOD, 5);
    set_flammable(Blocks::STRIPPED_PALE_OAK_WOOD, 5);
    set_flammable(Blocks::STRIPPED_POPLAR_WOOD, 5);
    set_flammable(Blocks::STRIPPED_MANGROVE_WOOD, 5);
    set_flammable(Blocks::OAK_WOOD, 5);
    set_flammable(Blocks::SPRUCE_WOOD, 5);
    set_flammable(Blocks::BIRCH_WOOD, 5);
    set_flammable(Blocks::JUNGLE_WOOD, 5);
    set_flammable(Blocks::ACACIA_WOOD, 5);
    set_flammable(Blocks::CHERRY_WOOD, 5);
    set_flammable(Blocks::PALE_OAK_WOOD, 5);
    set_flammable(Blocks::POPLAR_WOOD, 5);
    set_flammable(Blocks::DARK_OAK_WOOD, 5);
    set_flammable(Blocks::MANGROVE_WOOD, 5);
    set_flammable(Blocks::MANGROVE_ROOTS, 5);
    set_flammable(Blocks::OAK_LEAVES, 30);
    set_flammable(Blocks::SPRUCE_LEAVES, 30);
    set_flammable(Blocks::BIRCH_LEAVES, 30);
    set_flammable(Blocks::JUNGLE_LEAVES, 30);
    set_flammable(Blocks::ACACIA_LEAVES, 30);
    set_flammable(Blocks::CHERRY_LEAVES, 30);
    set_flammable(Blocks::DARK_OAK_LEAVES, 30);
    set_flammable(Blocks::PALE_OAK_LEAVES, 30);
    set_flammable(Blocks::MANGROVE_LEAVES, 30);
    set_flammable(Blocks::RED_POPLAR_LEAVES, 30);
    set_flammable(Blocks::ORANGE_POPLAR_LEAVES, 30);
    set_flammable(Blocks::YELLOW_POPLAR_LEAVES, 30);
    set_flammable(Blocks::BOOKSHELF, 30);
    set_flammable(Blocks::TNT, 15);
    set_flammable(Blocks::SHORT_GRASS, 60);
    set_flammable(Blocks::FERN, 60);
    set_flammable(Blocks::DEAD_BUSH, 60);
    set_flammable(Blocks::SHORT_DRY_GRASS, 60);
    set_flammable(Blocks::TALL_DRY_GRASS, 60);
    set_flammable(Blocks::SUNFLOWER, 60);
    set_flammable(Blocks::LILAC, 60);
    set_flammable(Blocks::ROSE_BUSH, 60);
    set_flammable(Blocks::PEONY, 60);
    set_flammable(Blocks::RED_SHRUB, 60);
    set_flammable(Blocks::TALL_GRASS, 60);
    set_flammable(Blocks::LARGE_FERN, 60);
    set_flammable(Blocks::DANDELION, 60);
    set_flammable(Blocks::GOLDEN_DANDELION, 60);
    set_flammable(Blocks::POPPY, 60);
    set_flammable(Blocks::OPEN_EYEBLOSSOM, 60);
    set_flammable(Blocks::CLOSED_EYEBLOSSOM, 60);
    set_flammable(Blocks::BLUE_ORCHID, 60);
    set_flammable(Blocks::ALLIUM, 60);
    set_flammable(Blocks::AZURE_BLUET, 60);
    set_flammable(Blocks::RED_TULIP, 60);
    set_flammable(Blocks::ORANGE_TULIP, 60);
    set_flammable(Blocks::WHITE_TULIP, 60);
    set_flammable(Blocks::PINK_TULIP, 60);
    set_flammable(Blocks::OXEYE_DAISY, 60);
    set_flammable(Blocks::CORNFLOWER, 60);
    set_flammable(Blocks::LILY_OF_THE_VALLEY, 60);
    set_flammable(Blocks::TORCHFLOWER, 60);
    set_flammable(Blocks::PITCHER_PLANT, 60);
    set_flammable(Blocks::WITHER_ROSE, 60);
    set_flammable(Blocks::PINK_PETALS, 60);
    set_flammable(Blocks::WILDFLOWERS, 60);
    set_flammable(Blocks::LEAF_LITTER, 60);
    set_flammable(Blocks::CACTUS_FLOWER, 60);
    for &block in Blocks::WOOL.as_list() {
        set_flammable(block, 30);
    }
    for &block in Blocks::WOOL_STAIRS.as_list() {
        set_flammable(block, 30);
    }
    for &block in Blocks::WOOL_SLAB.as_list() {
        set_flammable(block, 30);
    }
    set_flammable(Blocks::VINE, 15);
    set_flammable(Blocks::COAL_BLOCK, 5);
    set_flammable(Blocks::HAY_BLOCK, 60);
    set_flammable(Blocks::TARGET, 15);
    for &block in Blocks::CARPET.as_list() {
        set_flammable(block, 60);
    }
    set_flammable(Blocks::PALE_MOSS_BLOCK, 5);
    set_flammable(Blocks::PALE_MOSS_CARPET, 5);
    set_flammable(Blocks::PALE_HANGING_MOSS, 5);
    set_flammable(Blocks::DRIED_KELP_BLOCK, 30);
    set_flammable(Blocks::BAMBOO, 60);
    set_flammable(Blocks::SCAFFOLDING, 60);
    set_flammable(Blocks::LECTERN, 30);
    set_flammable(Blocks::COMPOSTER, 5);
    set_flammable(Blocks::SWEET_BERRY_BUSH, 60);
    set_flammable(Blocks::BEEHIVE, 5);
    set_flammable(Blocks::BEE_NEST, 30);
    set_flammable(Blocks::AZALEA_LEAVES, 30);
    set_flammable(Blocks::FLOWERING_AZALEA_LEAVES, 30);
    set_flammable(Blocks::CAVE_VINES, 15);
    set_flammable(Blocks::CAVE_VINES_PLANT, 15);
    set_flammable(Blocks::SPORE_BLOSSOM, 60);
    set_flammable(Blocks::AZALEA, 30);
    set_flammable(Blocks::FLOWERING_AZALEA, 30);
    set_flammable(Blocks::BIG_DRIPLEAF, 60);
    set_flammable(Blocks::BIG_DRIPLEAF_STEM, 60);
    set_flammable(Blocks::SMALL_DRIPLEAF, 60);
    set_flammable(Blocks::HANGING_ROOTS, 30);
    set_flammable(Blocks::GLOW_LICHEN, 15);
    set_flammable(Blocks::FIREFLY_BUSH, 60);
    set_flammable(Blocks::BUSH, 60);
    set_flammable(Blocks::ACACIA_SHELF, 30);
    set_flammable(Blocks::BAMBOO_SHELF, 30);
    set_flammable(Blocks::BIRCH_SHELF, 30);
    set_flammable(Blocks::CHERRY_SHELF, 30);
    set_flammable(Blocks::DARK_OAK_SHELF, 30);
    set_flammable(Blocks::JUNGLE_SHELF, 30);
    set_flammable(Blocks::MANGROVE_SHELF, 30);
    set_flammable(Blocks::POPLAR_SHELF, 30);
    set_flammable(Blocks::OAK_SHELF, 30);
    set_flammable(Blocks::PALE_OAK_SHELF, 30);
    set_flammable(Blocks::SPRUCE_SHELF, 30);
    set_flammable(Blocks::STRAW_BED, 30);
    ignite_odds
});

impl FireBlock {
    /// `getIgniteOdds(BlockState)`: none for a waterlogged state, its block's otherwise.
    fn get_ignite_odds(state: BlockState) -> i32 {
        if state.has_property(BlockStateProperties::WATERLOGGED)
            && state.get_value(BlockStateProperties::WATERLOGGED)
        {
            0
        } else {
            IGNITE_ODDS.get(&state.get_block()).copied().unwrap_or(0)
        }
    }

    /// `canBurn`: whether fire spreads to it.
    pub fn can_burn(state: BlockState) -> bool {
        FireBlock::get_ignite_odds(state) > 0
    }
}
