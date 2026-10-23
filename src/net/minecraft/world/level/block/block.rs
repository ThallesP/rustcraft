//! net/minecraft/world/level/block/Block.java
//!
//! Only what makes a block and its states, and what it keeps from its `Properties`: its state
//! definition and default state, `BLOCK_STATE_REGISTRY` with `getId` and `stateById`, the
//! getters of what `BlockBehaviour` copied (`getFriction`, `getExplosionResistance`…),
//! `toString`, and the shapes' helpers (`box`, `column`, `boxZ`, `cube`, `isShapeFullBlock`,
//! `getShapeForEachState`: the player track's). Not ported yet: every behaviour method
//! (`playerDestroy`, `fallOn`, `getStateForPlacement`…), `shouldRenderFace`, `asItem`, `getName`
//! and `withPropertiesOf`, which come with what calls them.
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

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::id_mapper::IdMapper;
use crate::net::minecraft::world::level::block::blocks::{self, Blocks, REGISTRY};
use crate::net::minecraft::world::level::block::state::block_behaviour::{
    BlockBehaviour, Properties,
};
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_definition::{
    Builder, StateDefinition,
};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
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

/// A class's `getShape` or `getCollisionShape`: the shape of its block's state at a position of
/// a level, for what asks.
pub type ShapeMethod =
    fn(&BlockData, BlockState, &dyn BlockGetter, &BlockPos, &dyn CollisionContext) -> VoxelShape;

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
        }
    }

    /// `instanceof EntityBlock`: it or a parent implements it.
    pub fn is_entity_block(&self) -> bool {
        self.entity_block || self.parent.is_some_and(BlockClass::is_entity_block)
    }

    fn find<T>(&'static self, method: impl Fn(&'static BlockClass) -> Option<T>) -> Option<T> {
        method(self).or_else(|| self.parent.and_then(|parent| parent.find(method)))
    }
}

/// `Block` itself.
pub static CLASS: BlockClass = BlockClass {
    name: "net.minecraft.world.level.block.Block",
    parent: None,
    entity_block: false,
    is_randomly_ticking: None,
    get_fluid_state: None,
    get_shape: None,
    get_collision_shape: None,
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

    /// `cube(double, double, double)`: a box centered every way.
    pub fn cube_xyz(size_x: f64, size_y: f64, size_z: f64) -> VoxelShape {
        let half_y = size_y / 2.0;
        Block::column_xz(size_x, size_z, 8.0 - half_y, 8.0 + half_y)
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
        let occluder = neighbor_state.get_face_occlusion_shape(direction.get_opposite());
        if occluder == Shapes::block() {
            return false;
        }

        if state.skip_rendering(neighbor_state, direction) {
            return false;
        }

        if occluder == Shapes::empty() {
            return true;
        }

        let shape = state.get_face_occlusion_shape(direction);
        if shape == Shapes::empty() {
            return true;
        }

        Shapes::join_is_not_empty(&shape, &occluder, BooleanOp::ONLY_FIRST)
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
            StatePredicate::Default => "default".to_owned(),
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
