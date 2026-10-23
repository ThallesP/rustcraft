//! net/minecraft/network/syncher/EntityDataSerializers.java
//!
//! Java registers its serializers in a `CrudeIncrementalIntIdentityHashBiMap`, which numbers them
//! in the order they're registered, from 0; here that's `EntityDataSerializer`'s order.
//! `registerSerializer`, which a mod would call, isn't ported.

use crate::net::minecraft::network::syncher::entity_data_serializer::EntityDataSerializer;

// In the order Java registers them, which is their ids.
const SERIALIZERS: [EntityDataSerializer; 44] = [
    EntityDataSerializer::BYTE,
    EntityDataSerializer::INT,
    EntityDataSerializer::LONG,
    EntityDataSerializer::FLOAT,
    EntityDataSerializer::STRING,
    EntityDataSerializer::COMPONENT,
    EntityDataSerializer::OPTIONAL_COMPONENT,
    EntityDataSerializer::ITEM_STACK,
    EntityDataSerializer::BOOLEAN,
    EntityDataSerializer::ROTATIONS,
    EntityDataSerializer::BLOCK_POS,
    EntityDataSerializer::OPTIONAL_BLOCK_POS,
    EntityDataSerializer::DIRECTION,
    EntityDataSerializer::OPTIONAL_LIVING_ENTITY_REFERENCE,
    EntityDataSerializer::BLOCK_STATE,
    EntityDataSerializer::OPTIONAL_BLOCK_STATE,
    EntityDataSerializer::PARTICLE,
    EntityDataSerializer::PARTICLES,
    EntityDataSerializer::VILLAGER_DATA,
    EntityDataSerializer::OPTIONAL_UNSIGNED_INT,
    EntityDataSerializer::POSE,
    EntityDataSerializer::CAT_VARIANT,
    EntityDataSerializer::CAT_SOUND_VARIANT,
    EntityDataSerializer::COW_VARIANT,
    EntityDataSerializer::COW_SOUND_VARIANT,
    EntityDataSerializer::WOLF_VARIANT,
    EntityDataSerializer::WOLF_SOUND_VARIANT,
    EntityDataSerializer::FROG_VARIANT,
    EntityDataSerializer::PIG_VARIANT,
    EntityDataSerializer::PIG_SOUND_VARIANT,
    EntityDataSerializer::CHICKEN_VARIANT,
    EntityDataSerializer::CHICKEN_SOUND_VARIANT,
    EntityDataSerializer::ZOMBIE_NAUTILUS_VARIANT,
    EntityDataSerializer::OPTIONAL_GLOBAL_POS,
    EntityDataSerializer::PAINTING_VARIANT,
    EntityDataSerializer::SNIFFER_STATE,
    EntityDataSerializer::ARMADILLO_STATE,
    EntityDataSerializer::COPPER_GOLEM_STATE,
    EntityDataSerializer::WEATHERING_COPPER_STATE,
    EntityDataSerializer::VECTOR3,
    EntityDataSerializer::QUATERNION,
    EntityDataSerializer::RESOLVABLE_PROFILE,
    EntityDataSerializer::HUMANOID_ARM,
    EntityDataSerializer::DYE_COLOR,
];

/// The serializers of the values entities sync, by id.
pub struct EntityDataSerializers;

impl EntityDataSerializers {
    /// The serializer with the id, or none.
    pub fn get_serializer(id: i32) -> Option<EntityDataSerializer> {
        usize::try_from(id)
            .ok()
            .and_then(|index| SERIALIZERS.get(index))
            .copied()
    }

    pub fn get_serialized_id(serializer: EntityDataSerializer) -> i32 {
        SERIALIZERS
            .iter()
            .position(|registered| *registered == serializer)
            .map_or(-1, |id| id as i32)
    }
}
