//! net/minecraft/world/level/block/state/properties/BlockStateProperties.java
//!
//! Every property a block's states vary in. A property made from a filter over its enum's
//! constants (`FACING_HOPPER`, `HORIZONTAL_FACING`, `RAIL_SHAPE_STRAIGHT`) lists the constants the
//! filter keeps, in their order.

use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::front_and_top::FrontAndTop;
use crate::net::minecraft::world::level::block::copper_golem_statue_block::Pose;
use crate::net::minecraft::world::level::block::entity::trialspawner::trial_spawner_state::TrialSpawnerState;
use crate::net::minecraft::world::level::block::entity::vault::vault_state::VaultState;
use crate::net::minecraft::world::level::block::state::properties::attach_face::AttachFace;
use crate::net::minecraft::world::level::block::state::properties::bamboo_leaves::BambooLeaves;
use crate::net::minecraft::world::level::block::state::properties::bed_part::BedPart;
use crate::net::minecraft::world::level::block::state::properties::bell_attach_type::BellAttachType;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::chest_type::ChestType;
use crate::net::minecraft::world::level::block::state::properties::comparator_mode::ComparatorMode;
use crate::net::minecraft::world::level::block::state::properties::creaking_heart_state::CreakingHeartState;
use crate::net::minecraft::world::level::block::state::properties::door_hinge_side::DoorHingeSide;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::half::Half;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::properties::note_block_instrument::NoteBlockInstrument;
use crate::net::minecraft::world::level::block::state::properties::piston_type::PistonType;
use crate::net::minecraft::world::level::block::state::properties::potent_sulfur_state::PotentSulfurState;
use crate::net::minecraft::world::level::block::state::properties::rail_shape::RailShape;
use crate::net::minecraft::world::level::block::state::properties::redstone_side::RedstoneSide;
use crate::net::minecraft::world::level::block::state::properties::rotation_segment::RotationSegment;
use crate::net::minecraft::world::level::block::state::properties::sculk_sensor_phase::SculkSensorPhase;
use crate::net::minecraft::world::level::block::state::properties::side_chain_part::SideChainPart;
use crate::net::minecraft::world::level::block::state::properties::slab_type::SlabType;
use crate::net::minecraft::world::level::block::state::properties::speleothem_thickness::SpeleothemThickness;
use crate::net::minecraft::world::level::block::state::properties::stairs_shape::StairsShape;
use crate::net::minecraft::world::level::block::state::properties::structure_mode::StructureMode;
use crate::net::minecraft::world::level::block::state::properties::test_block_mode::TestBlockMode;
use crate::net::minecraft::world::level::block::state::properties::tilt::Tilt;
use crate::net::minecraft::world::level::block::state::properties::wall_side::WallSide;

static ATTACHED: BooleanProperty = BooleanProperty::create("attached");
static BERRIES: BooleanProperty = BooleanProperty::create("berries");
static BLOOM: BooleanProperty = BooleanProperty::create("bloom");
static BOTTOM: BooleanProperty = BooleanProperty::create("bottom");
static CAN_SUMMON: BooleanProperty = BooleanProperty::create("can_summon");
static CONDITIONAL: BooleanProperty = BooleanProperty::create("conditional");
static DISARMED: BooleanProperty = BooleanProperty::create("disarmed");
static DRAG: BooleanProperty = BooleanProperty::create("drag");
static ENABLED: BooleanProperty = BooleanProperty::create("enabled");
static EXTENDED: BooleanProperty = BooleanProperty::create("extended");
static EYE: BooleanProperty = BooleanProperty::create("eye");
static FALLING: BooleanProperty = BooleanProperty::create("falling");
static HANGING: BooleanProperty = BooleanProperty::create("hanging");
static HAS_BOTTLE_0: BooleanProperty = BooleanProperty::create("has_bottle_0");
static HAS_BOTTLE_1: BooleanProperty = BooleanProperty::create("has_bottle_1");
static HAS_BOTTLE_2: BooleanProperty = BooleanProperty::create("has_bottle_2");
static HAS_RECORD: BooleanProperty = BooleanProperty::create("has_record");
static HAS_BOOK: BooleanProperty = BooleanProperty::create("has_book");
static INVERTED: BooleanProperty = BooleanProperty::create("inverted");
static IN_WALL: BooleanProperty = BooleanProperty::create("in_wall");
static LIT: BooleanProperty = BooleanProperty::create("lit");
static LOCKED: BooleanProperty = BooleanProperty::create("locked");
static NATURAL: BooleanProperty = BooleanProperty::create("natural");
static OCCUPIED: BooleanProperty = BooleanProperty::create("occupied");
static OPEN: BooleanProperty = BooleanProperty::create("open");
static PERSISTENT: BooleanProperty = BooleanProperty::create("persistent");
static POWERED: BooleanProperty = BooleanProperty::create("powered");
static SHORT: BooleanProperty = BooleanProperty::create("short");
static SHRIEKING: BooleanProperty = BooleanProperty::create("shrieking");
static SIGNAL_FIRE: BooleanProperty = BooleanProperty::create("signal_fire");
static SNOWY: BooleanProperty = BooleanProperty::create("snowy");
static TIP: BooleanProperty = BooleanProperty::create("tip");
static TRIGGERED: BooleanProperty = BooleanProperty::create("triggered");
static UNSTABLE: BooleanProperty = BooleanProperty::create("unstable");
static WATERLOGGED: BooleanProperty = BooleanProperty::create("waterlogged");
static HORIZONTAL_AXIS: EnumProperty<Axis> = EnumProperty::create("axis", &[Axis::X, Axis::Z]);
static AXIS: EnumProperty<Axis> = EnumProperty::create_all("axis");
static UP: BooleanProperty = BooleanProperty::create("up");
static DOWN: BooleanProperty = BooleanProperty::create("down");
static NORTH: BooleanProperty = BooleanProperty::create("north");
static EAST: BooleanProperty = BooleanProperty::create("east");
static SOUTH: BooleanProperty = BooleanProperty::create("south");
static WEST: BooleanProperty = BooleanProperty::create("west");
static FACING: EnumProperty<Direction> = EnumProperty::create(
    "facing",
    &[
        Direction::NORTH,
        Direction::EAST,
        Direction::SOUTH,
        Direction::WEST,
        Direction::UP,
        Direction::DOWN,
    ],
);
// Every direction but up.
static FACING_HOPPER: EnumProperty<Direction> = EnumProperty::create(
    "facing",
    &[
        Direction::DOWN,
        Direction::NORTH,
        Direction::SOUTH,
        Direction::WEST,
        Direction::EAST,
    ],
);
// `Direction.Plane.HORIZONTAL`'s directions, in the order of Direction's constants.
static HORIZONTAL_FACING: EnumProperty<Direction> = EnumProperty::create(
    "facing",
    &[
        Direction::NORTH,
        Direction::SOUTH,
        Direction::WEST,
        Direction::EAST,
    ],
);
static FLOWER_AMOUNT: IntegerProperty = IntegerProperty::create("flower_amount", 1, 4);
static SEGMENT_AMOUNT: IntegerProperty = IntegerProperty::create("segment_amount", 1, 4);
static ORIENTATION: EnumProperty<FrontAndTop> = EnumProperty::create_all("orientation");
static ATTACH_FACE: EnumProperty<AttachFace> = EnumProperty::create_all("face");
static BELL_ATTACHMENT: EnumProperty<BellAttachType> = EnumProperty::create_all("attachment");
static EAST_WALL: EnumProperty<WallSide> = EnumProperty::create_all("east");
static NORTH_WALL: EnumProperty<WallSide> = EnumProperty::create_all("north");
static SOUTH_WALL: EnumProperty<WallSide> = EnumProperty::create_all("south");
static WEST_WALL: EnumProperty<WallSide> = EnumProperty::create_all("west");
static EAST_REDSTONE: EnumProperty<RedstoneSide> = EnumProperty::create_all("east");
static NORTH_REDSTONE: EnumProperty<RedstoneSide> = EnumProperty::create_all("north");
static SOUTH_REDSTONE: EnumProperty<RedstoneSide> = EnumProperty::create_all("south");
static WEST_REDSTONE: EnumProperty<RedstoneSide> = EnumProperty::create_all("west");
static DOUBLE_BLOCK_HALF: EnumProperty<DoubleBlockHalf> = EnumProperty::create_all("half");
static HALF: EnumProperty<Half> = EnumProperty::create_all("half");
static SIDE_CHAIN_PART: EnumProperty<SideChainPart> = EnumProperty::create_all("side_chain");
static RAIL_SHAPE: EnumProperty<RailShape> = EnumProperty::create_all("shape");
// Every shape but the four curves.
static RAIL_SHAPE_STRAIGHT: EnumProperty<RailShape> = EnumProperty::create(
    "shape",
    &[
        RailShape::NORTH_SOUTH,
        RailShape::EAST_WEST,
        RailShape::ASCENDING_EAST,
        RailShape::ASCENDING_WEST,
        RailShape::ASCENDING_NORTH,
        RailShape::ASCENDING_SOUTH,
    ],
);
static AGE_1: IntegerProperty = IntegerProperty::create("age", 0, 1);
static AGE_2: IntegerProperty = IntegerProperty::create("age", 0, 2);
static AGE_3: IntegerProperty = IntegerProperty::create("age", 0, 3);
static AGE_4: IntegerProperty = IntegerProperty::create("age", 0, 4);
static AGE_5: IntegerProperty = IntegerProperty::create("age", 0, 5);
static AGE_7: IntegerProperty = IntegerProperty::create("age", 0, 7);
static AGE_15: IntegerProperty = IntegerProperty::create("age", 0, 15);
static AGE_25: IntegerProperty = IntegerProperty::create("age", 0, 25);
static BITES: IntegerProperty = IntegerProperty::create("bites", 0, 6);
static CANDLES: IntegerProperty = IntegerProperty::create("candles", 1, 4);
static DELAY: IntegerProperty = IntegerProperty::create("delay", 1, 4);
static DISTANCE: IntegerProperty = IntegerProperty::create("distance", 1, 7);
static EGGS: IntegerProperty = IntegerProperty::create("eggs", 1, 4);
static HATCH: IntegerProperty = IntegerProperty::create("hatch", 0, 2);
static LAYERS: IntegerProperty = IntegerProperty::create("layers", 1, 8);
static LEVEL_CAULDRON: IntegerProperty = IntegerProperty::create("level", 1, 3);
static LEVEL_COMPOSTER: IntegerProperty = IntegerProperty::create("level", 0, 8);
static LEVEL_FLOWING: IntegerProperty = IntegerProperty::create("level", 1, 8);
static LEVEL_HONEY: IntegerProperty = IntegerProperty::create("honey_level", 0, 5);
static LEVEL: IntegerProperty = IntegerProperty::create("level", 0, 15);
static MOISTURE: IntegerProperty = IntegerProperty::create("moisture", 0, 7);
static NOTE: IntegerProperty = IntegerProperty::create("note", 0, 24);
static PICKLES: IntegerProperty = IntegerProperty::create("pickles", 1, 4);
static POWER: IntegerProperty = IntegerProperty::create("power", 0, 15);
static STAGE: IntegerProperty = IntegerProperty::create("stage", 0, 1);
static STABILITY_DISTANCE: IntegerProperty = IntegerProperty::create("distance", 0, 7);
static RESPAWN_ANCHOR_CHARGES: IntegerProperty = IntegerProperty::create("charges", 0, 4);
static DRIED_GHAST_HYDRATION_LEVELS: IntegerProperty = IntegerProperty::create("hydration", 0, 3);
static ROTATION_16: IntegerProperty =
    IntegerProperty::create("rotation", 0, RotationSegment::get_max_segment_index());
static BED_PART: EnumProperty<BedPart> = EnumProperty::create_all("part");
static CHEST_TYPE: EnumProperty<ChestType> = EnumProperty::create_all("type");
static MODE_COMPARATOR: EnumProperty<ComparatorMode> = EnumProperty::create_all("mode");
static DOOR_HINGE: EnumProperty<DoorHingeSide> = EnumProperty::create_all("hinge");
static NOTEBLOCK_INSTRUMENT: EnumProperty<NoteBlockInstrument> =
    EnumProperty::create_all("instrument");
static PISTON_TYPE: EnumProperty<PistonType> = EnumProperty::create_all("type");
static SLAB_TYPE: EnumProperty<SlabType> = EnumProperty::create_all("type");
static STAIRS_SHAPE: EnumProperty<StairsShape> = EnumProperty::create_all("shape");
static STRUCTUREBLOCK_MODE: EnumProperty<StructureMode> = EnumProperty::create_all("mode");
static BAMBOO_LEAVES: EnumProperty<BambooLeaves> = EnumProperty::create_all("leaves");
static TILT: EnumProperty<Tilt> = EnumProperty::create_all("tilt");
static VERTICAL_DIRECTION: EnumProperty<Direction> =
    EnumProperty::create("vertical_direction", &[Direction::UP, Direction::DOWN]);
static SPELEOTHEM_THICKNESS: EnumProperty<SpeleothemThickness> =
    EnumProperty::create_all("thickness");
static SCULK_SENSOR_PHASE: EnumProperty<SculkSensorPhase> =
    EnumProperty::create_all("sculk_sensor_phase");
static SLOT_0_OCCUPIED: BooleanProperty = BooleanProperty::create("slot_0_occupied");
static SLOT_1_OCCUPIED: BooleanProperty = BooleanProperty::create("slot_1_occupied");
static SLOT_2_OCCUPIED: BooleanProperty = BooleanProperty::create("slot_2_occupied");
static SLOT_3_OCCUPIED: BooleanProperty = BooleanProperty::create("slot_3_occupied");
static SLOT_4_OCCUPIED: BooleanProperty = BooleanProperty::create("slot_4_occupied");
static SLOT_5_OCCUPIED: BooleanProperty = BooleanProperty::create("slot_5_occupied");
static DUSTED: IntegerProperty = IntegerProperty::create("dusted", 0, 3);
static CRACKED: BooleanProperty = BooleanProperty::create("cracked");
static CRAFTING: BooleanProperty = BooleanProperty::create("crafting");
static TRIAL_SPAWNER_STATE: EnumProperty<TrialSpawnerState> =
    EnumProperty::create_all("trial_spawner_state");
static VAULT_STATE: EnumProperty<VaultState> = EnumProperty::create_all("vault_state");
static CREAKING_HEART_STATE: EnumProperty<CreakingHeartState> =
    EnumProperty::create_all("creaking_heart_state");
static OMINOUS: BooleanProperty = BooleanProperty::create("ominous");
static TEST_BLOCK_MODE: EnumProperty<TestBlockMode> = EnumProperty::create_all("mode");
static MAP: BooleanProperty = BooleanProperty::create("map");
static COPPER_GOLEM_POSE: EnumProperty<Pose> = EnumProperty::create_all("copper_golem_pose");
static POTENT_SULFUR_STATE: EnumProperty<PotentSulfurState> =
    EnumProperty::create_all("potent_sulfur_state");

pub struct BlockStateProperties;

impl BlockStateProperties {
    pub const ATTACHED: &'static BooleanProperty = &ATTACHED;
    pub const BERRIES: &'static BooleanProperty = &BERRIES;
    pub const BLOOM: &'static BooleanProperty = &BLOOM;
    pub const BOTTOM: &'static BooleanProperty = &BOTTOM;
    pub const CAN_SUMMON: &'static BooleanProperty = &CAN_SUMMON;
    pub const CONDITIONAL: &'static BooleanProperty = &CONDITIONAL;
    pub const DISARMED: &'static BooleanProperty = &DISARMED;
    pub const DRAG: &'static BooleanProperty = &DRAG;
    pub const ENABLED: &'static BooleanProperty = &ENABLED;
    pub const EXTENDED: &'static BooleanProperty = &EXTENDED;
    pub const EYE: &'static BooleanProperty = &EYE;
    pub const FALLING: &'static BooleanProperty = &FALLING;
    pub const HANGING: &'static BooleanProperty = &HANGING;
    pub const HAS_BOTTLE_0: &'static BooleanProperty = &HAS_BOTTLE_0;
    pub const HAS_BOTTLE_1: &'static BooleanProperty = &HAS_BOTTLE_1;
    pub const HAS_BOTTLE_2: &'static BooleanProperty = &HAS_BOTTLE_2;
    pub const HAS_RECORD: &'static BooleanProperty = &HAS_RECORD;
    pub const HAS_BOOK: &'static BooleanProperty = &HAS_BOOK;
    pub const INVERTED: &'static BooleanProperty = &INVERTED;
    pub const IN_WALL: &'static BooleanProperty = &IN_WALL;
    pub const LIT: &'static BooleanProperty = &LIT;
    pub const LOCKED: &'static BooleanProperty = &LOCKED;
    pub const NATURAL: &'static BooleanProperty = &NATURAL;
    pub const OCCUPIED: &'static BooleanProperty = &OCCUPIED;
    pub const OPEN: &'static BooleanProperty = &OPEN;
    pub const PERSISTENT: &'static BooleanProperty = &PERSISTENT;
    pub const POWERED: &'static BooleanProperty = &POWERED;
    pub const SHORT: &'static BooleanProperty = &SHORT;
    pub const SHRIEKING: &'static BooleanProperty = &SHRIEKING;
    pub const SIGNAL_FIRE: &'static BooleanProperty = &SIGNAL_FIRE;
    pub const SNOWY: &'static BooleanProperty = &SNOWY;
    pub const TIP: &'static BooleanProperty = &TIP;
    pub const TRIGGERED: &'static BooleanProperty = &TRIGGERED;
    pub const UNSTABLE: &'static BooleanProperty = &UNSTABLE;
    pub const WATERLOGGED: &'static BooleanProperty = &WATERLOGGED;
    pub const HORIZONTAL_AXIS: &'static EnumProperty<Axis> = &HORIZONTAL_AXIS;
    pub const AXIS: &'static EnumProperty<Axis> = &AXIS;
    pub const UP: &'static BooleanProperty = &UP;
    pub const DOWN: &'static BooleanProperty = &DOWN;
    pub const NORTH: &'static BooleanProperty = &NORTH;
    pub const EAST: &'static BooleanProperty = &EAST;
    pub const SOUTH: &'static BooleanProperty = &SOUTH;
    pub const WEST: &'static BooleanProperty = &WEST;
    pub const FACING: &'static EnumProperty<Direction> = &FACING;
    pub const FACING_HOPPER: &'static EnumProperty<Direction> = &FACING_HOPPER;
    pub const HORIZONTAL_FACING: &'static EnumProperty<Direction> = &HORIZONTAL_FACING;
    pub const FLOWER_AMOUNT: &'static IntegerProperty = &FLOWER_AMOUNT;
    pub const SEGMENT_AMOUNT: &'static IntegerProperty = &SEGMENT_AMOUNT;
    pub const ORIENTATION: &'static EnumProperty<FrontAndTop> = &ORIENTATION;
    pub const ATTACH_FACE: &'static EnumProperty<AttachFace> = &ATTACH_FACE;
    pub const BELL_ATTACHMENT: &'static EnumProperty<BellAttachType> = &BELL_ATTACHMENT;
    pub const EAST_WALL: &'static EnumProperty<WallSide> = &EAST_WALL;
    pub const NORTH_WALL: &'static EnumProperty<WallSide> = &NORTH_WALL;
    pub const SOUTH_WALL: &'static EnumProperty<WallSide> = &SOUTH_WALL;
    pub const WEST_WALL: &'static EnumProperty<WallSide> = &WEST_WALL;
    pub const EAST_REDSTONE: &'static EnumProperty<RedstoneSide> = &EAST_REDSTONE;
    pub const NORTH_REDSTONE: &'static EnumProperty<RedstoneSide> = &NORTH_REDSTONE;
    pub const SOUTH_REDSTONE: &'static EnumProperty<RedstoneSide> = &SOUTH_REDSTONE;
    pub const WEST_REDSTONE: &'static EnumProperty<RedstoneSide> = &WEST_REDSTONE;
    pub const DOUBLE_BLOCK_HALF: &'static EnumProperty<DoubleBlockHalf> = &DOUBLE_BLOCK_HALF;
    pub const HALF: &'static EnumProperty<Half> = &HALF;
    pub const SIDE_CHAIN_PART: &'static EnumProperty<SideChainPart> = &SIDE_CHAIN_PART;
    pub const RAIL_SHAPE: &'static EnumProperty<RailShape> = &RAIL_SHAPE;
    pub const RAIL_SHAPE_STRAIGHT: &'static EnumProperty<RailShape> = &RAIL_SHAPE_STRAIGHT;
    pub const AGE_1: &'static IntegerProperty = &AGE_1;
    pub const AGE_2: &'static IntegerProperty = &AGE_2;
    pub const AGE_3: &'static IntegerProperty = &AGE_3;
    pub const AGE_4: &'static IntegerProperty = &AGE_4;
    pub const AGE_5: &'static IntegerProperty = &AGE_5;
    pub const AGE_7: &'static IntegerProperty = &AGE_7;
    pub const AGE_15: &'static IntegerProperty = &AGE_15;
    pub const AGE_25: &'static IntegerProperty = &AGE_25;
    pub const BITES: &'static IntegerProperty = &BITES;
    pub const CANDLES: &'static IntegerProperty = &CANDLES;
    pub const DELAY: &'static IntegerProperty = &DELAY;
    pub const DISTANCE: &'static IntegerProperty = &DISTANCE;
    pub const EGGS: &'static IntegerProperty = &EGGS;
    pub const HATCH: &'static IntegerProperty = &HATCH;
    pub const LAYERS: &'static IntegerProperty = &LAYERS;
    pub const LEVEL_CAULDRON: &'static IntegerProperty = &LEVEL_CAULDRON;
    pub const LEVEL_COMPOSTER: &'static IntegerProperty = &LEVEL_COMPOSTER;
    pub const LEVEL_FLOWING: &'static IntegerProperty = &LEVEL_FLOWING;
    pub const LEVEL_HONEY: &'static IntegerProperty = &LEVEL_HONEY;
    pub const LEVEL: &'static IntegerProperty = &LEVEL;
    pub const MOISTURE: &'static IntegerProperty = &MOISTURE;
    pub const NOTE: &'static IntegerProperty = &NOTE;
    pub const PICKLES: &'static IntegerProperty = &PICKLES;
    pub const POWER: &'static IntegerProperty = &POWER;
    pub const STAGE: &'static IntegerProperty = &STAGE;
    pub const STABILITY_DISTANCE: &'static IntegerProperty = &STABILITY_DISTANCE;
    pub const RESPAWN_ANCHOR_CHARGES: &'static IntegerProperty = &RESPAWN_ANCHOR_CHARGES;
    pub const DRIED_GHAST_HYDRATION_LEVELS: &'static IntegerProperty =
        &DRIED_GHAST_HYDRATION_LEVELS;
    pub const ROTATION_16: &'static IntegerProperty = &ROTATION_16;
    pub const BED_PART: &'static EnumProperty<BedPart> = &BED_PART;
    pub const CHEST_TYPE: &'static EnumProperty<ChestType> = &CHEST_TYPE;
    pub const MODE_COMPARATOR: &'static EnumProperty<ComparatorMode> = &MODE_COMPARATOR;
    pub const DOOR_HINGE: &'static EnumProperty<DoorHingeSide> = &DOOR_HINGE;
    pub const NOTEBLOCK_INSTRUMENT: &'static EnumProperty<NoteBlockInstrument> =
        &NOTEBLOCK_INSTRUMENT;
    pub const PISTON_TYPE: &'static EnumProperty<PistonType> = &PISTON_TYPE;
    pub const SLAB_TYPE: &'static EnumProperty<SlabType> = &SLAB_TYPE;
    pub const STAIRS_SHAPE: &'static EnumProperty<StairsShape> = &STAIRS_SHAPE;
    pub const STRUCTUREBLOCK_MODE: &'static EnumProperty<StructureMode> = &STRUCTUREBLOCK_MODE;
    pub const BAMBOO_LEAVES: &'static EnumProperty<BambooLeaves> = &BAMBOO_LEAVES;
    pub const TILT: &'static EnumProperty<Tilt> = &TILT;
    pub const VERTICAL_DIRECTION: &'static EnumProperty<Direction> = &VERTICAL_DIRECTION;
    pub const SPELEOTHEM_THICKNESS: &'static EnumProperty<SpeleothemThickness> =
        &SPELEOTHEM_THICKNESS;
    pub const SCULK_SENSOR_PHASE: &'static EnumProperty<SculkSensorPhase> = &SCULK_SENSOR_PHASE;
    pub const SLOT_0_OCCUPIED: &'static BooleanProperty = &SLOT_0_OCCUPIED;
    pub const SLOT_1_OCCUPIED: &'static BooleanProperty = &SLOT_1_OCCUPIED;
    pub const SLOT_2_OCCUPIED: &'static BooleanProperty = &SLOT_2_OCCUPIED;
    pub const SLOT_3_OCCUPIED: &'static BooleanProperty = &SLOT_3_OCCUPIED;
    pub const SLOT_4_OCCUPIED: &'static BooleanProperty = &SLOT_4_OCCUPIED;
    pub const SLOT_5_OCCUPIED: &'static BooleanProperty = &SLOT_5_OCCUPIED;
    pub const DUSTED: &'static IntegerProperty = &DUSTED;
    pub const CRACKED: &'static BooleanProperty = &CRACKED;
    pub const CRAFTING: &'static BooleanProperty = &CRAFTING;
    pub const TRIAL_SPAWNER_STATE: &'static EnumProperty<TrialSpawnerState> = &TRIAL_SPAWNER_STATE;
    pub const VAULT_STATE: &'static EnumProperty<VaultState> = &VAULT_STATE;
    pub const CREAKING_HEART_STATE: &'static EnumProperty<CreakingHeartState> =
        &CREAKING_HEART_STATE;
    pub const OMINOUS: &'static BooleanProperty = &OMINOUS;
    pub const TEST_BLOCK_MODE: &'static EnumProperty<TestBlockMode> = &TEST_BLOCK_MODE;
    pub const MAP: &'static BooleanProperty = &MAP;
    pub const COPPER_GOLEM_POSE: &'static EnumProperty<Pose> = &COPPER_GOLEM_POSE;
    pub const POTENT_SULFUR_STATE: &'static EnumProperty<PotentSulfurState> = &POTENT_SULFUR_STATE;
    pub const MAX_AGE_1: i32 = 1;
    pub const MAX_AGE_2: i32 = 2;
    pub const MAX_AGE_3: i32 = 3;
    pub const MAX_AGE_4: i32 = 4;
    pub const MAX_AGE_5: i32 = 5;
    pub const MAX_AGE_7: i32 = 7;
    pub const MAX_AGE_15: i32 = 15;
    pub const MAX_AGE_25: i32 = 25;
    pub const MAX_DISTANCE: i32 = 7;
    pub const MIN_LEVEL: i32 = 0;
    pub const MIN_LEVEL_CAULDRON: i32 = 1;
    pub const MAX_LEVEL_3: i32 = 3;
    pub const MAX_LEVEL_8: i32 = 8;
    pub const MAX_LEVEL_15: i32 = 15;
    pub const STABILITY_MAX_DISTANCE: i32 = 7;
    pub const MIN_RESPAWN_ANCHOR_CHARGES: i32 = 0;
    pub const MAX_RESPAWN_ANCHOR_CHARGES: i32 = 4;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::BlockStateProperties;
    use crate::golden::Golden;
    use crate::net::minecraft::world::level::block::state::properties::property::{self, Property};

    /// Every property field, by name, in declaration order.
    pub(crate) const FIELDS: [(&str, &dyn Property); 124] = [
        ("ATTACHED", BlockStateProperties::ATTACHED),
        ("BERRIES", BlockStateProperties::BERRIES),
        ("BLOOM", BlockStateProperties::BLOOM),
        ("BOTTOM", BlockStateProperties::BOTTOM),
        ("CAN_SUMMON", BlockStateProperties::CAN_SUMMON),
        ("CONDITIONAL", BlockStateProperties::CONDITIONAL),
        ("DISARMED", BlockStateProperties::DISARMED),
        ("DRAG", BlockStateProperties::DRAG),
        ("ENABLED", BlockStateProperties::ENABLED),
        ("EXTENDED", BlockStateProperties::EXTENDED),
        ("EYE", BlockStateProperties::EYE),
        ("FALLING", BlockStateProperties::FALLING),
        ("HANGING", BlockStateProperties::HANGING),
        ("HAS_BOTTLE_0", BlockStateProperties::HAS_BOTTLE_0),
        ("HAS_BOTTLE_1", BlockStateProperties::HAS_BOTTLE_1),
        ("HAS_BOTTLE_2", BlockStateProperties::HAS_BOTTLE_2),
        ("HAS_RECORD", BlockStateProperties::HAS_RECORD),
        ("HAS_BOOK", BlockStateProperties::HAS_BOOK),
        ("INVERTED", BlockStateProperties::INVERTED),
        ("IN_WALL", BlockStateProperties::IN_WALL),
        ("LIT", BlockStateProperties::LIT),
        ("LOCKED", BlockStateProperties::LOCKED),
        ("NATURAL", BlockStateProperties::NATURAL),
        ("OCCUPIED", BlockStateProperties::OCCUPIED),
        ("OPEN", BlockStateProperties::OPEN),
        ("PERSISTENT", BlockStateProperties::PERSISTENT),
        ("POWERED", BlockStateProperties::POWERED),
        ("SHORT", BlockStateProperties::SHORT),
        ("SHRIEKING", BlockStateProperties::SHRIEKING),
        ("SIGNAL_FIRE", BlockStateProperties::SIGNAL_FIRE),
        ("SNOWY", BlockStateProperties::SNOWY),
        ("TIP", BlockStateProperties::TIP),
        ("TRIGGERED", BlockStateProperties::TRIGGERED),
        ("UNSTABLE", BlockStateProperties::UNSTABLE),
        ("WATERLOGGED", BlockStateProperties::WATERLOGGED),
        ("HORIZONTAL_AXIS", BlockStateProperties::HORIZONTAL_AXIS),
        ("AXIS", BlockStateProperties::AXIS),
        ("UP", BlockStateProperties::UP),
        ("DOWN", BlockStateProperties::DOWN),
        ("NORTH", BlockStateProperties::NORTH),
        ("EAST", BlockStateProperties::EAST),
        ("SOUTH", BlockStateProperties::SOUTH),
        ("WEST", BlockStateProperties::WEST),
        ("FACING", BlockStateProperties::FACING),
        ("FACING_HOPPER", BlockStateProperties::FACING_HOPPER),
        ("HORIZONTAL_FACING", BlockStateProperties::HORIZONTAL_FACING),
        ("FLOWER_AMOUNT", BlockStateProperties::FLOWER_AMOUNT),
        ("SEGMENT_AMOUNT", BlockStateProperties::SEGMENT_AMOUNT),
        ("ORIENTATION", BlockStateProperties::ORIENTATION),
        ("ATTACH_FACE", BlockStateProperties::ATTACH_FACE),
        ("BELL_ATTACHMENT", BlockStateProperties::BELL_ATTACHMENT),
        ("EAST_WALL", BlockStateProperties::EAST_WALL),
        ("NORTH_WALL", BlockStateProperties::NORTH_WALL),
        ("SOUTH_WALL", BlockStateProperties::SOUTH_WALL),
        ("WEST_WALL", BlockStateProperties::WEST_WALL),
        ("EAST_REDSTONE", BlockStateProperties::EAST_REDSTONE),
        ("NORTH_REDSTONE", BlockStateProperties::NORTH_REDSTONE),
        ("SOUTH_REDSTONE", BlockStateProperties::SOUTH_REDSTONE),
        ("WEST_REDSTONE", BlockStateProperties::WEST_REDSTONE),
        ("DOUBLE_BLOCK_HALF", BlockStateProperties::DOUBLE_BLOCK_HALF),
        ("HALF", BlockStateProperties::HALF),
        ("SIDE_CHAIN_PART", BlockStateProperties::SIDE_CHAIN_PART),
        ("RAIL_SHAPE", BlockStateProperties::RAIL_SHAPE),
        (
            "RAIL_SHAPE_STRAIGHT",
            BlockStateProperties::RAIL_SHAPE_STRAIGHT,
        ),
        ("AGE_1", BlockStateProperties::AGE_1),
        ("AGE_2", BlockStateProperties::AGE_2),
        ("AGE_3", BlockStateProperties::AGE_3),
        ("AGE_4", BlockStateProperties::AGE_4),
        ("AGE_5", BlockStateProperties::AGE_5),
        ("AGE_7", BlockStateProperties::AGE_7),
        ("AGE_15", BlockStateProperties::AGE_15),
        ("AGE_25", BlockStateProperties::AGE_25),
        ("BITES", BlockStateProperties::BITES),
        ("CANDLES", BlockStateProperties::CANDLES),
        ("DELAY", BlockStateProperties::DELAY),
        ("DISTANCE", BlockStateProperties::DISTANCE),
        ("EGGS", BlockStateProperties::EGGS),
        ("HATCH", BlockStateProperties::HATCH),
        ("LAYERS", BlockStateProperties::LAYERS),
        ("LEVEL_CAULDRON", BlockStateProperties::LEVEL_CAULDRON),
        ("LEVEL_COMPOSTER", BlockStateProperties::LEVEL_COMPOSTER),
        ("LEVEL_FLOWING", BlockStateProperties::LEVEL_FLOWING),
        ("LEVEL_HONEY", BlockStateProperties::LEVEL_HONEY),
        ("LEVEL", BlockStateProperties::LEVEL),
        ("MOISTURE", BlockStateProperties::MOISTURE),
        ("NOTE", BlockStateProperties::NOTE),
        ("PICKLES", BlockStateProperties::PICKLES),
        ("POWER", BlockStateProperties::POWER),
        ("STAGE", BlockStateProperties::STAGE),
        (
            "STABILITY_DISTANCE",
            BlockStateProperties::STABILITY_DISTANCE,
        ),
        (
            "RESPAWN_ANCHOR_CHARGES",
            BlockStateProperties::RESPAWN_ANCHOR_CHARGES,
        ),
        (
            "DRIED_GHAST_HYDRATION_LEVELS",
            BlockStateProperties::DRIED_GHAST_HYDRATION_LEVELS,
        ),
        ("ROTATION_16", BlockStateProperties::ROTATION_16),
        ("BED_PART", BlockStateProperties::BED_PART),
        ("CHEST_TYPE", BlockStateProperties::CHEST_TYPE),
        ("MODE_COMPARATOR", BlockStateProperties::MODE_COMPARATOR),
        ("DOOR_HINGE", BlockStateProperties::DOOR_HINGE),
        (
            "NOTEBLOCK_INSTRUMENT",
            BlockStateProperties::NOTEBLOCK_INSTRUMENT,
        ),
        ("PISTON_TYPE", BlockStateProperties::PISTON_TYPE),
        ("SLAB_TYPE", BlockStateProperties::SLAB_TYPE),
        ("STAIRS_SHAPE", BlockStateProperties::STAIRS_SHAPE),
        (
            "STRUCTUREBLOCK_MODE",
            BlockStateProperties::STRUCTUREBLOCK_MODE,
        ),
        ("BAMBOO_LEAVES", BlockStateProperties::BAMBOO_LEAVES),
        ("TILT", BlockStateProperties::TILT),
        (
            "VERTICAL_DIRECTION",
            BlockStateProperties::VERTICAL_DIRECTION,
        ),
        (
            "SPELEOTHEM_THICKNESS",
            BlockStateProperties::SPELEOTHEM_THICKNESS,
        ),
        (
            "SCULK_SENSOR_PHASE",
            BlockStateProperties::SCULK_SENSOR_PHASE,
        ),
        ("SLOT_0_OCCUPIED", BlockStateProperties::SLOT_0_OCCUPIED),
        ("SLOT_1_OCCUPIED", BlockStateProperties::SLOT_1_OCCUPIED),
        ("SLOT_2_OCCUPIED", BlockStateProperties::SLOT_2_OCCUPIED),
        ("SLOT_3_OCCUPIED", BlockStateProperties::SLOT_3_OCCUPIED),
        ("SLOT_4_OCCUPIED", BlockStateProperties::SLOT_4_OCCUPIED),
        ("SLOT_5_OCCUPIED", BlockStateProperties::SLOT_5_OCCUPIED),
        ("DUSTED", BlockStateProperties::DUSTED),
        ("CRACKED", BlockStateProperties::CRACKED),
        ("CRAFTING", BlockStateProperties::CRAFTING),
        (
            "TRIAL_SPAWNER_STATE",
            BlockStateProperties::TRIAL_SPAWNER_STATE,
        ),
        ("VAULT_STATE", BlockStateProperties::VAULT_STATE),
        (
            "CREAKING_HEART_STATE",
            BlockStateProperties::CREAKING_HEART_STATE,
        ),
        ("OMINOUS", BlockStateProperties::OMINOUS),
        ("TEST_BLOCK_MODE", BlockStateProperties::TEST_BLOCK_MODE),
        ("MAP", BlockStateProperties::MAP),
        ("COPPER_GOLEM_POSE", BlockStateProperties::COPPER_GOLEM_POSE),
        (
            "POTENT_SULFUR_STATE",
            BlockStateProperties::POTENT_SULFUR_STATE,
        ),
    ];

    /// The property in the field named, as the golden files name properties.
    pub(crate) fn property(field: &str) -> &'static dyn Property {
        FIELDS
            .iter()
            .find(|(name, _)| *name == field)
            .unwrap_or_else(|| panic!("no BlockStateProperties field {field}"))
            .1
    }

    /// The name of the field holding the property.
    pub(crate) fn field_name(target: &dyn Property) -> &'static str {
        FIELDS
            .iter()
            .find(|(_, candidate)| property::same(*candidate, target))
            .unwrap_or_else(|| panic!("{target} isn't a BlockStateProperties field"))
            .0
    }

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.world.level.block.state.properties.BlockStateProperties");
        for (name, property) in FIELDS {
            let class = property.to_string();
            let class = &class[..class.find('{').unwrap()];
            golden.field(
                name,
                (
                    class.to_owned(),
                    property.get_name().to_owned(),
                    property.get_possible_values(),
                ),
            );
        }
        golden.field("MAX_AGE_1", BlockStateProperties::MAX_AGE_1);
        golden.field("MAX_AGE_2", BlockStateProperties::MAX_AGE_2);
        golden.field("MAX_AGE_3", BlockStateProperties::MAX_AGE_3);
        golden.field("MAX_AGE_4", BlockStateProperties::MAX_AGE_4);
        golden.field("MAX_AGE_5", BlockStateProperties::MAX_AGE_5);
        golden.field("MAX_AGE_7", BlockStateProperties::MAX_AGE_7);
        golden.field("MAX_AGE_15", BlockStateProperties::MAX_AGE_15);
        golden.field("MAX_AGE_25", BlockStateProperties::MAX_AGE_25);
        golden.field("MAX_DISTANCE", BlockStateProperties::MAX_DISTANCE);
        golden.field("MIN_LEVEL", BlockStateProperties::MIN_LEVEL);
        golden.field(
            "MIN_LEVEL_CAULDRON",
            BlockStateProperties::MIN_LEVEL_CAULDRON,
        );
        golden.field("MAX_LEVEL_3", BlockStateProperties::MAX_LEVEL_3);
        golden.field("MAX_LEVEL_8", BlockStateProperties::MAX_LEVEL_8);
        golden.field("MAX_LEVEL_15", BlockStateProperties::MAX_LEVEL_15);
        golden.field(
            "STABILITY_MAX_DISTANCE",
            BlockStateProperties::STABILITY_MAX_DISTANCE,
        );
        golden.field(
            "MIN_RESPAWN_ANCHOR_CHARGES",
            BlockStateProperties::MIN_RESPAWN_ANCHOR_CHARGES,
        );
        golden.field(
            "MAX_RESPAWN_ANCHOR_CHARGES",
            BlockStateProperties::MAX_RESPAWN_ANCHOR_CHARGES,
        );
        golden.finish();
    }
}
