//! net/minecraft/world/level/block/MultifaceBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::pipe_block::PipeBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct MultifaceBlock;

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(MultifaceBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.MultifaceBlock",
        &block::CLASS,
    )
};

impl MultifaceBlock {
    pub const DIRECTIONS: [Direction; 6] = Direction::VALUES;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        MultifaceBlock::new_with(
            properties,
            &CLASS,
            MultifaceBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(MultifaceBlock::get_default_multiface_state(
            block.get_state_definition(),
        ));
        block
    }

    /// A property for each face it can be on, which is every face (no class overrides
    /// `isFaceSupported`), and `waterlogged`.
    pub fn create_block_state_definition(builder: &mut Builder) {
        for direction in MultifaceBlock::DIRECTIONS {
            if MultifaceBlock::is_face_supported(direction) {
                builder.add(&[MultifaceBlock::get_face_property(direction)]);
            }
        }
        builder.add(&[MultifaceBlock::WATERLOGGED]);
    }

    fn is_face_supported(_face_direction: Direction) -> bool {
        true
    }

    pub fn get_face_property(face_direction: Direction) -> &'static BooleanProperty {
        PipeBlock::PROPERTY_BY_DIRECTION[face_direction as usize]
    }

    pub fn has_face(state: BlockState, face_direction: Direction) -> bool {
        state.get_value_or_else(MultifaceBlock::get_face_property(face_direction), false)
    }

    pub fn has_any_face(state: BlockState) -> bool {
        MultifaceBlock::DIRECTIONS
            .into_iter()
            .any(|direction| MultifaceBlock::has_face(state, direction))
    }

    /// Not waterlogged, on no face.
    fn get_default_multiface_state(state_definition: &'static StateDefinition) -> BlockState {
        let mut state = state_definition
            .any()
            .set_value(MultifaceBlock::WATERLOGGED, false);
        for face_property in PipeBlock::PROPERTY_BY_DIRECTION {
            state = state.try_set_value(face_property, false);
        }
        state
    }
}

impl MultifaceBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}
