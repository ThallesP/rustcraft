//! com/mojang/math/SymmetricGroup3.java
//!
//! The constants' permutations and matrices are a table by ordinal, one line per constant like
//! Java's, and the Cayley and inverse tables are worked out on first use like its static
//! initializer.

use std::sync::LazyLock;

use crate::net::minecraft::core::direction::Axis;
use crate::org::joml::matrix3f::Matrix3f;
use crate::org::joml::vector3f::Vector3f;
use crate::org::joml::vector3i::Vector3i;

/// The six permutations of three axes: `P231` sends component 0 to where component 1 was read
/// from, and so on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SymmetricGroup3 {
    P123,
    P213,
    P132,
    P312,
    P231,
    P321,
}

/// `[p0, p1, p2]` of each constant.
const PERMUTATIONS: [[i32; 3]; 6] = [
    [0, 1, 2],
    [1, 0, 2],
    [0, 2, 1],
    [2, 0, 1],
    [1, 2, 0],
    [2, 1, 0],
];

static TRANSFORMATIONS: LazyLock<[Matrix3f; 6]> = LazyLock::new(|| {
    SymmetricGroup3::VALUES.map(|permutation| {
        *Matrix3f::new()
            .zero()
            .set_element(permutation.permute(0), 0, 1.0)
            .set_element(permutation.permute(1), 1, 1.0)
            .set_element(permutation.permute(2), 2, 1.0)
    })
});

static CAYLEY_TABLE: LazyLock<[[SymmetricGroup3; 6]; 6]> = LazyLock::new(|| {
    SymmetricGroup3::VALUES.map(|first| {
        SymmetricGroup3::VALUES.map(|second| {
            let p0 = first.permute(second.p0());
            let p1 = first.permute(second.p1());
            let p2 = first.permute(second.p2());
            SymmetricGroup3::VALUES
                .into_iter()
                .find(|p| p.p0() == p0 && p.p1() == p1 && p.p2() == p2)
                .expect("permutations compose to a permutation")
        })
    })
});

static INVERSE_TABLE: LazyLock<[SymmetricGroup3; 6]> = LazyLock::new(|| {
    SymmetricGroup3::VALUES.map(|f| {
        SymmetricGroup3::VALUES
            .into_iter()
            .find(|&s| f.compose(s) == SymmetricGroup3::P123)
            .expect("every permutation has an inverse")
    })
});

impl SymmetricGroup3 {
    pub const VALUES: [SymmetricGroup3; 6] = [
        SymmetricGroup3::P123,
        SymmetricGroup3::P213,
        SymmetricGroup3::P132,
        SymmetricGroup3::P312,
        SymmetricGroup3::P231,
        SymmetricGroup3::P321,
    ];

    pub fn values() -> [SymmetricGroup3; 6] {
        SymmetricGroup3::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> SymmetricGroup3 {
        SymmetricGroup3::VALUES
            .into_iter()
            .find(|value| format!("{value:?}") == name)
            .unwrap_or_else(|| {
                panic!("IllegalArgumentException: no SymmetricGroup3 constant {name}")
            })
    }

    fn p0(&self) -> i32 {
        PERMUTATIONS[*self as usize][0]
    }

    fn p1(&self) -> i32 {
        PERMUTATIONS[*self as usize][1]
    }

    fn p2(&self) -> i32 {
        PERMUTATIONS[*self as usize][2]
    }

    pub fn compose(&self, that: SymmetricGroup3) -> SymmetricGroup3 {
        CAYLEY_TABLE[*self as usize][that as usize]
    }

    pub fn inverse(&self) -> SymmetricGroup3 {
        INVERSE_TABLE[*self as usize]
    }

    /// Java throws `IllegalArgumentException` for anything but 0, 1 or 2.
    pub fn permute(&self, i: i32) -> i32 {
        match i {
            0 => self.p0(),
            1 => self.p1(),
            2 => self.p2(),
            _ => panic!("IllegalArgumentException: Must be 0, 1 or 2, but got {i}"),
        }
    }

    pub fn permute_axis(&self, axis: Axis) -> Axis {
        Axis::VALUES[self.permute(axis as i32) as usize]
    }

    pub fn permute_vector3f<'a>(&self, v: &'a mut Vector3f) -> &'a mut Vector3f {
        let v0 = v.get(self.p0());
        let v1 = v.get(self.p1());
        let v2 = v.get(self.p2());
        v.set_xyz(v0, v1, v2)
    }

    pub fn permute_vector3i<'a>(&self, v: &'a mut Vector3i) -> &'a mut Vector3i {
        let v0 = v.get(self.p0());
        let v1 = v.get(self.p1());
        let v2 = v.get(self.p2());
        v.set_xyz(v0, v1, v2)
    }

    pub fn transformation(&self) -> &'static Matrix3f {
        &TRANSFORMATIONS[*self as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::SymmetricGroup3;
    use crate::golden::{Golden, encode_names};
    use crate::net::minecraft::core::direction::Axis;
    use crate::org::joml::vector3f::Vector3f;
    use crate::org::joml::vector3i::Vector3i;

    encode_names!(SymmetricGroup3 {
        P123,
        P213,
        P132,
        P312,
        P231,
        P321
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.math.SymmetricGroup3");
        for value in SymmetricGroup3::VALUES {
            golden.field(&format!("{value:?}"), value);
        }
        golden.check(
            "values()[Lcom/mojang/math/SymmetricGroup3;",
            SymmetricGroup3::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lcom/mojang/math/SymmetricGroup3;",
            |name: String| SymmetricGroup3::value_of(&name),
        );
        golden.check_method(
            "compose(Lcom/mojang/math/SymmetricGroup3;)Lcom/mojang/math/SymmetricGroup3;",
            |this: &mut SymmetricGroup3, that| this.compose(that),
        );
        golden.check_method(
            "inverse()Lcom/mojang/math/SymmetricGroup3;",
            |this: &mut SymmetricGroup3| this.inverse(),
        );
        golden.check_method("permute(I)I", |this: &mut SymmetricGroup3, i| {
            this.permute(i)
        });
        golden.check_method(
            "permuteAxis(Lnet/minecraft/core/Direction$Axis;)Lnet/minecraft/core/Direction$Axis;",
            |this: &mut SymmetricGroup3, axis: Axis| this.permute_axis(axis),
        );
        golden.check_mut(
            "permuteVector(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut SymmetricGroup3, v: &mut Vector3f| *this.permute_vector3f(v),
        );
        golden.check_mut(
            "permuteVector(Lorg/joml/Vector3i;)Lorg/joml/Vector3i;",
            |this: &mut SymmetricGroup3, v: &mut Vector3i| *this.permute_vector3i(v),
        );
        golden.check_method(
            "transformation()Lorg/joml/Matrix3fc;",
            |this: &mut SymmetricGroup3| *this.transformation(),
        );
        golden.finish();
    }
}
