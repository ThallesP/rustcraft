//! net/minecraft/world/entity/PositionMoveRotation.java
//!
//! Not ported yet: `of(Entity)` and `of(TeleportTransition)`, which come with entities. The
//! listener makes one from the player's fields until then.

use std::sync::LazyLock;

use crate::java::lang::float::Float;
use crate::java::lang::math::Math;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::entity::relative::Relative;
use crate::net::minecraft::world::phys::vec3::Vec3;
use std::fmt;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, PositionMoveRotation> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite4(
            Vec3::STREAM_CODEC,
            PositionMoveRotation::position,
            Vec3::STREAM_CODEC,
            PositionMoveRotation::delta_movement,
            ByteBufCodecs::FLOAT,
            PositionMoveRotation::y_rot,
            ByteBufCodecs::FLOAT,
            PositionMoveRotation::x_rot,
            PositionMoveRotation::new,
        ))
    });

/// Where an entity is, how it moves and where it looks: a teleport's values.
#[derive(Clone, Copy, Debug)]
pub struct PositionMoveRotation {
    position: Vec3,
    delta_movement: Vec3,
    y_rot: f32,
    x_rot: f32,
}

impl PositionMoveRotation {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, PositionMoveRotation> =
        &STREAM_CODEC;

    pub fn new(
        position: Vec3,
        delta_movement: Vec3,
        y_rot: f32,
        x_rot: f32,
    ) -> PositionMoveRotation {
        PositionMoveRotation {
            position,
            delta_movement,
            y_rot,
            x_rot,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn delta_movement(&self) -> Vec3 {
        self.delta_movement
    }

    pub fn y_rot(&self) -> f32 {
        self.y_rot
    }

    pub fn x_rot(&self) -> f32 {
        self.x_rot
    }

    pub fn with_rotation(&self, y_rot: f32, x_rot: f32) -> PositionMoveRotation {
        PositionMoveRotation::new(self.position, self.delta_movement, y_rot, x_rot)
    }

    /// The values `change` makes of `source`: each part `relatives` has is added to the source's,
    /// and the others replace it.
    pub fn calculate_absolute(
        source: &PositionMoveRotation,
        change: &PositionMoveRotation,
        relatives: &[Relative],
    ) -> PositionMoveRotation {
        let offset_x = if relatives.contains(&Relative::X) {
            source.position.x
        } else {
            0.0
        };
        let offset_y = if relatives.contains(&Relative::Y) {
            source.position.y
        } else {
            0.0
        };
        let offset_z = if relatives.contains(&Relative::Z) {
            source.position.z
        } else {
            0.0
        };
        let offset_y_rot = if relatives.contains(&Relative::Y_ROT) {
            source.y_rot
        } else {
            0.0
        };
        let offset_x_rot = if relatives.contains(&Relative::X_ROT) {
            source.x_rot
        } else {
            0.0
        };
        let absolute_position = Vec3::new(
            offset_x + change.position.x,
            offset_y + change.position.y,
            offset_z + change.position.z,
        );
        let absolute_y_rot = offset_y_rot + change.y_rot;
        let absolute_x_rot = Mth::clamp_f32(offset_x_rot + change.x_rot, -90.0, 90.0);
        let mut rotated_current_movement = source.delta_movement;
        if relatives.contains(&Relative::ROTATE_DELTA) {
            let diff_y_rot = source.y_rot - absolute_y_rot;
            let diff_x_rot = source.x_rot - absolute_x_rot;
            rotated_current_movement =
                rotated_current_movement.x_rot(Math::to_radians(f64::from(diff_x_rot)) as f32);
            rotated_current_movement =
                rotated_current_movement.y_rot(Math::to_radians(f64::from(diff_y_rot)) as f32);
        }

        let absolute_delta_movement = Vec3::new(
            PositionMoveRotation::calculate_delta(
                rotated_current_movement.x,
                change.delta_movement.x,
                relatives,
                Relative::DELTA_X,
            ),
            PositionMoveRotation::calculate_delta(
                rotated_current_movement.y,
                change.delta_movement.y,
                relatives,
                Relative::DELTA_Y,
            ),
            PositionMoveRotation::calculate_delta(
                rotated_current_movement.z,
                change.delta_movement.z,
                relatives,
                Relative::DELTA_Z,
            ),
        );
        PositionMoveRotation::new(
            absolute_position,
            absolute_delta_movement,
            absolute_y_rot,
            absolute_x_rot,
        )
    }

    fn calculate_delta(
        current_delta: f64,
        delta_change: f64,
        relatives: &[Relative],
        relative: Relative,
    ) -> f64 {
        if relatives.contains(&relative) {
            current_delta + delta_change
        } else {
            delta_change
        }
    }

    /// A record's: each component's hash added to 31 times the ones before, the floats' bits.
    pub fn hash_code(&self) -> i32 {
        [
            self.delta_movement.hash_code(),
            Float::hash_code(self.y_rot),
            Float::hash_code(self.x_rot),
        ]
        .into_iter()
        .fold(self.position.hash_code(), |hash, component| {
            hash.wrapping_mul(31).wrapping_add(component)
        })
    }
}

/// A record's: `Vec3`'s `equals` for the vectors, and the floats by their bits.
impl PartialEq for PositionMoveRotation {
    fn eq(&self, other: &PositionMoveRotation) -> bool {
        self.position == other.position
            && self.delta_movement == other.delta_movement
            && Float::float_to_int_bits(self.y_rot) == Float::float_to_int_bits(other.y_rot)
            && Float::float_to_int_bits(self.x_rot) == Float::float_to_int_bits(other.x_rot)
    }
}

/// A record's: `PositionMoveRotation[position=(0.5, -60.0, 0.5), deltaMovement=(0.0, 0.0, 0.0), yRot=0.0, xRot=0.0]`.
impl fmt::Display for PositionMoveRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PositionMoveRotation[position={}, deltaMovement={}, yRot={}, xRot={}]",
            self.position,
            self.delta_movement,
            Float::to_string(self.y_rot),
            Float::to_string(self.x_rot)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PositionMoveRotation;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::world::entity::relative::Relative;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    encode_fields!(PositionMoveRotation {
        position: Vec3,
        delta_movement: Vec3,
        y_rot: f32,
        x_rot: f32,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.PositionMoveRotation");
        golden.check_codec("STREAM_CODEC", || PositionMoveRotation::STREAM_CODEC);
        golden.check(
            "<init>(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;FF)V",
            PositionMoveRotation::new,
        );
        golden.check(
            "calculateAbsolute(Lnet/minecraft/world/entity/PositionMoveRotation;Lnet/minecraft/world/entity/PositionMoveRotation;Ljava/util/Set;)Lnet/minecraft/world/entity/PositionMoveRotation;",
            |source: PositionMoveRotation, change: PositionMoveRotation, relatives: Vec<Relative>| {
                PositionMoveRotation::calculate_absolute(&source, &change, &relatives)
            },
        );
        golden.check_method(
            "position()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut PositionMoveRotation| this.position(),
        );
        golden.check_method(
            "deltaMovement()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut PositionMoveRotation| this.delta_movement(),
        );
        golden.check_method("yRot()F", |this: &mut PositionMoveRotation| this.y_rot());
        golden.check_method("xRot()F", |this: &mut PositionMoveRotation| this.x_rot());
        golden.check_method(
            "withRotation(FF)Lnet/minecraft/world/entity/PositionMoveRotation;",
            |this: &mut PositionMoveRotation, y_rot: f32, x_rot: f32| {
                this.with_rotation(y_rot, x_rot)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut PositionMoveRotation, other: PositionMoveRotation| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut PositionMoveRotation| {
            this.hash_code()
        });
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut PositionMoveRotation| this.to_string(),
        );
        golden.pending(
            "of(Lnet/minecraft/world/entity/Entity;)Lnet/minecraft/world/entity/PositionMoveRotation;",
            "comes with entities",
        );
        golden.pending(
            "of(Lnet/minecraft/world/level/portal/TeleportTransition;)Lnet/minecraft/world/entity/PositionMoveRotation;",
            "comes with portals",
        );
        golden.finish();
    }
}
