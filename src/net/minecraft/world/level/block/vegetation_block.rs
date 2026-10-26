//! net/minecraft/world/level/block/VegetationBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine, and `updateShape`
//! with `canSurvive` and `mayPlaceOn`, which its subclasses override (`MAY_PLACE_ON_CLASSES`).

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition, MAY_PLACE_ON_CLASSES,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct VegetationBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(VegetationBlock::update_shape),
    can_survive: Some(VegetationBlock::can_survive),
    may_place_on: Some(VegetationBlock::may_place_on_supports_vegetation),
    propagates_skylight_down: Some(VegetationBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.VegetationBlock",
        &block::CLASS,
    )
};

impl VegetationBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl VegetationBlock {
    /// `propagatesSkylightDown`: without a fluid.
    fn propagates_skylight_down(block: &BlockData, state: BlockState) -> bool {
        block.get_fluid_state_of(state).is_empty()
    }
}

impl VegetationBlock {
    /// `mayPlaceOn(state, level, pos)`: its class's, whether it stands on the state below. A
    /// class whose override isn't ported says so and answers true.
    pub fn may_place_on(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        let mut class = Some(block.class());
        while let Some(current) = class {
            if let Some(may_place_on) = current.may_place_on {
                return may_place_on(block, state, level, pos);
            }
            if MAY_PLACE_ON_CLASSES.contains(&current.name) {
                eprintln!("{}.mayPlaceOn isn't ported", current.name);
                return true;
            }
            class = current.parent;
        }
        unreachable!("VegetationBlock has its own")
    }

    /// `VegetationBlock.mayPlaceOn`: on what supports vegetation.
    fn may_place_on_supports_vegetation(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> bool {
        state.is_tag(BlockTags::SUPPORTS_VEGETATION)
    }

    /// `updateShape`: air (which a client leaves) when it can't stay; its parent's otherwise.
    #[allow(clippy::too_many_arguments)]
    fn update_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        random: &mut dyn RandomSource,
    ) -> BlockState {
        if !state.can_survive(level, pos) {
            Blocks::AIR.default_block_state()
        } else {
            Block::super_update_shape(
                block,
                &CLASS,
                state,
                level,
                pos,
                direction_to_neighbour,
                neighbour_pos,
                neighbour_state,
                random,
            )
        }
    }

    /// `canSurvive`: on what it may be placed on (`mayPlaceOn`).
    fn can_survive(
        block: &BlockData,
        _state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        let below = pos.below();
        VegetationBlock::may_place_on(block, level.get_block_state(&below), level, &below)
    }

    /// `super.mayPlaceOn(...)` from `class`'s override: its parent's, up to this one's.
    pub fn super_may_place_on(
        block: &BlockData,
        class: &'static BlockClass,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        let may_place_on = class
            .find_super(|class| class.may_place_on)
            .expect("VegetationBlock has its own");
        may_place_on(block, state, level, pos)
    }

    /// `super.canSurvive(...)` from `class`'s override: its parent's, up to this one's.
    pub fn super_can_survive(
        block: &BlockData,
        class: &'static BlockClass,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        match class.find_super(|class| class.can_survive) {
            Some(can_survive) => can_survive(block, state, level, pos),
            None => true,
        }
    }
}
