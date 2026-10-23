//! net/minecraft/network/syncher/SynchedEntityData.java
//!
//! Not ported yet: the entities' data itself (the data items each entity class defines, by
//! `defineId`, and `assignValues`, `packDirty` and `getNonDefaultValues`), which comes with the
//! entities, and `DataValue.create`, which takes an `EntityDataAccessor`. What's here is
//! `DataValue`, what the server sends of an entity's data. Its record's `hashCode` and
//! `toString` hash and print its serializer, a lambda, by identity, so they aren't ported.

use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::network::syncher::entity_data_serializer::{
    EntityDataSerializer, EntityDataValue,
};
use crate::net::minecraft::network::syncher::entity_data_serializers::EntityDataSerializers;

/// `SynchedEntityData.DataValue`: an item of an entity's data, by its id, and its value.
#[derive(Clone, Debug, PartialEq)]
pub struct DataValue {
    id: i32,
    value: EntityDataValue,
}

impl DataValue {
    pub fn new(id: i32, value: EntityDataValue) -> DataValue {
        DataValue { id, value }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn serializer(&self) -> EntityDataSerializer {
        self.value.serializer()
    }

    pub fn value(&self) -> &EntityDataValue {
        &self.value
    }

    /// Its id in a byte, its serializer's id, then its value. Panics where Java throws
    /// `EncoderException`, for a serializer that isn't registered, which every one here is.
    pub fn write(&self, output: &mut RegistryFriendlyByteBuf) {
        let serializer = self.serializer();
        let serializer_id = EntityDataSerializers::get_serialized_id(serializer);
        if serializer_id < 0 {
            panic!("EncoderException: Unknown serializer type {serializer:?}");
        }
        output.write_byte(self.id);
        output.write_var_int(serializer_id);
        serializer.encode(output, &self.value);
    }

    /// The value of the item `id`, which the caller has read: its serializer's id, then its
    /// value. Panics where Java throws `DecoderException`, for an id no serializer has.
    pub fn read(input: &mut RegistryFriendlyByteBuf, id: i32) -> DataValue {
        let r#type = input.read_var_int();
        let Some(serializer) = EntityDataSerializers::get_serializer(r#type) else {
            panic!("DecoderException: Unknown serializer type {type}", type = r#type);
        };
        DataValue::new(id, serializer.decode(input))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::DataValue;
    use crate::golden::{Decode, Encode, Golden, elements};
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::core::global_pos::GlobalPos;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::rotations::Rotations;
    use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
    use crate::net::minecraft::network::syncher::entity_data_serializer::{
        EntityDataSerializer, EntityDataValue,
    };
    use crate::net::minecraft::network::syncher::entity_data_serializers::EntityDataSerializers;
    use crate::net::minecraft::world::entity::humanoid_arm::HumanoidArm;
    use crate::net::minecraft::world::entity::pose::Pose;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::org::joml::quaternionf::Quaternionf;
    use crate::org::joml::vector3f::Vector3f;

    // [id, serializer, value]: the serializer's id, and the value as its type is encoded.
    impl Decode for DataValue {
        fn decode(value: &Value) -> Self {
            let [id, serializer, value] = elements(value);
            let serializer = EntityDataSerializers::get_serializer(i32::decode(serializer))
                .expect("a registered serializer");
            let value = match serializer {
                EntityDataSerializer::BYTE => EntityDataValue::Byte(i8::decode(value)),
                EntityDataSerializer::INT => EntityDataValue::Int(i32::decode(value)),
                EntityDataSerializer::LONG => EntityDataValue::Long(i64::decode(value)),
                EntityDataSerializer::FLOAT => EntityDataValue::Float(f32::decode(value)),
                EntityDataSerializer::STRING => EntityDataValue::String(String::decode(value)),
                EntityDataSerializer::BOOLEAN => EntityDataValue::Boolean(bool::decode(value)),
                EntityDataSerializer::ROTATIONS => {
                    EntityDataValue::Rotations(Rotations::decode(value))
                }
                EntityDataSerializer::BLOCK_POS => {
                    EntityDataValue::BlockPos(BlockPos::decode(value))
                }
                EntityDataSerializer::OPTIONAL_BLOCK_POS => {
                    EntityDataValue::OptionalBlockPos(Option::decode(value))
                }
                EntityDataSerializer::DIRECTION => {
                    EntityDataValue::Direction(Direction::decode(value))
                }
                EntityDataSerializer::BLOCK_STATE => {
                    EntityDataValue::BlockState(BlockState::decode(value))
                }
                EntityDataSerializer::OPTIONAL_BLOCK_STATE => {
                    EntityDataValue::OptionalBlockState(Option::decode(value))
                }
                EntityDataSerializer::OPTIONAL_UNSIGNED_INT => {
                    EntityDataValue::OptionalUnsignedInt(Option::decode(value))
                }
                EntityDataSerializer::POSE => EntityDataValue::Pose(Pose::decode(value)),
                EntityDataSerializer::OPTIONAL_GLOBAL_POS => {
                    EntityDataValue::OptionalGlobalPos(Option::<GlobalPos>::decode(value))
                }
                EntityDataSerializer::VECTOR3 => EntityDataValue::Vector3(Vector3f::decode(value)),
                EntityDataSerializer::QUATERNION => {
                    EntityDataValue::Quaternion(Quaternionf::decode(value))
                }
                EntityDataSerializer::HUMANOID_ARM => {
                    EntityDataValue::HumanoidArm(HumanoidArm::decode(value))
                }
                variant => EntityDataValue::Variant(variant, Holder::decode(value)),
            };
            DataValue::new(i32::decode(id), value)
        }
    }

    impl Encode for EntityDataValue {
        fn encode(&self) -> Value {
            match self {
                EntityDataValue::Byte(value) => value.encode(),
                EntityDataValue::Int(value) => value.encode(),
                EntityDataValue::Long(value) => value.encode(),
                EntityDataValue::Float(value) => value.encode(),
                EntityDataValue::String(value) => value.encode(),
                EntityDataValue::Boolean(value) => value.encode(),
                EntityDataValue::Rotations(value) => value.encode(),
                EntityDataValue::BlockPos(value) => value.encode(),
                EntityDataValue::OptionalBlockPos(value) => value.encode(),
                EntityDataValue::Direction(value) => value.encode(),
                EntityDataValue::BlockState(value) => value.encode(),
                EntityDataValue::OptionalBlockState(value) => value.encode(),
                EntityDataValue::OptionalUnsignedInt(value) => value.encode(),
                EntityDataValue::Pose(value) => value.encode(),
                EntityDataValue::Variant(_, value) => value.encode(),
                EntityDataValue::OptionalGlobalPos(value) => value.encode(),
                EntityDataValue::Vector3(value) => value.encode(),
                EntityDataValue::Quaternion(value) => value.encode(),
                EntityDataValue::HumanoidArm(value) => value.encode(),
                EntityDataValue::Unported(serializer) => {
                    Value::from(format!("unported {serializer:?}"))
                }
            }
        }

        fn matches(&self, expected: &Value) -> bool {
            match self {
                EntityDataValue::Float(value) => value.matches(expected),
                EntityDataValue::Rotations(value) => value.matches(expected),
                EntityDataValue::Vector3(value) => value.matches(expected),
                EntityDataValue::Quaternion(value) => value.matches(expected),
                _ => self.encode() == *expected,
            }
        }
    }

    impl Encode for DataValue {
        fn encode(&self) -> Value {
            Value::from(vec![
                self.id.encode(),
                EntityDataSerializers::get_serialized_id(self.serializer()).encode(),
                self.value.encode(),
            ])
        }

        fn matches(&self, expected: &Value) -> bool {
            let Some([id, serializer, value]) = expected.as_array().map(Vec::as_slice) else {
                return false;
            };
            self.id.matches(id)
                && EntityDataSerializers::get_serialized_id(self.serializer()).matches(serializer)
                && self.value.matches(value)
        }
    }

    type This = DataValue;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.syncher.SynchedEntityData$DataValue");
        golden.pending(
            "<init>(ILnet/minecraft/network/syncher/EntityDataSerializer;Ljava/lang/Object;)V",
            "the harness can't generate a serializer, a lambda",
        );
        golden.pending(
            "create(Lnet/minecraft/network/syncher/EntityDataAccessor;Ljava/lang/Object;)Lnet/minecraft/network/syncher/SynchedEntityData$DataValue;",
            "takes an EntityDataAccessor, which comes with the entities' data",
        );
        golden.pending("hashCode()I", "hashes the serializer's identity hash");
        golden.pending(
            "toString()Ljava/lang/String;",
            "prints the serializer's identity",
        );
        golden.pending(
            "serializer()Lnet/minecraft/network/syncher/EntityDataSerializer;",
            "a lambda, which the harness can't encode; each DataValue case records its id",
        );
        golden.pending(
            "value()Ljava/lang/Object;",
            "an Object, which the harness can't encode; each DataValue case records it",
        );
        golden.check_method("id()I", |this: &mut This| this.id());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_mut(
            "write(Lnet/minecraft/network/RegistryFriendlyByteBuf;)V",
            |this: &mut This, output: &mut RegistryFriendlyByteBuf| this.write(output),
        );
        golden.check_mut(
            "read(Lnet/minecraft/network/RegistryFriendlyByteBuf;I)Lnet/minecraft/network/syncher/SynchedEntityData$DataValue;",
            |input: &mut RegistryFriendlyByteBuf, id: &mut i32| DataValue::read(input, *id),
        );
        golden.finish();
    }
}
