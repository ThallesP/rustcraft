//! net/minecraft/world/level/block/RedstoneWireBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its shape (a dot, with lines toward its sides and up the ones it climbs), and
//! its specks (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::direction::Plane;
use crate::net::minecraft::core::particles::dust_particle_options::DustParticleOptions;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::properties::redstone_side::RedstoneSide;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct RedstoneWireBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(RedstoneWireBlock::animate_tick),
    get_shape: Some(RedstoneWireBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.RedstoneWireBlock",
        &block::CLASS,
    )
};

impl RedstoneWireBlock {
    pub const NORTH: &'static EnumProperty<RedstoneSide> = BlockStateProperties::NORTH_REDSTONE;
    pub const EAST: &'static EnumProperty<RedstoneSide> = BlockStateProperties::EAST_REDSTONE;
    pub const SOUTH: &'static EnumProperty<RedstoneSide> = BlockStateProperties::SOUTH_REDSTONE;
    pub const WEST: &'static EnumProperty<RedstoneSide> = BlockStateProperties::WEST_REDSTONE;
    pub const POWER: &'static IntegerProperty = BlockStateProperties::POWER;

    pub fn new(properties: Properties) -> BlockData {
        RedstoneWireBlock::new_with(
            properties,
            &CLASS,
            RedstoneWireBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(RedstoneWireBlock::NORTH, RedstoneSide::NONE)
                .set_value(RedstoneWireBlock::EAST, RedstoneSide::NONE)
                .set_value(RedstoneWireBlock::SOUTH, RedstoneSide::NONE)
                .set_value(RedstoneWireBlock::WEST, RedstoneSide::NONE)
                .set_value(RedstoneWireBlock::POWER, 0),
        );
        let shapes = RedstoneWireBlock::make_shapes(&block);
        block.set_fields(RedstoneWireBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            RedstoneWireBlock::NORTH,
            RedstoneWireBlock::EAST,
            RedstoneWireBlock::SOUTH,
            RedstoneWireBlock::WEST,
            RedstoneWireBlock::POWER,
        ]);
    }
}

impl RedstoneWireBlock {
    /// `PROPERTY_BY_DIRECTION`, an `EnumMap`, in its order.
    pub const PROPERTY_BY_DIRECTION: [(Direction, &'static EnumProperty<RedstoneSide>); 4] = [
        (Direction::NORTH, RedstoneWireBlock::NORTH),
        (Direction::SOUTH, RedstoneWireBlock::SOUTH),
        (Direction::WEST, RedstoneWireBlock::WEST),
        (Direction::EAST, RedstoneWireBlock::EAST),
    ];

    /// `makeShapes`: a dot, with a line toward each side it goes and up each side it climbs,
    /// for each state but for its power.
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let dot = Block::column(10.0, 0.0, 1.0);
        let floor = Shapes::rotate_horizontal(&Block::box_z(10.0, 0.0, 1.0, 0.0, 8.0));
        let up = Shapes::rotate_horizontal(&Block::box_z_sized(10.0, 16.0, 0.0, 1.0));
        Block::get_shape_for_each_state_ignoring(
            block,
            |state| {
                let mut shape = dot.clone();
                for (direction, property) in RedstoneWireBlock::PROPERTY_BY_DIRECTION {
                    shape = match state.get_value(property) {
                        RedstoneSide::UP => Shapes::or_tail(
                            &shape,
                            &[
                                Shapes::get(&floor, direction).clone(),
                                Shapes::get(&up, direction).clone(),
                            ],
                        ),
                        RedstoneSide::SIDE => Shapes::or(&shape, Shapes::get(&floor, direction)),
                        RedstoneSide::NONE => shape,
                    };
                }
                shape
            },
            &[RedstoneWireBlock::POWER],
        )
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<RedstoneWireBlock>().shapes[state.index() as usize].clone()
    }
}

impl RedstoneWireBlock {
    /// `animateTick`: while powered, specks of its colour along each side's connection.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let power = state.get_value(RedstoneWireBlock::POWER);
        if power != 0 {
            let color = COLORS[power as usize];
            for horizontal in Plane::HORIZONTAL.iterator() {
                let property = match horizontal {
                    Direction::NORTH => RedstoneWireBlock::NORTH,
                    Direction::EAST => RedstoneWireBlock::EAST,
                    Direction::SOUTH => RedstoneWireBlock::SOUTH,
                    _ => RedstoneWireBlock::WEST,
                };
                match state.get_value(property) {
                    RedstoneSide::UP => {
                        RedstoneWireBlock::spawn_particles_along_line(
                            level,
                            random,
                            pos,
                            color,
                            horizontal,
                            Direction::UP,
                            -0.5,
                            0.5,
                        );
                        RedstoneWireBlock::spawn_particles_along_line(
                            level,
                            random,
                            pos,
                            color,
                            Direction::DOWN,
                            horizontal,
                            0.0,
                            0.5,
                        );
                    }
                    RedstoneSide::SIDE => {
                        RedstoneWireBlock::spawn_particles_along_line(
                            level,
                            random,
                            pos,
                            color,
                            Direction::DOWN,
                            horizontal,
                            0.0,
                            0.5,
                        );
                    }
                    RedstoneSide::NONE => {
                        RedstoneWireBlock::spawn_particles_along_line(
                            level,
                            random,
                            pos,
                            color,
                            Direction::DOWN,
                            horizontal,
                            0.0,
                            0.3,
                        );
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_particles_along_line(
        level: &LevelWithEffects<'_>,
        random: &mut dyn RandomSource,
        pos: &BlockPos,
        color: i32,
        side: Direction,
        along: Direction,
        from: f32,
        to: f32,
    ) {
        let span = to - from;
        if random.next_float() < 0.2 * span {
            let side_of_block = 0.4375_f32;
            let position_on_line = from + span * random.next_float();
            let x = 0.5
                + f64::from(side_of_block * side.get_step_x() as f32)
                + f64::from(position_on_line * along.get_step_x() as f32);
            let y = 0.5
                + f64::from(side_of_block * side.get_step_y() as f32)
                + f64::from(position_on_line * along.get_step_y() as f32);
            let z = 0.5
                + f64::from(side_of_block * side.get_step_z() as f32)
                + f64::from(position_on_line * along.get_step_z() as f32);
            level.add_particle(
                DustParticleOptions::new(color, 1.0),
                f64::from(pos.get_x()) + x,
                f64::from(pos.get_y()) + y,
                f64::from(pos.get_z()) + z,
                0.0,
                0.0,
                0.0,
            );
        }
    }
}

/// `COLORS`: the wire's colour at each power.
static COLORS: LazyLock<[i32; 16]> = LazyLock::new(|| {
    let mut list = [0; 16];
    for (i, color) in list.iter_mut().enumerate() {
        let power = i as f32 / 15.0;
        let red = power * 0.6 + if power > 0.0 { 0.4 } else { 0.3 };
        let green = Mth::clamp_f32(power * power * 0.7 - 0.5, 0.0, 1.0);
        let blue = Mth::clamp_f32(power * power * 0.6 - 0.7, 0.0, 1.0);
        *color = ARGB::color_from_float(1.0, red, green, blue);
    }
    list
});

impl RedstoneWireBlock {}
