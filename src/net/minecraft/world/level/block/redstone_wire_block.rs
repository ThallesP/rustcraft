//! net/minecraft/world/level/block/RedstoneWireBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its shape (a dot, with lines toward its sides and up the ones it climbs), its
//! specks (`animateTick`), and its connections (`updateShape`, `updateIndirectNeighbourShapes`,
//! `getConnectionState`, `canSurvive`) are ported. Its power (`updatePowerStrength`, its
//! evaluators, its signals) is the server's.

use crate::net::minecraft::client::multiplayer::client_level::{ClientLevel, LevelWithEffects};
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Direction, Plane};
use crate::net::minecraft::core::particles::dust_particle_options::DustParticleOptions;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::properties::redstone_side::RedstoneSide;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::trap_door_block;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct RedstoneWireBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
    /// `crossState`: unpowered, a side each way.
    cross_state: BlockState,
}

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(RedstoneWireBlock::update_shape),
    update_indirect_neighbour_shapes: Some(RedstoneWireBlock::update_indirect_neighbour_shapes),
    can_survive: Some(RedstoneWireBlock::can_survive),
    is_signal_source: Some(RedstoneWireBlock::is_signal_source),
    should_redstone_wire_connect_to: Some(RedstoneWireBlock::should_redstone_wire_connect_to_this),
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
        let cross_state = block
            .default_block_state()
            .set_value(RedstoneWireBlock::NORTH, RedstoneSide::SIDE)
            .set_value(RedstoneWireBlock::EAST, RedstoneSide::SIDE)
            .set_value(RedstoneWireBlock::SOUTH, RedstoneSide::SIDE)
            .set_value(RedstoneWireBlock::WEST, RedstoneSide::SIDE);
        block.set_fields(RedstoneWireBlock {
            shapes,
            cross_state,
        });
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

impl RedstoneWireBlock {
    /// `PROPERTY_BY_DIRECTION.get(direction)`, for a horizontal direction.
    fn property_by_direction(direction: Direction) -> &'static EnumProperty<RedstoneSide> {
        RedstoneWireBlock::PROPERTY_BY_DIRECTION
            .iter()
            .find(|(candidate, _)| *candidate == direction)
            .map(|(_, property)| *property)
            .expect("a horizontal direction")
    }

    /// `getConnectionState(level, state, pos)`: its sides as its neighbours have them, a dot
    /// staying a dot, and a line through it where only one side of an axis connects (or a
    /// cross where none does).
    fn get_connection_state(
        block: &BlockData,
        level: &dyn BlockGetter,
        state: BlockState,
        pos: &BlockPos,
    ) -> BlockState {
        let was_dot = RedstoneWireBlock::is_dot(state);
        let mut state = RedstoneWireBlock::get_missing_connections(
            level,
            block.default_block_state().set_value(
                RedstoneWireBlock::POWER,
                state.get_value(RedstoneWireBlock::POWER),
            ),
            pos,
        );
        if was_dot && RedstoneWireBlock::is_dot(state) {
            return state;
        }

        let north = state.get_value(RedstoneWireBlock::NORTH).is_connected();
        let south = state.get_value(RedstoneWireBlock::SOUTH).is_connected();
        let east = state.get_value(RedstoneWireBlock::EAST).is_connected();
        let west = state.get_value(RedstoneWireBlock::WEST).is_connected();
        let north_south_empty = !north && !south;
        let east_west_empty = !east && !west;
        if !west && north_south_empty {
            state = state.set_value(RedstoneWireBlock::WEST, RedstoneSide::SIDE);
        }

        if !east && north_south_empty {
            state = state.set_value(RedstoneWireBlock::EAST, RedstoneSide::SIDE);
        }

        if !north && east_west_empty {
            state = state.set_value(RedstoneWireBlock::NORTH, RedstoneSide::SIDE);
        }

        if !south && east_west_empty {
            state = state.set_value(RedstoneWireBlock::SOUTH, RedstoneSide::SIDE);
        }

        state
    }

    /// `getMissingConnections(level, state, pos)`: each side not yet connected as its neighbour
    /// has it.
    fn get_missing_connections(
        level: &dyn BlockGetter,
        mut state: BlockState,
        pos: &BlockPos,
    ) -> BlockState {
        let can_connect_up = !level
            .get_block_state(&pos.above())
            .is_redstone_conductor_at(level, pos);

        for direction in Plane::HORIZONTAL.iterator() {
            let property = RedstoneWireBlock::property_by_direction(direction);
            if !state.get_value(property).is_connected() {
                let side_connection = RedstoneWireBlock::get_connecting_side_up(
                    level,
                    pos,
                    direction,
                    can_connect_up,
                );
                state = state.set_value(property, side_connection);
            }
        }

        state
    }

    /// `updateShape`: air (which a client leaves) when what's below can't hold it; its sides
    /// worked out again for what's above; the side that way as its neighbour has it, or all of
    /// them when that changes whether it connects.
    #[allow(clippy::too_many_arguments)]
    fn update_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        _random: &mut dyn RandomSource,
    ) -> BlockState {
        if direction_to_neighbour == Direction::DOWN {
            return if !RedstoneWireBlock::can_survive_on(level, neighbour_pos, neighbour_state) {
                Blocks::AIR.default_block_state()
            } else {
                state
            };
        }

        if direction_to_neighbour == Direction::UP {
            return RedstoneWireBlock::get_connection_state(block, level, state, pos);
        }

        let property = RedstoneWireBlock::property_by_direction(direction_to_neighbour);
        let side_connection =
            RedstoneWireBlock::get_connecting_side(level, pos, direction_to_neighbour);
        if side_connection.is_connected() == state.get_value(property).is_connected()
            && !RedstoneWireBlock::is_cross(state)
        {
            state.set_value(property, side_connection)
        } else {
            RedstoneWireBlock::get_connection_state(
                block,
                level,
                block
                    .fields::<RedstoneWireBlock>()
                    .cross_state
                    .set_value(
                        RedstoneWireBlock::POWER,
                        state.get_value(RedstoneWireBlock::POWER),
                    )
                    .set_value(property, side_connection),
                pos,
            )
        }
    }

    /// `isCross(state)`: connected every way.
    fn is_cross(state: BlockState) -> bool {
        state.get_value(RedstoneWireBlock::NORTH).is_connected()
            && state.get_value(RedstoneWireBlock::SOUTH).is_connected()
            && state.get_value(RedstoneWireBlock::EAST).is_connected()
            && state.get_value(RedstoneWireBlock::WEST).is_connected()
    }

    /// `isDot(state)`: connected no way.
    fn is_dot(state: BlockState) -> bool {
        !state.get_value(RedstoneWireBlock::NORTH).is_connected()
            && !state.get_value(RedstoneWireBlock::SOUTH).is_connected()
            && !state.get_value(RedstoneWireBlock::EAST).is_connected()
            && !state.get_value(RedstoneWireBlock::WEST).is_connected()
    }

    /// `updateIndirectNeighbourShapes`: for each side it goes, where the block that way isn't
    /// wire, the wire below and above that block is told of the block beside it.
    fn update_indirect_neighbour_shapes(
        block: &BlockData,
        state: BlockState,
        level: &mut ClientLevel,
        pos: &BlockPos,
        update_flags: i32,
        update_limit: i32,
    ) {
        for direction in Plane::HORIZONTAL.iterator() {
            let value = state.get_value(RedstoneWireBlock::property_by_direction(direction));
            let side_pos = pos.relative(direction);
            if value != RedstoneSide::NONE
                && !std::ptr::eq(&*level.get_block_state(&side_pos).get_block(), block)
            {
                for block_pos in [side_pos.below(), side_pos.above()] {
                    if std::ptr::eq(&*level.get_block_state(&block_pos).get_block(), block) {
                        let neighbor_pos = block_pos.relative(direction.get_opposite());
                        let neighbor_state = level.get_block_state(&neighbor_pos);
                        level.neighbor_shape_changed(
                            direction.get_opposite(),
                            &block_pos,
                            &neighbor_pos,
                            neighbor_state,
                            update_flags,
                            update_limit,
                        );
                    }
                }
            }
        }
    }

    /// `getConnectingSide(level, pos, direction)`: as `get_connecting_side_up` has it, climbing
    /// unless a conductor is above it.
    fn get_connecting_side(
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction: Direction,
    ) -> RedstoneSide {
        let can_connect_up = !level
            .get_block_state(&pos.above())
            .is_redstone_conductor_at(level, pos);
        RedstoneWireBlock::get_connecting_side_up(level, pos, direction, can_connect_up)
    }

    /// `getConnectingSide(level, pos, direction, canConnectUp)`: up the block that way to wire
    /// on top of it (side, over a face that isn't sturdy), along to what it joins there or to
    /// wire below it, or none.
    fn get_connecting_side_up(
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction: Direction,
        can_connect_up: bool,
    ) -> RedstoneSide {
        let relative_pos = pos.relative(direction);
        let relative_state = level.get_block_state(&relative_pos);
        if can_connect_up {
            let is_placeable_above = relative_state
                .get_block()
                .is_instance_of(&trap_door_block::CLASS)
                || RedstoneWireBlock::can_survive_on(level, &relative_pos, relative_state);
            if is_placeable_above
                && RedstoneWireBlock::should_connect_to(level, &relative_pos.above())
            {
                if relative_state.is_face_sturdy(level, &relative_pos, direction.get_opposite()) {
                    return RedstoneSide::UP;
                }

                return RedstoneSide::SIDE;
            }
        }

        if !RedstoneWireBlock::should_connect_to_from(
            relative_state,
            level,
            &relative_pos,
            Some(direction),
        ) && (relative_state.is_redstone_conductor_at(level, &relative_pos)
            || !RedstoneWireBlock::should_connect_to(level, &relative_pos.below()))
        {
            RedstoneSide::NONE
        } else {
            RedstoneSide::SIDE
        }
    }

    /// `canSurvive`: on what can hold it.
    fn can_survive(
        _block: &BlockData,
        _state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        let below = pos.below();
        let below_state = level.get_block_state(&below);
        RedstoneWireBlock::can_survive_on(level, &below, below_state)
    }

    /// `canSurviveOn(level, relativePos, relativeState)`: a sturdy top, or a hopper.
    fn can_survive_on(
        level: &dyn BlockGetter,
        relative_pos: &BlockPos,
        relative_state: BlockState,
    ) -> bool {
        relative_state.is_face_sturdy(level, relative_pos, Direction::UP)
            || relative_state.is(Blocks::HOPPER)
    }

    /// `shouldConnectTo(level, pos)`: to what's there, from above or below.
    fn should_connect_to(level: &dyn BlockGetter, pos: &BlockPos) -> bool {
        RedstoneWireBlock::should_connect_to_from(level.get_block_state(pos), level, pos, None)
    }

    /// `shouldConnectTo(state, level, pos, direction)`: the state's
    /// `shouldRedstoneWireConnectTo`.
    fn should_connect_to_from(
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction: Option<Direction>,
    ) -> bool {
        state.should_redstone_wire_connect_to(level, pos, direction)
    }

    /// `shouldRedstoneWireConnectTo`: wire joins wire.
    fn should_redstone_wire_connect_to_this(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _direction: Option<Direction>,
    ) -> bool {
        true
    }

    /// `isSignalSource`: `shouldSignal`, which only the server's `getBlockSignal` turns off for a
    /// moment.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
