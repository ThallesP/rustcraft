//! net/minecraft/network/syncher/EntityDataSerializer.java
//!
//! Java's is an interface, each serializer a lambda over its codec, and a value an `Object` of
//! the serializer's type. Here a serializer is a constant naming it (`EntityDataSerializers`
//! holds them), and a value is `EntityDataValue`, a variant for each serializer's type, which
//! says its serializer. `createAccessor`, `copy` and `ForValueType` come with the entities'
//! data (`EntityDataAccessor`, `SynchedEntityData.DataItem`): only `ITEM_STACK`'s `copy` copies.
//!
//! The serializers whose values aren't ported yet (components, item stacks, particles, villager
//! data, entity references, the armadillo's, sniffer's, copper's and copper golem's states,
//! paintings, profiles and dye colours) read as `EntityDataValue::Unported`: like an `Unported`
//! packet, the rest of the frame is skipped, since nothing says how long the value is.

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::float::Float;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::global_pos::GlobalPos;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::core::rotations::Rotations;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::network::var_int::VarInt;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::entity::humanoid_arm::HumanoidArm;
use crate::net::minecraft::world::entity::pose::Pose;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

/// `EntityDataSerializer`: one of `EntityDataSerializers`' constants, by its id there.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntityDataSerializer {
    BYTE,
    INT,
    LONG,
    FLOAT,
    STRING,
    COMPONENT,
    OPTIONAL_COMPONENT,
    ITEM_STACK,
    BOOLEAN,
    ROTATIONS,
    BLOCK_POS,
    OPTIONAL_BLOCK_POS,
    DIRECTION,
    OPTIONAL_LIVING_ENTITY_REFERENCE,
    BLOCK_STATE,
    OPTIONAL_BLOCK_STATE,
    PARTICLE,
    PARTICLES,
    VILLAGER_DATA,
    OPTIONAL_UNSIGNED_INT,
    POSE,
    CAT_VARIANT,
    CAT_SOUND_VARIANT,
    COW_VARIANT,
    COW_SOUND_VARIANT,
    WOLF_VARIANT,
    WOLF_SOUND_VARIANT,
    FROG_VARIANT,
    PIG_VARIANT,
    PIG_SOUND_VARIANT,
    CHICKEN_VARIANT,
    CHICKEN_SOUND_VARIANT,
    ZOMBIE_NAUTILUS_VARIANT,
    OPTIONAL_GLOBAL_POS,
    PAINTING_VARIANT,
    SNIFFER_STATE,
    ARMADILLO_STATE,
    COPPER_GOLEM_STATE,
    WEATHERING_COPPER_STATE,
    VECTOR3,
    QUATERNION,
    RESOLVABLE_PROFILE,
    HUMANOID_ARM,
    DYE_COLOR,
}

/// A value of one of the serializers' types: Java's `Object`, whose class the serializer's type
/// is.
#[derive(Clone, Debug)]
pub enum EntityDataValue {
    Byte(i8),
    Int(i32),
    Long(i64),
    Float(f32),
    String(String),
    Boolean(bool),
    Rotations(Rotations),
    BlockPos(BlockPos),
    OptionalBlockPos(Option<BlockPos>),
    Direction(Direction),
    BlockState(BlockState),
    OptionalBlockState(Option<BlockState>),
    OptionalUnsignedInt(Option<i32>),
    Pose(Pose),
    /// A holder of one of the variants' registries (cats', cows', wolves', …), which its
    /// serializer says.
    Variant(EntityDataSerializer, Holder),
    OptionalGlobalPos(Option<GlobalPos>),
    Vector3(Vector3f),
    Quaternion(Quaternionf),
    HumanoidArm(HumanoidArm),
    /// A value of a serializer whose type isn't ported yet, whose bytes were skipped with the
    /// rest of the frame.
    Unported(EntityDataSerializer),
}

impl EntityDataValue {
    /// The serializer of the value's type.
    pub fn serializer(&self) -> EntityDataSerializer {
        match self {
            EntityDataValue::Byte(_) => EntityDataSerializer::BYTE,
            EntityDataValue::Int(_) => EntityDataSerializer::INT,
            EntityDataValue::Long(_) => EntityDataSerializer::LONG,
            EntityDataValue::Float(_) => EntityDataSerializer::FLOAT,
            EntityDataValue::String(_) => EntityDataSerializer::STRING,
            EntityDataValue::Boolean(_) => EntityDataSerializer::BOOLEAN,
            EntityDataValue::Rotations(_) => EntityDataSerializer::ROTATIONS,
            EntityDataValue::BlockPos(_) => EntityDataSerializer::BLOCK_POS,
            EntityDataValue::OptionalBlockPos(_) => EntityDataSerializer::OPTIONAL_BLOCK_POS,
            EntityDataValue::Direction(_) => EntityDataSerializer::DIRECTION,
            EntityDataValue::BlockState(_) => EntityDataSerializer::BLOCK_STATE,
            EntityDataValue::OptionalBlockState(_) => EntityDataSerializer::OPTIONAL_BLOCK_STATE,
            EntityDataValue::OptionalUnsignedInt(_) => EntityDataSerializer::OPTIONAL_UNSIGNED_INT,
            EntityDataValue::Pose(_) => EntityDataSerializer::POSE,
            EntityDataValue::Variant(serializer, _) => *serializer,
            EntityDataValue::OptionalGlobalPos(_) => EntityDataSerializer::OPTIONAL_GLOBAL_POS,
            EntityDataValue::Vector3(_) => EntityDataSerializer::VECTOR3,
            EntityDataValue::Quaternion(_) => EntityDataSerializer::QUATERNION,
            EntityDataValue::HumanoidArm(_) => EntityDataSerializer::HUMANOID_ARM,
            EntityDataValue::Unported(serializer) => *serializer,
        }
    }
}

/// Java's `equals` of the values: a float's (and a vector's or quaternion's components') by its
/// bits, NaNs made one, like `Float.equals`.
impl PartialEq for EntityDataValue {
    fn eq(&self, other: &EntityDataValue) -> bool {
        let bits = Float::float_to_int_bits;
        match (self, other) {
            (EntityDataValue::Byte(a), EntityDataValue::Byte(b)) => a == b,
            (EntityDataValue::Int(a), EntityDataValue::Int(b)) => a == b,
            (EntityDataValue::Long(a), EntityDataValue::Long(b)) => a == b,
            (EntityDataValue::Float(a), EntityDataValue::Float(b)) => bits(*a) == bits(*b),
            (EntityDataValue::String(a), EntityDataValue::String(b)) => a == b,
            (EntityDataValue::Boolean(a), EntityDataValue::Boolean(b)) => a == b,
            (EntityDataValue::Rotations(a), EntityDataValue::Rotations(b)) => a == b,
            (EntityDataValue::BlockPos(a), EntityDataValue::BlockPos(b)) => a == b,
            (EntityDataValue::OptionalBlockPos(a), EntityDataValue::OptionalBlockPos(b)) => a == b,
            (EntityDataValue::Direction(a), EntityDataValue::Direction(b)) => a == b,
            (EntityDataValue::BlockState(a), EntityDataValue::BlockState(b)) => a == b,
            (EntityDataValue::OptionalBlockState(a), EntityDataValue::OptionalBlockState(b)) => {
                a == b
            }
            (EntityDataValue::OptionalUnsignedInt(a), EntityDataValue::OptionalUnsignedInt(b)) => {
                a == b
            }
            (EntityDataValue::Pose(a), EntityDataValue::Pose(b)) => a == b,
            (EntityDataValue::Variant(_, a), EntityDataValue::Variant(_, b)) => a == b,
            (EntityDataValue::OptionalGlobalPos(a), EntityDataValue::OptionalGlobalPos(b)) => {
                a == b
            }
            (EntityDataValue::Vector3(a), EntityDataValue::Vector3(b)) => {
                [a.x, a.y, a.z].map(bits) == [b.x, b.y, b.z].map(bits)
            }
            (EntityDataValue::Quaternion(a), EntityDataValue::Quaternion(b)) => {
                [a.x, a.y, a.z, a.w].map(bits) == [b.x, b.y, b.z, b.w].map(bits)
            }
            (EntityDataValue::HumanoidArm(a), EntityDataValue::HumanoidArm(b)) => a == b,
            (EntityDataValue::Unported(a), EntityDataValue::Unported(b)) => a == b,
            _ => false,
        }
    }
}

impl EntityDataSerializer {
    /// The registry a variant serializer's holders are of, for the ones that are a
    /// `holderRegistry`.
    fn variant_registry(self) -> Option<&'static ResourceKey> {
        let registry = match self {
            EntityDataSerializer::CAT_VARIANT => Registries::CAT_VARIANT,
            EntityDataSerializer::CAT_SOUND_VARIANT => Registries::CAT_SOUND_VARIANT,
            EntityDataSerializer::COW_VARIANT => Registries::COW_VARIANT,
            EntityDataSerializer::COW_SOUND_VARIANT => Registries::COW_SOUND_VARIANT,
            EntityDataSerializer::WOLF_VARIANT => Registries::WOLF_VARIANT,
            EntityDataSerializer::WOLF_SOUND_VARIANT => Registries::WOLF_SOUND_VARIANT,
            EntityDataSerializer::FROG_VARIANT => Registries::FROG_VARIANT,
            EntityDataSerializer::PIG_VARIANT => Registries::PIG_VARIANT,
            EntityDataSerializer::PIG_SOUND_VARIANT => Registries::PIG_SOUND_VARIANT,
            EntityDataSerializer::CHICKEN_VARIANT => Registries::CHICKEN_VARIANT,
            EntityDataSerializer::CHICKEN_SOUND_VARIANT => Registries::CHICKEN_SOUND_VARIANT,
            EntityDataSerializer::ZOMBIE_NAUTILUS_VARIANT => Registries::ZOMBIE_NAUTILUS_VARIANT,
            _ => return None,
        };
        Some(registry)
    }

    /// `codec().decode(input)`: a value of the serializer's type. One whose type isn't ported
    /// skips the rest of the frame (above).
    pub fn decode(self, input: &mut RegistryFriendlyByteBuf) -> EntityDataValue {
        if let Some(registry) = self.variant_registry() {
            return EntityDataValue::Variant(
                self,
                ByteBufCodecs::holder_registry(registry).decode(input),
            );
        }
        let buffer: &mut ByteBuf = input.as_mut();
        match self {
            EntityDataSerializer::BYTE => EntityDataValue::Byte(ByteBufCodecs::BYTE.decode(buffer)),
            EntityDataSerializer::INT => {
                EntityDataValue::Int(ByteBufCodecs::VAR_INT.decode(buffer))
            }
            EntityDataSerializer::LONG => {
                EntityDataValue::Long(ByteBufCodecs::VAR_LONG.decode(buffer))
            }
            EntityDataSerializer::FLOAT => {
                EntityDataValue::Float(ByteBufCodecs::FLOAT.decode(buffer))
            }
            EntityDataSerializer::STRING => {
                EntityDataValue::String(ByteBufCodecs::STRING_UTF8.decode(buffer))
            }
            EntityDataSerializer::BOOLEAN => {
                EntityDataValue::Boolean(ByteBufCodecs::BOOL.decode(buffer))
            }
            EntityDataSerializer::ROTATIONS => {
                EntityDataValue::Rotations(Rotations::STREAM_CODEC.decode(buffer))
            }
            EntityDataSerializer::BLOCK_POS => {
                EntityDataValue::BlockPos(BlockPos::STREAM_CODEC.decode(buffer))
            }
            EntityDataSerializer::OPTIONAL_BLOCK_POS => EntityDataValue::OptionalBlockPos(
                ByteBufCodecs::optional(BlockPos::STREAM_CODEC).decode(buffer),
            ),
            EntityDataSerializer::DIRECTION => {
                EntityDataValue::Direction(Direction::STREAM_CODEC.decode(buffer))
            }
            // idMapper(Block.BLOCK_STATE_REGISTRY): byIdOrThrow.
            EntityDataSerializer::BLOCK_STATE => EntityDataValue::BlockState(
                Block::block_state_registry().by_id_or_throw(VarInt::read(buffer)),
            ),
            // OPTIONAL_BLOCK_STATE_CODEC: 0 for none, and stateById for the rest.
            EntityDataSerializer::OPTIONAL_BLOCK_STATE => {
                let id = VarInt::read(buffer);
                EntityDataValue::OptionalBlockState((id != 0).then(|| Block::state_by_id(id)))
            }
            // OPTIONAL_UNSIGNED_INT_CODEC: 0 for none, and one more than the value.
            EntityDataSerializer::OPTIONAL_UNSIGNED_INT => {
                let value = VarInt::read(buffer);
                EntityDataValue::OptionalUnsignedInt((value != 0).then(|| value.wrapping_sub(1)))
            }
            EntityDataSerializer::POSE => EntityDataValue::Pose(Pose::STREAM_CODEC.decode(buffer)),
            EntityDataSerializer::OPTIONAL_GLOBAL_POS => EntityDataValue::OptionalGlobalPos(
                ByteBufCodecs::optional(GlobalPos::STREAM_CODEC).decode(buffer),
            ),
            EntityDataSerializer::VECTOR3 => {
                EntityDataValue::Vector3(ByteBufCodecs::VECTOR3F.decode(buffer))
            }
            EntityDataSerializer::QUATERNION => {
                EntityDataValue::Quaternion(ByteBufCodecs::QUATERNIONF.decode(buffer))
            }
            EntityDataSerializer::HUMANOID_ARM => {
                EntityDataValue::HumanoidArm(HumanoidArm::STREAM_CODEC.decode(buffer))
            }
            unported => {
                let rest = buffer.readable_bytes();
                buffer.skip_bytes(rest);
                EntityDataValue::Unported(unported)
            }
        }
    }

    /// `codec().encode(output, value)`. Panics for a value of another serializer's type, where
    /// Java's cast throws `ClassCastException`, and for one whose type isn't ported.
    pub fn encode(self, output: &mut RegistryFriendlyByteBuf, value: &EntityDataValue) {
        assert_eq!(
            self,
            value.serializer(),
            "ClassCastException: a value of another serializer"
        );
        if let (Some(registry), EntityDataValue::Variant(_, holder)) =
            (self.variant_registry(), value)
        {
            return ByteBufCodecs::holder_registry(registry).encode(output, holder);
        }
        let buffer: &mut ByteBuf = output.as_mut();
        match value {
            EntityDataValue::Byte(value) => ByteBufCodecs::BYTE.encode(buffer, value),
            EntityDataValue::Int(value) => ByteBufCodecs::VAR_INT.encode(buffer, value),
            EntityDataValue::Long(value) => ByteBufCodecs::VAR_LONG.encode(buffer, value),
            EntityDataValue::Float(value) => ByteBufCodecs::FLOAT.encode(buffer, value),
            EntityDataValue::String(value) => ByteBufCodecs::STRING_UTF8.encode(buffer, value),
            EntityDataValue::Boolean(value) => ByteBufCodecs::BOOL.encode(buffer, value),
            EntityDataValue::Rotations(value) => Rotations::STREAM_CODEC.encode(buffer, value),
            EntityDataValue::BlockPos(value) => BlockPos::STREAM_CODEC.encode(buffer, value),
            EntityDataValue::OptionalBlockPos(value) => {
                ByteBufCodecs::optional(BlockPos::STREAM_CODEC).encode(buffer, value)
            }
            EntityDataValue::Direction(value) => Direction::STREAM_CODEC.encode(buffer, value),
            EntityDataValue::BlockState(value) => {
                let id = Block::block_state_registry().get_id_or_throw(value);
                VarInt::write(buffer, id);
            }
            EntityDataValue::OptionalBlockState(value) => {
                VarInt::write(buffer, value.map_or(0, |state| Block::get_id(Some(state))));
            }
            EntityDataValue::OptionalUnsignedInt(value) => {
                VarInt::write(buffer, value.unwrap_or(-1).wrapping_add(1));
            }
            EntityDataValue::Pose(value) => Pose::STREAM_CODEC.encode(buffer, value),
            EntityDataValue::OptionalGlobalPos(value) => {
                ByteBufCodecs::optional(GlobalPos::STREAM_CODEC).encode(buffer, value)
            }
            EntityDataValue::Vector3(value) => ByteBufCodecs::VECTOR3F.encode(buffer, value),
            EntityDataValue::Quaternion(value) => ByteBufCodecs::QUATERNIONF.encode(buffer, value),
            EntityDataValue::HumanoidArm(value) => HumanoidArm::STREAM_CODEC.encode(buffer, value),
            EntityDataValue::Variant(..) => unreachable!("a variant's serializer has a registry"),
            EntityDataValue::Unported(serializer) => {
                panic!("{serializer:?}'s values aren't ported yet")
            }
        }
    }
}
