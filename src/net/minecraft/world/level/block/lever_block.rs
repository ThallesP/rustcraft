//! net/minecraft/world/level/block/LeverBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its shape (on the face it hangs from), and its particles (`animateTick`) are
//! ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::dust_particle_options::DustParticleOptions;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::face_attached_horizontal_directional_block::{
    self, FaceAttachedHorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::attach_face::AttachFace;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct LeverBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    is_signal_source: Some(LeverBlock::is_signal_source),
    animate_tick: Some(LeverBlock::animate_tick),
    get_shape: Some(LeverBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.LeverBlock",
        &face_attached_horizontal_directional_block::CLASS,
    )
};

impl LeverBlock {
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        LeverBlock::new_with(
            properties,
            &CLASS,
            LeverBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block =
            FaceAttachedHorizontalDirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(LeverBlock::POWERED, false)
                .set_value(
                    FaceAttachedHorizontalDirectionalBlock::FACE,
                    AttachFace::WALL,
                ),
        );
        let shapes = LeverBlock::make_shapes(&block);
        block.set_fields(LeverBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            FaceAttachedHorizontalDirectionalBlock::FACE,
            HorizontalDirectionalBlock::FACING,
            LeverBlock::POWERED,
        ]);
    }
}

impl LeverBlock {
    /// `makeShapes`: on the face it hangs from, for each state but for whether it's powered.
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let attach_face = Shapes::rotate_attach_face(&Block::box_z_sized(6.0, 8.0, 10.0, 16.0));
        Block::get_shape_for_each_state_ignoring(
            block,
            |state| {
                Shapes::get(
                    Shapes::get(
                        &attach_face,
                        state.get_value(FaceAttachedHorizontalDirectionalBlock::FACE),
                    ),
                    state.get_value(HorizontalDirectionalBlock::FACING),
                )
                .clone()
            },
            &[LeverBlock::POWERED],
        )
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<LeverBlock>().shapes[state.index() as usize].clone()
    }
}

impl LeverBlock {
    /// `animateTick`: now and then, while on, a speck of redstone at its tip.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if state.get_value(LeverBlock::POWERED) && random.next_float() < 0.25 {
            LeverBlock::make_particle(state, level, pos, 0.5);
        }
    }

    fn make_particle(state: BlockState, level: &LevelWithEffects<'_>, pos: &BlockPos, scale: f32) {
        let opposite = state
            .get_value(HorizontalDirectionalBlock::FACING)
            .get_opposite();
        let opposite_connect =
            FaceAttachedHorizontalDirectionalBlock::get_connected_direction(state).get_opposite();
        let x = f64::from(pos.get_x())
            + 0.5
            + 0.1 * f64::from(opposite.get_step_x())
            + 0.2 * f64::from(opposite_connect.get_step_x());
        let y = f64::from(pos.get_y())
            + 0.5
            + 0.1 * f64::from(opposite.get_step_y())
            + 0.2 * f64::from(opposite_connect.get_step_y());
        let z = f64::from(pos.get_z())
            + 0.5
            + 0.1 * f64::from(opposite.get_step_z())
            + 0.2 * f64::from(opposite_connect.get_step_z());
        level.add_particle(
            DustParticleOptions::new(16711680, scale),
            x,
            y,
            z,
            0.0,
            0.0,
            0.0,
        );
    }
}

impl LeverBlock {
    /// `isSignalSource`: it is one.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
