//! net/minecraft/world/level/block/entity/BrushableBlockEntity.java
//!
//! Suspicious sand's and gravel's brushing: the face it was brushed from, as `loadAdditional`
//! reads it (`Direction.LEGACY_ID_CODEC`, a byte's 3D data value). Not ported yet: the item buried
//! in it (`item`, an item stack, which the renderer shows) and its loot table, and brushing.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{BYTE, CodecExt, StaticCodec};
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

// Direction.LEGACY_ID_CODEC.
pub(crate) static LEGACY_ID_CODEC: StaticCodec<Direction> = LazyLock::new(|| {
    BYTE.xmap(
        |value: i8| Direction::from_3d_data_value(i32::from(value)),
        |direction: &Direction| direction.get_3d_data_value() as i8,
    )
    .boxed()
});

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BrushableBlockEntity {
    hit_direction: Option<Direction>,
}

impl BrushableBlockEntity {
    pub fn new() -> BrushableBlockEntity {
        BrushableBlockEntity::default()
    }

    pub fn load_additional(&mut self, input: &TagValueInput<'_>) {
        self.hit_direction = input.read("hit_direction", &LEGACY_ID_CODEC);
    }

    pub fn get_hit_direction(&self) -> Option<Direction> {
        self.hit_direction
    }
}
