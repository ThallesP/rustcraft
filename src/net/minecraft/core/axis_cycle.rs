//! net/minecraft/core/AxisCycle.java
//!
//! In Java each constant is a subclass that overrides the abstract methods; here each method
//! matches on the constant.

use crate::java::lang::math::Math;
use crate::net::minecraft::core::direction::Axis;

/// A turn of the axes: none, x to y to z to x (`FORWARD`), or back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxisCycle {
    NONE,
    FORWARD,
    BACKWARD,
}

impl AxisCycle {
    pub const AXIS_VALUES: [Axis; 3] = Axis::VALUES;
    /// `values()`: the constants in declaration order, which is also their ordinal.
    pub const VALUES: [AxisCycle; 3] = [AxisCycle::NONE, AxisCycle::FORWARD, AxisCycle::BACKWARD];

    pub fn values() -> [AxisCycle; 3] {
        AxisCycle::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> AxisCycle {
        match name {
            "NONE" => AxisCycle::NONE,
            "FORWARD" => AxisCycle::FORWARD,
            "BACKWARD" => AxisCycle::BACKWARD,
            _ => panic!("IllegalArgumentException: no AxisCycle constant {name}"),
        }
    }

    pub fn cycle_i32(&self, x: i32, y: i32, z: i32, axis: Axis) -> i32 {
        match self {
            AxisCycle::NONE => axis.choose_i32(x, y, z),
            AxisCycle::FORWARD => axis.choose_i32(z, x, y),
            AxisCycle::BACKWARD => axis.choose_i32(y, z, x),
        }
    }

    pub fn cycle_f64(&self, x: f64, y: f64, z: f64, axis: Axis) -> f64 {
        match self {
            AxisCycle::NONE => axis.choose_f64(x, y, z),
            AxisCycle::FORWARD => axis.choose_f64(z, x, y),
            AxisCycle::BACKWARD => axis.choose_f64(y, z, x),
        }
    }

    pub fn cycle_axis(&self, axis: Axis) -> Axis {
        match self {
            AxisCycle::NONE => axis,
            AxisCycle::FORWARD => {
                AxisCycle::AXIS_VALUES[Math::floor_mod_i32(axis as i32 + 1, 3) as usize]
            }
            AxisCycle::BACKWARD => {
                AxisCycle::AXIS_VALUES[Math::floor_mod_i32(axis as i32 - 1, 3) as usize]
            }
        }
    }

    pub fn inverse(&self) -> AxisCycle {
        match self {
            AxisCycle::NONE => AxisCycle::NONE,
            AxisCycle::FORWARD => AxisCycle::BACKWARD,
            AxisCycle::BACKWARD => AxisCycle::FORWARD,
        }
    }

    pub fn between(from: Axis, to: Axis) -> AxisCycle {
        AxisCycle::VALUES[Math::floor_mod_i32(to as i32 - from as i32, 3) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::AxisCycle;
    use crate::golden::{Golden, encode_names};
    use crate::net::minecraft::core::direction::Axis;

    encode_names!(AxisCycle {
        NONE,
        FORWARD,
        BACKWARD
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.AxisCycle");
        for cycle in AxisCycle::VALUES {
            golden.field(&format!("{cycle:?}"), cycle);
        }
        golden.field("AXIS_VALUES", AxisCycle::AXIS_VALUES);
        golden.field("VALUES", AxisCycle::VALUES);
        golden.check("values()[Lnet/minecraft/core/AxisCycle;", AxisCycle::values);
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/core/AxisCycle;",
            |name: String| AxisCycle::value_of(&name),
        );
        golden.check_method(
            "cycle(IIILnet/minecraft/core/Direction$Axis;)I",
            |this: &mut AxisCycle, x, y, z, axis: Axis| this.cycle_i32(x, y, z, axis),
        );
        golden.check_method(
            "cycle(DDDLnet/minecraft/core/Direction$Axis;)D",
            |this: &mut AxisCycle, x, y, z, axis: Axis| this.cycle_f64(x, y, z, axis),
        );
        golden.check_method(
            "cycle(Lnet/minecraft/core/Direction$Axis;)Lnet/minecraft/core/Direction$Axis;",
            |this: &mut AxisCycle, axis: Axis| this.cycle_axis(axis),
        );
        golden.check_method(
            "inverse()Lnet/minecraft/core/AxisCycle;",
            |this: &mut AxisCycle| this.inverse(),
        );
        golden.check(
            "between(Lnet/minecraft/core/Direction$Axis;Lnet/minecraft/core/Direction$Axis;)Lnet/minecraft/core/AxisCycle;",
            AxisCycle::between,
        );
        golden.finish();
    }
}
