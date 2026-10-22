//! org/joml/Matrix4fc.java (JOML 1.10.9)
//!
//! The read-only view of a `Matrix4f`, which is the only class implementing it, so its
//! methods are `Matrix4f`'s and only its constants live here. Java declares them as bytes.

/// A matrix's properties are a guarantee about its values that lets methods take shortcuts:
/// none of them.
pub const PROPERTY_UNKNOWN: i32 = 0;
/// A perspective projection, `perspective()`'s or `frustum()`'s.
pub const PROPERTY_PERSPECTIVE: i32 = 1 << 0;
/// The last row is (0, 0, 0, 1).
pub const PROPERTY_AFFINE: i32 = 1 << 1;
pub const PROPERTY_IDENTITY: i32 = 1 << 2;
/// Affine, with the identity in the upper-left 3x3: at most a translation.
pub const PROPERTY_TRANSLATION: i32 = 1 << 3;
/// The upper-left 3x3 is a rotation: orthonormal columns, no scale.
pub const PROPERTY_ORTHONORMAL: i32 = 1 << 4;

pub const PLANE_NX: i32 = 0;
pub const PLANE_PX: i32 = 1;
pub const PLANE_NY: i32 = 2;
pub const PLANE_PY: i32 = 3;
pub const PLANE_NZ: i32 = 4;
pub const PLANE_PZ: i32 = 5;

pub const CORNER_NXNYNZ: i32 = 0;
pub const CORNER_PXNYNZ: i32 = 1;
pub const CORNER_PXPYNZ: i32 = 2;
pub const CORNER_NXPYNZ: i32 = 3;
pub const CORNER_PXNYPZ: i32 = 4;
pub const CORNER_NXNYPZ: i32 = 5;
pub const CORNER_NXPYPZ: i32 = 6;
pub const CORNER_PXPYPZ: i32 = 7;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Matrix4fc");
        golden.field("PROPERTY_UNKNOWN", PROPERTY_UNKNOWN);
        golden.field("PROPERTY_PERSPECTIVE", PROPERTY_PERSPECTIVE);
        golden.field("PROPERTY_AFFINE", PROPERTY_AFFINE);
        golden.field("PROPERTY_IDENTITY", PROPERTY_IDENTITY);
        golden.field("PROPERTY_TRANSLATION", PROPERTY_TRANSLATION);
        golden.field("PROPERTY_ORTHONORMAL", PROPERTY_ORTHONORMAL);
        golden.field("PLANE_NX", PLANE_NX);
        golden.field("PLANE_PX", PLANE_PX);
        golden.field("PLANE_NY", PLANE_NY);
        golden.field("PLANE_PY", PLANE_PY);
        golden.field("PLANE_NZ", PLANE_NZ);
        golden.field("PLANE_PZ", PLANE_PZ);
        golden.field("CORNER_NXNYNZ", CORNER_NXNYNZ);
        golden.field("CORNER_PXNYNZ", CORNER_PXNYNZ);
        golden.field("CORNER_PXPYNZ", CORNER_PXPYNZ);
        golden.field("CORNER_NXPYNZ", CORNER_NXPYNZ);
        golden.field("CORNER_PXNYPZ", CORNER_PXNYPZ);
        golden.field("CORNER_NXNYPZ", CORNER_NXNYPZ);
        golden.field("CORNER_NXPYPZ", CORNER_NXPYPZ);
        golden.field("CORNER_PXPYPZ", CORNER_PXPYPZ);
        golden.finish();
    }
}
