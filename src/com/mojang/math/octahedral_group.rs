//! com/mojang/math/OctahedralGroup.java
//!
//! Each constant's name, permutation and inversions are a table by ordinal, one line per constant
//! like Java's, and its matrix and the Cayley and inverse tables are worked out on first use like
//! the static initializer. Java caches `rotate(Direction)`'s answers in a map per constant on
//! first use; working them out each time gives the same.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::math::symmetric_group3::SymmetricGroup3;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::front_and_top::FrontAndTop;
use crate::org::joml::matrix3f::Matrix3f;
use crate::org::joml::vector3i::Vector3i;

/// The 48 symmetries of a cube: the rotations, and the rotations after an inversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OctahedralGroup {
    IDENTITY,
    ROT_180_FACE_XY,
    ROT_180_FACE_XZ,
    ROT_180_FACE_YZ,
    ROT_120_NNN,
    ROT_120_NNP,
    ROT_120_NPN,
    ROT_120_NPP,
    ROT_120_PNN,
    ROT_120_PNP,
    ROT_120_PPN,
    ROT_120_PPP,
    ROT_180_EDGE_XY_NEG,
    ROT_180_EDGE_XY_POS,
    ROT_180_EDGE_XZ_NEG,
    ROT_180_EDGE_XZ_POS,
    ROT_180_EDGE_YZ_NEG,
    ROT_180_EDGE_YZ_POS,
    ROT_90_X_NEG,
    ROT_90_X_POS,
    ROT_90_Y_NEG,
    ROT_90_Y_POS,
    ROT_90_Z_NEG,
    ROT_90_Z_POS,
    INVERSION,
    INVERT_X,
    INVERT_Y,
    INVERT_Z,
    ROT_60_REF_NNN,
    ROT_60_REF_NNP,
    ROT_60_REF_NPN,
    ROT_60_REF_NPP,
    ROT_60_REF_PNN,
    ROT_60_REF_PNP,
    ROT_60_REF_PPN,
    ROT_60_REF_PPP,
    SWAP_XY,
    SWAP_YZ,
    SWAP_XZ,
    SWAP_NEG_XY,
    SWAP_NEG_YZ,
    SWAP_NEG_XZ,
    ROT_90_REF_X_NEG,
    ROT_90_REF_X_POS,
    ROT_90_REF_Y_NEG,
    ROT_90_REF_Y_POS,
    ROT_90_REF_Z_NEG,
    ROT_90_REF_Z_POS,
}

struct Data {
    name: &'static str,
    permutation: SymmetricGroup3,
    invert_x: bool,
    invert_y: bool,
    invert_z: bool,
}

const fn data(
    name: &'static str,
    permutation: SymmetricGroup3,
    invert_x: bool,
    invert_y: bool,
    invert_z: bool,
) -> Data {
    Data {
        name,
        permutation,
        invert_x,
        invert_y,
        invert_z,
    }
}

use SymmetricGroup3::{P123, P132, P213, P231, P312, P321};

const DATA: [Data; 48] = [
    data("identity", P123, false, false, false),
    data("rot_180_face_xy", P123, true, true, false),
    data("rot_180_face_xz", P123, true, false, true),
    data("rot_180_face_yz", P123, false, true, true),
    data("rot_120_nnn", P231, false, false, false),
    data("rot_120_nnp", P312, true, false, true),
    data("rot_120_npn", P312, false, true, true),
    data("rot_120_npp", P231, true, false, true),
    data("rot_120_pnn", P312, true, true, false),
    data("rot_120_pnp", P231, true, true, false),
    data("rot_120_ppn", P231, false, true, true),
    data("rot_120_ppp", P312, false, false, false),
    data("rot_180_edge_xy_neg", P213, true, true, true),
    data("rot_180_edge_xy_pos", P213, false, false, true),
    data("rot_180_edge_xz_neg", P321, true, true, true),
    data("rot_180_edge_xz_pos", P321, false, true, false),
    data("rot_180_edge_yz_neg", P132, true, true, true),
    data("rot_180_edge_yz_pos", P132, true, false, false),
    data("rot_90_x_neg", P132, false, false, true),
    data("rot_90_x_pos", P132, false, true, false),
    data("rot_90_y_neg", P321, true, false, false),
    data("rot_90_y_pos", P321, false, false, true),
    data("rot_90_z_neg", P213, false, true, false),
    data("rot_90_z_pos", P213, true, false, false),
    data("inversion", P123, true, true, true),
    data("invert_x", P123, true, false, false),
    data("invert_y", P123, false, true, false),
    data("invert_z", P123, false, false, true),
    data("rot_60_ref_nnn", P312, true, true, true),
    data("rot_60_ref_nnp", P231, true, false, false),
    data("rot_60_ref_npn", P231, false, false, true),
    data("rot_60_ref_npp", P312, false, false, true),
    data("rot_60_ref_pnn", P231, false, true, false),
    data("rot_60_ref_pnp", P312, true, false, false),
    data("rot_60_ref_ppn", P312, false, true, false),
    data("rot_60_ref_ppp", P231, true, true, true),
    data("swap_xy", P213, false, false, false),
    data("swap_yz", P132, false, false, false),
    data("swap_xz", P321, false, false, false),
    data("swap_neg_xy", P213, true, true, false),
    data("swap_neg_yz", P132, false, true, true),
    data("swap_neg_xz", P321, true, false, true),
    data("rot_90_ref_x_neg", P132, true, false, true),
    data("rot_90_ref_x_pos", P132, true, true, false),
    data("rot_90_ref_y_neg", P321, true, true, false),
    data("rot_90_ref_y_pos", P321, false, true, true),
    data("rot_90_ref_z_neg", P213, false, true, true),
    data("rot_90_ref_z_pos", P213, true, false, true),
];

// The constructor's `new Matrix3f().scaling(…).mul(permutation.transformation())`.
static TRANSFORMATIONS: LazyLock<[Matrix3f; 48]> = LazyLock::new(|| {
    OctahedralGroup::VALUES.map(|group| {
        let data = group.data();
        let sign = |invert: bool| if invert { -1.0 } else { 1.0 };
        *Matrix3f::new()
            .scaling(
                sign(data.invert_x),
                sign(data.invert_y),
                sign(data.invert_z),
            )
            .mul(data.permutation.transformation())
    })
});

static CAYLEY_TABLE: LazyLock<[[OctahedralGroup; 48]; 48]> = LazyLock::new(|| {
    // `fingerprints`: each constant by its trace, which tells them apart.
    let by_trace = |trace: i32| {
        OctahedralGroup::VALUES
            .into_iter()
            .find(|group| group.trace() == trace)
            .expect("every trace is a constant's")
    };
    OctahedralGroup::VALUES.map(|first| {
        OctahedralGroup::VALUES.map(|second| {
            let composed_permutation = second.permutation().compose(first.permutation());
            let composed_invert = |axis: Axis| {
                first.inverts(axis) ^ second.inverts(first.permutation().permute_axis(axis))
            };
            by_trace(OctahedralGroup::trace_of(
                composed_invert(Axis::X),
                composed_invert(Axis::Y),
                composed_invert(Axis::Z),
                composed_permutation,
            ))
        })
    })
});

static INVERSE_TABLE: LazyLock<[OctahedralGroup; 48]> = LazyLock::new(|| {
    OctahedralGroup::VALUES.map(|f| {
        OctahedralGroup::VALUES
            .into_iter()
            .find(|&s| f.compose(s) == OctahedralGroup::IDENTITY)
            .expect("every symmetry has an inverse")
    })
});

impl OctahedralGroup {
    pub const BLOCK_ROT_X_270: OctahedralGroup = OctahedralGroup::ROT_90_X_POS;
    pub const BLOCK_ROT_X_180: OctahedralGroup = OctahedralGroup::ROT_180_FACE_YZ;
    pub const BLOCK_ROT_X_90: OctahedralGroup = OctahedralGroup::ROT_90_X_NEG;
    pub const BLOCK_ROT_Y_270: OctahedralGroup = OctahedralGroup::ROT_90_Y_POS;
    pub const BLOCK_ROT_Y_180: OctahedralGroup = OctahedralGroup::ROT_180_FACE_XZ;
    pub const BLOCK_ROT_Y_90: OctahedralGroup = OctahedralGroup::ROT_90_Y_NEG;
    pub const BLOCK_ROT_Z_270: OctahedralGroup = OctahedralGroup::ROT_90_Z_POS;
    pub const BLOCK_ROT_Z_180: OctahedralGroup = OctahedralGroup::ROT_180_FACE_XY;
    pub const BLOCK_ROT_Z_90: OctahedralGroup = OctahedralGroup::ROT_90_Z_NEG;

    pub const VALUES: [OctahedralGroup; 48] = {
        use OctahedralGroup::*;
        [
            IDENTITY,
            ROT_180_FACE_XY,
            ROT_180_FACE_XZ,
            ROT_180_FACE_YZ,
            ROT_120_NNN,
            ROT_120_NNP,
            ROT_120_NPN,
            ROT_120_NPP,
            ROT_120_PNN,
            ROT_120_PNP,
            ROT_120_PPN,
            ROT_120_PPP,
            ROT_180_EDGE_XY_NEG,
            ROT_180_EDGE_XY_POS,
            ROT_180_EDGE_XZ_NEG,
            ROT_180_EDGE_XZ_POS,
            ROT_180_EDGE_YZ_NEG,
            ROT_180_EDGE_YZ_POS,
            ROT_90_X_NEG,
            ROT_90_X_POS,
            ROT_90_Y_NEG,
            ROT_90_Y_POS,
            ROT_90_Z_NEG,
            ROT_90_Z_POS,
            INVERSION,
            INVERT_X,
            INVERT_Y,
            INVERT_Z,
            ROT_60_REF_NNN,
            ROT_60_REF_NNP,
            ROT_60_REF_NPN,
            ROT_60_REF_NPP,
            ROT_60_REF_PNN,
            ROT_60_REF_PNP,
            ROT_60_REF_PPN,
            ROT_60_REF_PPP,
            SWAP_XY,
            SWAP_YZ,
            SWAP_XZ,
            SWAP_NEG_XY,
            SWAP_NEG_YZ,
            SWAP_NEG_XZ,
            ROT_90_REF_X_NEG,
            ROT_90_REF_X_POS,
            ROT_90_REF_Y_NEG,
            ROT_90_REF_Y_POS,
            ROT_90_REF_Z_NEG,
            ROT_90_REF_Z_POS,
        ]
    };

    pub fn values() -> [OctahedralGroup; 48] {
        OctahedralGroup::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> OctahedralGroup {
        OctahedralGroup::VALUES
            .into_iter()
            .find(|value| format!("{value:?}") == name)
            .unwrap_or_else(|| {
                panic!("IllegalArgumentException: no OctahedralGroup constant {name}")
            })
    }

    fn data(&self) -> &'static Data {
        &DATA[*self as usize]
    }

    fn trace_of(
        invert_x: bool,
        invert_y: bool,
        invert_z: bool,
        permutation: SymmetricGroup3,
    ) -> i32 {
        let inversion_index = (if invert_z { 4 } else { 0 })
            + (if invert_y { 2 } else { 0 })
            + (if invert_x { 1 } else { 0 });
        (permutation as i32) << 3 | inversion_index
    }

    fn trace(&self) -> i32 {
        let data = self.data();
        OctahedralGroup::trace_of(
            data.invert_x,
            data.invert_y,
            data.invert_z,
            data.permutation,
        )
    }

    pub fn compose(&self, that: OctahedralGroup) -> OctahedralGroup {
        CAYLEY_TABLE[*self as usize][that as usize]
    }

    pub fn inverse(&self) -> OctahedralGroup {
        INVERSE_TABLE[*self as usize]
    }

    pub fn transformation(&self) -> &'static Matrix3f {
        &TRANSFORMATIONS[*self as usize]
    }

    pub fn get_serialized_name(&self) -> &'static str {
        self.data().name
    }

    pub fn rotate_direction(&self, direction: Direction) -> Direction {
        let old_axis = direction.get_axis();
        let old_direction = direction.get_axis_direction();
        let new_axis = self.permutation().inverse().permute_axis(old_axis);
        let new_direction = if self.inverts(new_axis) {
            old_direction.opposite()
        } else {
            old_direction
        };
        Direction::from_axis_and_direction(new_axis, new_direction)
    }

    pub fn rotate_vector3i<'a>(&self, v: &'a mut Vector3i) -> &'a mut Vector3i {
        let data = self.data();
        self.permutation().permute_vector3i(v);
        v.x = v.x.wrapping_mul(if data.invert_x { -1 } else { 1 });
        v.y = v.y.wrapping_mul(if data.invert_y { -1 } else { 1 });
        v.z = v.z.wrapping_mul(if data.invert_z { -1 } else { 1 });
        v
    }

    pub fn inverts(&self, axis: Axis) -> bool {
        let data = self.data();
        match axis {
            Axis::X => data.invert_x,
            Axis::Y => data.invert_y,
            Axis::Z => data.invert_z,
        }
    }

    pub fn permutation(&self) -> SymmetricGroup3 {
        self.data().permutation
    }

    /// Java's `fromFrontAndTop` hands back null for a pair no constant has, which `rotate`
    /// passes on.
    pub fn rotate_front_and_top(&self, input: FrontAndTop) -> Option<FrontAndTop> {
        FrontAndTop::from_front_and_top(
            self.rotate_direction(input.front()),
            self.rotate_direction(input.top()),
        )
    }
}

/// Its serialized name, like Java's `toString`.
impl fmt::Display for OctahedralGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.data().name)
    }
}

#[cfg(test)]
mod tests {
    use super::OctahedralGroup;
    use crate::com::mojang::math::symmetric_group3::SymmetricGroup3;
    use crate::golden::{Golden, encode_names};
    use crate::net::minecraft::core::direction::{Axis, Direction};
    use crate::net::minecraft::core::front_and_top::FrontAndTop;
    use crate::org::joml::vector3i::Vector3i;

    encode_names!(OctahedralGroup {
        IDENTITY,
        ROT_180_FACE_XY,
        ROT_180_FACE_XZ,
        ROT_180_FACE_YZ,
        ROT_120_NNN,
        ROT_120_NNP,
        ROT_120_NPN,
        ROT_120_NPP,
        ROT_120_PNN,
        ROT_120_PNP,
        ROT_120_PPN,
        ROT_120_PPP,
        ROT_180_EDGE_XY_NEG,
        ROT_180_EDGE_XY_POS,
        ROT_180_EDGE_XZ_NEG,
        ROT_180_EDGE_XZ_POS,
        ROT_180_EDGE_YZ_NEG,
        ROT_180_EDGE_YZ_POS,
        ROT_90_X_NEG,
        ROT_90_X_POS,
        ROT_90_Y_NEG,
        ROT_90_Y_POS,
        ROT_90_Z_NEG,
        ROT_90_Z_POS,
        INVERSION,
        INVERT_X,
        INVERT_Y,
        INVERT_Z,
        ROT_60_REF_NNN,
        ROT_60_REF_NNP,
        ROT_60_REF_NPN,
        ROT_60_REF_NPP,
        ROT_60_REF_PNN,
        ROT_60_REF_PNP,
        ROT_60_REF_PPN,
        ROT_60_REF_PPP,
        SWAP_XY,
        SWAP_YZ,
        SWAP_XZ,
        SWAP_NEG_XY,
        SWAP_NEG_YZ,
        SWAP_NEG_XZ,
        ROT_90_REF_X_NEG,
        ROT_90_REF_X_POS,
        ROT_90_REF_Y_NEG,
        ROT_90_REF_Y_POS,
        ROT_90_REF_Z_NEG,
        ROT_90_REF_Z_POS
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.math.OctahedralGroup");
        for value in OctahedralGroup::VALUES {
            golden.field(&format!("{value:?}"), value);
        }
        golden.field("BLOCK_ROT_X_270", OctahedralGroup::BLOCK_ROT_X_270);
        golden.field("BLOCK_ROT_X_180", OctahedralGroup::BLOCK_ROT_X_180);
        golden.field("BLOCK_ROT_X_90", OctahedralGroup::BLOCK_ROT_X_90);
        golden.field("BLOCK_ROT_Y_270", OctahedralGroup::BLOCK_ROT_Y_270);
        golden.field("BLOCK_ROT_Y_180", OctahedralGroup::BLOCK_ROT_Y_180);
        golden.field("BLOCK_ROT_Y_90", OctahedralGroup::BLOCK_ROT_Y_90);
        golden.field("BLOCK_ROT_Z_270", OctahedralGroup::BLOCK_ROT_Z_270);
        golden.field("BLOCK_ROT_Z_180", OctahedralGroup::BLOCK_ROT_Z_180);
        golden.field("BLOCK_ROT_Z_90", OctahedralGroup::BLOCK_ROT_Z_90);
        golden.check(
            "values()[Lcom/mojang/math/OctahedralGroup;",
            OctahedralGroup::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lcom/mojang/math/OctahedralGroup;",
            |name: String| OctahedralGroup::value_of(&name),
        );
        golden.check_method(
            "compose(Lcom/mojang/math/OctahedralGroup;)Lcom/mojang/math/OctahedralGroup;",
            |this: &mut OctahedralGroup, that| this.compose(that),
        );
        golden.check_method(
            "inverse()Lcom/mojang/math/OctahedralGroup;",
            |this: &mut OctahedralGroup| this.inverse(),
        );
        golden.check_method(
            "transformation()Lorg/joml/Matrix3fc;",
            |this: &mut OctahedralGroup| *this.transformation(),
        );
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut OctahedralGroup| this.to_string(),
        );
        golden.check_method(
            "getSerializedName()Ljava/lang/String;",
            |this: &mut OctahedralGroup| this.get_serialized_name().to_owned(),
        );
        golden.check_method(
            "rotate(Lnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;",
            |this: &mut OctahedralGroup, direction: Direction| this.rotate_direction(direction),
        );
        golden.check_mut(
            "rotate(Lorg/joml/Vector3i;)Lorg/joml/Vector3i;",
            |this: &mut OctahedralGroup, v: &mut Vector3i| *this.rotate_vector3i(v),
        );
        golden.check_method(
            "inverts(Lnet/minecraft/core/Direction$Axis;)Z",
            |this: &mut OctahedralGroup, axis: Axis| this.inverts(axis),
        );
        golden.check_method(
            "permutation()Lcom/mojang/math/SymmetricGroup3;",
            |this: &mut OctahedralGroup| -> SymmetricGroup3 { this.permutation() },
        );
        golden.check_method(
            "rotate(Lnet/minecraft/core/FrontAndTop;)Lnet/minecraft/core/FrontAndTop;",
            |this: &mut OctahedralGroup, input: FrontAndTop| this.rotate_front_and_top(input),
        );
        golden.finish();
    }
}
