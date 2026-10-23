//! net/minecraft/world/level/block/state/BlockState.java
//!
//! Not ported yet: `FULL_CODEC` and `CODEC`, which wait for DataFixerUpper's codecs.
//!
//! A state is a reference to its block's `StateDefinition` and its index there: `Copy`, 16 bytes,
//! and answering for its values without a registry, so states work while `Blocks` is still
//! registering. Its id in `Block.BLOCK_STATE_REGISTRY` is its block's first id plus the index.
//! Java compares states by reference, which here is comparing the definition and the index. What
//! `BlockStateBase` keeps of each state (its light emission, whether it occludes…) is in the
//! registry's table, by id (`block_behaviour.rs`).

use std::fmt;
use std::hash::{Hash, Hasher};

use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

/// One of a block's states: the block and a value for each of its properties.
#[derive(Clone, Copy)]
pub struct BlockState {
    definition: &'static StateDefinition,
    index: u32,
}

impl BlockState {
    pub(crate) fn new(definition: &'static StateDefinition, index: u32) -> BlockState {
        BlockState { definition, index }
    }

    pub fn get_block(&self) -> Block {
        self.definition.get_owner()
    }

    /// Whether it's one of the block's states.
    pub fn is(&self, block: Block) -> bool {
        self.get_block() == block
    }

    /// Its id in `Block.BLOCK_STATE_REGISTRY`, which `Block.getId` and the network use.
    pub fn id(&self) -> u32 {
        self.definition.first_id() + self.index
    }
}

impl StateHolder for BlockState {
    fn definition(&self) -> &'static StateDefinition {
        self.definition
    }

    fn index(&self) -> u32 {
        self.index
    }

    fn with_index(&self, index: u32) -> BlockState {
        BlockState::new(self.definition, index)
    }
}

impl PartialEq for BlockState {
    fn eq(&self, other: &BlockState) -> bool {
        std::ptr::eq(self.definition, other.definition) && self.index == other.index
    }
}

impl Eq for BlockState {}

impl Hash for BlockState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.definition, state);
        self.index.hash(state);
    }
}

/// `StateHolder.toString()`.
impl fmt::Display for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_state(f)
    }
}

impl fmt::Debug for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_state(f)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::BlockState;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
    use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block::state::properties::block_state_properties::tests::{
        field_name, property,
    };
    use crate::net::minecraft::world::level::block::state::properties::property::tests::value;
    use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

    /// A state is its id in `Block.BLOCK_STATE_REGISTRY`.
    impl Encode for BlockState {
        fn encode(&self) -> Value {
            Value::from(self.id())
        }
    }

    impl Decode for BlockState {
        fn decode(value: &Value) -> BlockState {
            let id = i32::decode(value);
            Block::block_state_registry()
                .by_id(id)
                .unwrap_or_else(|| panic!("no state {id}"))
        }
    }

    // Where vanilla asks the functions taking a level and a position.
    const POS: BlockPos = BlockPos::new(1, 2, 3);

    const PROPERTY: &str = "Lnet/minecraft/world/level/block/state/properties/Property;";
    const GETTER_AND_POS: &str =
        "(Lnet/minecraft/world/level/BlockGetter;Lnet/minecraft/core/BlockPos;)";

    /// `"BLOCK"`, `"EMPTY"` or `"OTHER"`, as the suite tells shapes apart.
    fn shape_kind(shape: &VoxelShape) -> &'static str {
        if *shape == Shapes::block() {
            "BLOCK"
        } else if *shape == Shapes::empty() {
            "EMPTY"
        } else {
            "OTHER"
        }
    }

    /// The states' light, `initCache`'s, for every state.
    fn check_light(golden: &Golden) {
        for (args, expected) in golden.cases("light") {
            let state = BlockState::decode(&args[0]);
            let faces: Vec<&str> = Direction::VALUES
                .iter()
                .map(|&direction| shape_kind(&state.get_face_occlusion_shape(direction)))
                .collect();
            let actual = serde_json::json!([
                state.use_shape_for_light_occlusion(),
                state.propagates_skylight_down(),
                state.get_light_dampening(),
                state.is_solid_render(),
                shape_kind(&state.get_occlusion_shape()),
                faces,
            ]);
            assert_eq!(actual, expected, "the light of {state} ({})", state.id());
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.block.state.BlockState");
        golden.check(
            "getBlock()Lnet/minecraft/world/level/block/Block;",
            |state: BlockState| state.get_block(),
        );
        golden.check(
            "getValues()Ljava/util/stream/Stream;",
            |state: BlockState| {
                state
                    .get_values()
                    .into_iter()
                    .map(|(property, value)| (field_name(property).to_owned(), value))
                    .collect::<Vec<_>>()
            },
        );
        golden.check("toString()Ljava/lang/String;", |state: BlockState| {
            state.to_string()
        });
        golden.check("getLightEmission()I", |state: BlockState| {
            state.get_light_emission()
        });
        golden.check("isAir()Z", |state: BlockState| state.is_air());
        golden.check("ignitedByLava()Z", |state: BlockState| {
            state.ignited_by_lava()
        });
        golden.check("liquid()Z", |state: BlockState| state.liquid());
        golden.check(
            "getPistonPushReaction()Lnet/minecraft/world/level/material/PushReaction;",
            |state: BlockState| format!("{:?}", state.get_piston_push_reaction()),
        );
        golden.check(
            &format!("getDestroySpeed{GETTER_AND_POS}F"),
            |state: BlockState| state.get_destroy_speed(),
        );
        golden.check("requiresCorrectToolForDrops()Z", |state: BlockState| {
            state.requires_correct_tool_for_drops()
        });
        golden.check("canOcclude()Z", |state: BlockState| state.can_occlude());
        golden.check("isSolid()Z", |state: BlockState| state.is_solid());
        check_light(&golden);
        golden.check("emissiveRendering()Z", |state: BlockState| {
            state.emissive_rendering()
        });
        golden.check("canBeReplaced()Z", |state: BlockState| {
            state.can_be_replaced()
        });
        golden.check("shouldSpawnTerrainParticles()Z", |state: BlockState| {
            state.should_spawn_terrain_particles()
        });
        golden.check(
            "instrument()Lnet/minecraft/world/level/block/state/properties/NoteBlockInstrument;",
            |state: BlockState| format!("{:?}", state.instrument()),
        );
        golden.check("isRandomlyTicking()Z", |state: BlockState| {
            state.is_randomly_ticking()
        });
        golden.check("hasOffsetFunction()Z", |state: BlockState| {
            state.has_offset_function()
        });
        golden.check("hasBlockEntity()Z", |state: BlockState| {
            state.has_block_entity()
        });
        golden.check(
            "getFluidState()Lnet/minecraft/world/level/material/FluidState;",
            |state: BlockState| state.get_fluid_state(),
        );
        golden.check(
            &format!("getMapColor{GETTER_AND_POS}Lnet/minecraft/world/level/material/MapColor;"),
            |state: BlockState| state.get_map_color().id,
        );
        golden.check(
            &format!("getPostProcessPos{GETTER_AND_POS}Lnet/minecraft/core/BlockPos;"),
            |state: BlockState| {
                state
                    .get_post_process_pos(POS)
                    .map(|pos| (pos.get_x(), pos.get_y(), pos.get_z()))
            },
        );
        golden.check(
            &format!("isRedstoneConductor{GETTER_AND_POS}Z"),
            |state: BlockState| state.is_redstone_conductor(),
        );
        golden.check(
            &format!("isSuffocating{GETTER_AND_POS}Z"),
            |state: BlockState| state.is_suffocating(),
        );
        golden.check(
            "isViewBlocking(Lnet/minecraft/world/level/BlockGetter;Lnet/minecraft/core/BlockPos;Lnet/minecraft/world/phys/AABB;)Z",
            |state: BlockState| state.is_view_blocking(),
        );

        golden.check(
            &format!("cycle({PROPERTY})Ljava/lang/Object;"),
            |state: BlockState, field: String| state.cycle(property(&field)),
        );
        golden.check(
            &format!("setValue({PROPERTY}Ljava/lang/Comparable;)Ljava/lang/Object;[every value]"),
            |state: BlockState, field: String| {
                let property = property(&field);
                property
                    .get_possible_values()
                    .into_iter()
                    .map(|value| state.set_comparable(property, value))
                    .collect::<Vec<_>>()
            },
        );
        golden.check(
            &format!("getValue({PROPERTY})Ljava/lang/Comparable;"),
            |state: BlockState, field: String| state.get_comparable(property(&field)),
        );
        golden.check(
            &format!("hasProperty({PROPERTY})Z"),
            |state: BlockState, field: String| state.has_property(property(&field)),
        );
        golden.check(
            &format!("getOptionalValue({PROPERTY})Ljava/util/Optional;"),
            |state: BlockState, field: String| state.get_optional_comparable(property(&field)),
        );
        golden.check(
            &format!("getValueOrElse({PROPERTY}Ljava/lang/Comparable;)Ljava/lang/Comparable;"),
            |state: BlockState, field: String, default_value: Value| {
                let property = property(&field);
                state
                    .get_optional_comparable(property)
                    .unwrap_or_else(|| value(property, &default_value))
            },
        );
        golden.check(
            &format!("setValue({PROPERTY}Ljava/lang/Comparable;)Ljava/lang/Object;"),
            |state: BlockState, field: String, encoded: Value| {
                let property = property(&field);
                state.set_comparable(property, value(property, &encoded))
            },
        );
        golden.check(
            &format!("trySetValue({PROPERTY}Ljava/lang/Comparable;)Ljava/lang/Object;"),
            |state: BlockState, field: String, encoded: Value| {
                let property = property(&field);
                state.try_set_comparable(property, value(property, &encoded))
            },
        );
        golden.check(
            &format!("cycle({PROPERTY})Ljava/lang/Object;[probe]"),
            |state: BlockState, field: String| state.cycle(property(&field)),
        );
        golden.finish();
    }
}
