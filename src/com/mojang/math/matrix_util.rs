//! com/mojang/math/MatrixUtil.java
//!
//! `eigenvalueJacobi` and `svdDecompose`, with the `GivensParameters` they work with, aren't
//! ported yet: only `Transformation` calls them.

use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::matrix4fc::{PROPERTY_IDENTITY, PROPERTY_TRANSLATION};

pub struct MatrixUtil;

impl MatrixUtil {
    pub fn mul_component_wise(m: &mut Matrix4f, factor: f32) -> &mut Matrix4f {
        m.set_values(
            m.m00() * factor,
            m.m01() * factor,
            m.m02() * factor,
            m.m03() * factor,
            m.m10() * factor,
            m.m11() * factor,
            m.m12() * factor,
            m.m13() * factor,
            m.m20() * factor,
            m.m21() * factor,
            m.m22() * factor,
            m.m23() * factor,
            m.m30() * factor,
            m.m31() * factor,
            m.m32() * factor,
            m.m33() * factor,
        )
    }

    pub fn check_property_raw(matrix: &Matrix4f, property: i32) -> bool {
        (matrix.properties() & property) != 0
    }

    /// When the matrix doesn't claim the property, works its properties out again and keeps
    /// them, on top of the ones it had. Java takes any `Matrix4fc` and gives up on one that isn't
    /// a `Matrix4f`, but JOML has no such class.
    pub fn check_property(matrix: &mut Matrix4f, property: i32) -> bool {
        if MatrixUtil::check_property_raw(matrix, property) {
            return true;
        }
        let current_properties = matrix.properties();
        matrix.determine_properties();
        let properties = matrix.properties() | current_properties;
        matrix.assume(properties);
        MatrixUtil::check_property_raw(matrix, property)
    }

    pub fn is_identity(matrix: &mut Matrix4f) -> bool {
        MatrixUtil::check_property(matrix, PROPERTY_IDENTITY)
    }

    pub fn is_pure_translation(matrix: &mut Matrix4f) -> bool {
        MatrixUtil::check_property(matrix, PROPERTY_TRANSLATION)
    }
}

#[cfg(test)]
mod tests {
    use super::MatrixUtil;
    use crate::golden::Golden;
    use crate::org::joml::matrix4f::Matrix4f;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.math.MatrixUtil");
        golden.check_method(
            "mulComponentWise(Lorg/joml/Matrix4f;F)Lorg/joml/Matrix4f;",
            |m: &mut Matrix4f, factor| *MatrixUtil::mul_component_wise(m, factor),
        );
        golden.check(
            "checkPropertyRaw(Lorg/joml/Matrix4fc;I)Z",
            |matrix: Matrix4f, property| MatrixUtil::check_property_raw(&matrix, property),
        );
        golden.check_method(
            "checkProperty(Lorg/joml/Matrix4fc;I)Z",
            MatrixUtil::check_property,
        );
        golden.check_method("isIdentity(Lorg/joml/Matrix4fc;)Z", MatrixUtil::is_identity);
        golden.check_method(
            "isPureTranslation(Lorg/joml/Matrix4fc;)Z",
            MatrixUtil::is_pure_translation,
        );
        let reason = "not ported: only Transformation calls it (vanilla skips it too, since it returns void)";
        golden.pending(
            "eigenvalueJacobi(Lorg/joml/Matrix3f;ILorg/joml/Quaternionf;)V",
            reason,
        );
        golden.pending(
            "svdDecompose(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;Lorg/joml/Quaternionf;Lorg/joml/Vector3f;Lorg/joml/Quaternionf;)V",
            reason,
        );
        golden.finish();
    }
}
