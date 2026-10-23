//! net/minecraft/network/protocol/game/VecDeltaCodec.java
//!
//! Where an entity is, as the moves the server sends tell it: a base position, which each move's
//! delta is from, in 4096ths of a block.

use crate::java::lang::math::Math;
use crate::net::minecraft::network::protocol::game::vec_delta::{self, VecDelta};
use crate::net::minecraft::world::entity::position_path::PositionPath;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// The position moves are encoded from and decoded against.
#[derive(Clone, Debug, PartialEq)]
pub struct VecDeltaCodec {
    base: Vec3,
}

impl Default for VecDeltaCodec {
    fn default() -> VecDeltaCodec {
        VecDeltaCodec::new()
    }
}

impl VecDeltaCodec {
    const TRUNCATION_STEPS: f64 = 4096.0;

    pub fn new() -> VecDeltaCodec {
        VecDeltaCodec { base: Vec3::ZERO }
    }

    /// `encode(double)`, package-private in Java: a coordinate in 4096ths of a block, rounded.
    pub(crate) fn encode(input: f64) -> i64 {
        Math::round_f64(input * VecDeltaCodec::TRUNCATION_STEPS)
    }

    /// `decode(long)`, package-private in Java.
    pub(crate) fn decode_coordinate(v: i64) -> f64 {
        v as f64 / VecDeltaCodec::TRUNCATION_STEPS
    }

    /// How far a coordinate moves when encoded and decoded.
    pub fn encoding_precision_loss(d: f64) -> f64 {
        VecDeltaCodec::decode_coordinate(VecDeltaCodec::encode(d)) - d
    }

    /// Whether a delta along any axis is past what a short holds.
    pub fn is_delta_too_big(xa: i64, ya: i64, za: i64) -> bool {
        const SHORT: std::ops::RangeInclusive<i64> = -32768..=32767;
        !SHORT.contains(&xa) || !SHORT.contains(&ya) || !SHORT.contains(&za)
    }

    /// The base moved by the delta: an axis without one keeps the base's coordinate exactly, one
    /// with one is on the 4096ths' grid.
    pub fn decode(&self, xa: i64, ya: i64, za: i64) -> Vec3 {
        if xa == 0 && ya == 0 && za == 0 {
            return self.base;
        }

        let along = |base: f64, delta: i64| {
            if delta == 0 {
                base
            } else {
                VecDeltaCodec::decode_coordinate(VecDeltaCodec::encode(base).wrapping_add(delta))
            }
        };
        Vec3::new(
            along(self.base.x, xa),
            along(self.base.y, ya),
            along(self.base.z, za),
        )
    }

    /// The move from the base to the path, or none when a delta is too big for a short.
    pub fn try_encode_path(&mut self, position: &PositionPath) -> Option<VecDelta> {
        match position {
            PositionPath::Linear { end_position } => self.try_encode(end_position),
            PositionPath::Stepped { steps, .. } => {
                vec_delta::try_encode_stepped(self, steps).map(VecDelta::Stepped)
            }
        }
    }

    /// The move from the base to the position, or none when a delta is too big for a short.
    pub fn try_encode(&self, pos: &Vec3) -> Option<VecDelta> {
        let xa = self.encode_x(pos);
        let ya = self.encode_y(pos);
        let za = self.encode_z(pos);
        if VecDeltaCodec::is_delta_too_big(xa, ya, za) {
            None
        } else {
            Some(VecDelta::Linear {
                xa: xa as i16,
                ya: ya as i16,
                za: za as i16,
            })
        }
    }

    pub fn encode_x(&self, pos: &Vec3) -> i64 {
        VecDeltaCodec::encode(pos.x).wrapping_sub(VecDeltaCodec::encode(self.base.x))
    }

    pub fn encode_y(&self, pos: &Vec3) -> i64 {
        VecDeltaCodec::encode(pos.y).wrapping_sub(VecDeltaCodec::encode(self.base.y))
    }

    pub fn encode_z(&self, pos: &Vec3) -> i64 {
        VecDeltaCodec::encode(pos.z).wrapping_sub(VecDeltaCodec::encode(self.base.z))
    }

    pub fn delta(&self, pos: &Vec3) -> Vec3 {
        pos.subtract(&self.base)
    }

    pub fn set_base(&mut self, base: Vec3) {
        self.base = base;
    }

    pub fn get_base(&self) -> Vec3 {
        self.base
    }
}

#[cfg(test)]
mod tests {
    use super::VecDeltaCodec;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::world::entity::position_path::PositionPath;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    // Its one field, read by reflection.
    encode_fields!(VecDeltaCodec { base: Vec3 });

    type This = VecDeltaCodec;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.VecDeltaCodec");
        golden.check("<init>()V", VecDeltaCodec::new);
        golden.check_method(
            "decode(JJJ)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut This, xa: i64, ya: i64, za: i64| this.decode(xa, ya, za),
        );
        golden.check_method(
            "delta(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;",
            |this: &mut This, pos: Vec3| this.delta(&pos),
        );
        golden.check_method(
            "encodeX(Lnet/minecraft/world/phys/Vec3;)J",
            |this: &mut This, pos: Vec3| this.encode_x(&pos),
        );
        golden.check_method(
            "encodeY(Lnet/minecraft/world/phys/Vec3;)J",
            |this: &mut This, pos: Vec3| this.encode_y(&pos),
        );
        golden.check_method(
            "encodeZ(Lnet/minecraft/world/phys/Vec3;)J",
            |this: &mut This, pos: Vec3| this.encode_z(&pos),
        );
        golden.check(
            "encodingPrecisionLoss(D)D",
            VecDeltaCodec::encoding_precision_loss,
        );
        golden.check_method(
            "getBase()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut This| this.get_base(),
        );
        golden.check("isDeltaTooBig(JJJ)Z", VecDeltaCodec::is_delta_too_big);
        golden.check_method(
            "setBase(Lnet/minecraft/world/phys/Vec3;)V",
            |this: &mut This, base: Vec3| this.set_base(base),
        );
        golden.check_method(
            "tryEncode(Lnet/minecraft/world/entity/PositionPath;)Lnet/minecraft/network/protocol/game/VecDelta;",
            |this: &mut This, position: PositionPath| this.try_encode_path(&position),
        );
        golden.check_method(
            "tryEncode(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/network/protocol/game/VecDelta;",
            |this: &mut This, pos: Vec3| this.try_encode(&pos),
        );
        golden.finish();
    }
}
