//! com/mojang/math/MatrixUtil.java

use std::f64::consts::PI;
use std::sync::LazyLock;

use crate::com::mojang::math::givens_parameters::GivensParameters;
use crate::java::lang::math::Math as JavaMath;
use crate::org::joml::math::Math;
use crate::org::joml::matrix3f::Matrix3f;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::matrix4fc::{PROPERTY_IDENTITY, PROPERTY_TRANSLATION};
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

static G: LazyLock<f32> = LazyLock::new(|| 3.0 + 2.0 * Math::sqrt_f32(2.0));
static PI_4: LazyLock<GivensParameters> =
    LazyLock::new(|| GivensParameters::from_positive_angle((PI / 4.0) as f32));

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

    // The rotation that about zeroes the off-diagonal entry of the symmetric 2x2 matrix
    // [[a11, a12], [a12, a22]], or a quarter turn when that would be too far off.
    fn approx_givens_quat(a11: f32, a12: f32, a22: f32) -> GivensParameters {
        let ch = 2.0 * (a11 - a22);
        let sh = a12;
        if *G * sh * sh < ch * ch {
            GivensParameters::from_unnormalized(sh, ch)
        } else {
            *PI_4
        }
    }

    // The rotation that zeroes a2 in the vector (a1, a2), the step of a QR decomposition.
    fn qr_givens_quat(a1: f32, a2: f32) -> GivensParameters {
        let p = JavaMath::hypot(f64::from(a1), f64::from(a2)) as f32;
        let mut sh = if p > 1.0E-6 { a2 } else { 0.0 };
        let mut ch = Math::abs_f32(a1) + Math::max_f32(p, 1.0E-6);
        if a1 < 0.0 {
            std::mem::swap(&mut sh, &mut ch);
        }
        GivensParameters::from_unnormalized(sh, ch)
    }

    // a = q^T a q.
    fn similarity_transform(a: &mut Matrix3f, q: &mut Matrix3f) {
        a.mul(q);
        q.transpose();
        q.mul(a);
        a.set(q);
    }

    // One sweep of Jacobi rotations over the xy, xz and yz pairs, each applied to m and
    // collected in output.
    fn step_jacobi(
        m: &mut Matrix3f,
        tmp_mat: &mut Matrix3f,
        tmp_q: &mut Quaternionf,
        output: &mut Quaternionf,
    ) {
        if m.m01 * m.m01 + m.m10 * m.m10 > 1.0E-6 {
            let p = MatrixUtil::approx_givens_quat(m.m00, 0.5 * (m.m01 + m.m10), m.m11);
            let qt = p.around_z_quaternionf(tmp_q);
            output.mul(qt);
            p.around_z_matrix3f(tmp_mat);
            MatrixUtil::similarity_transform(m, tmp_mat);
        }

        if m.m02 * m.m02 + m.m20 * m.m20 > 1.0E-6 {
            let p = MatrixUtil::approx_givens_quat(m.m00, 0.5 * (m.m02 + m.m20), m.m22).inverse();
            let qt = p.around_y_quaternionf(tmp_q);
            output.mul(qt);
            p.around_y_matrix3f(tmp_mat);
            MatrixUtil::similarity_transform(m, tmp_mat);
        }

        if m.m12 * m.m12 + m.m21 * m.m21 > 1.0E-6 {
            let p = MatrixUtil::approx_givens_quat(m.m11, 0.5 * (m.m12 + m.m21), m.m22);
            let qt = p.around_x_quaternionf(tmp_q);
            output.mul(qt);
            p.around_x_matrix3f(tmp_mat);
            MatrixUtil::similarity_transform(m, tmp_mat);
        }
    }

    /// Diagonalizes the symmetric `in_out` in place with `steps` sweeps of Jacobi rotations, and
    /// puts the rotation that does it in `result`.
    pub fn eigenvalue_jacobi(in_out: &mut Matrix3f, steps: i32, result: &mut Quaternionf) {
        result.identity();
        let mut scratch_mat = Matrix3f::new();
        let mut scratch_q = Quaternionf::new();

        for _ in 0..steps {
            MatrixUtil::step_jacobi(in_out, &mut scratch_mat, &mut scratch_q, result);
        }

        result.normalize();
    }

    /// Splits `input` into a translation `t`, then a rotation `u`, a scale `s` and a rotation `v`,
    /// all divided by its `m33`.
    pub fn svd_decompose(
        input: &Matrix4f,
        t: &mut Vector3f,
        u: &mut Quaternionf,
        s: &mut Vector3f,
        v: &mut Quaternionf,
    ) {
        let scale_factor = 1.0 / input.m33();
        let mut matrix = Matrix3f::from_matrix4f(input);
        matrix.scale_scalar(scale_factor);
        MatrixUtil::svd_decompose_matrix3f(matrix, u, s, v);
        input.get_translation(t).mul_scalar(scale_factor);
    }

    // Java names four matrices after what they hold as it goes (u012s, u12s, u2s, sm), but they
    // are two objects, `input` and `scratch`, which the steps overwrite in turn.
    fn svd_decompose_matrix3f(
        mut input: Matrix3f,
        u: &mut Quaternionf,
        s: &mut Vector3f,
        v: &mut Quaternionf,
    ) {
        let mut scratch = Matrix3f::from_matrix3f(&input);
        scratch.transpose();
        scratch.mul(&input);
        MatrixUtil::eigenvalue_jacobi(&mut scratch, 5, v);
        let column_scale_square0 = scratch.m00;
        let column_scale_square1 = scratch.m11;
        // Compared as doubles: the bytecode widens them.
        let zero_column0 = f64::from(column_scale_square0) < 1.0E-6;
        let zero_column1 = f64::from(column_scale_square1) < 1.0E-6;
        let u012s = input.rotate(v);
        u.identity();
        let mut tmp_q = Quaternionf::new();
        let mut p = if zero_column0 {
            MatrixUtil::qr_givens_quat(u012s.m11, -u012s.m10)
        } else {
            MatrixUtil::qr_givens_quat(u012s.m00, u012s.m01)
        };

        let qt0 = p.around_z_quaternionf(&mut tmp_q);
        let u12s = p.around_z_matrix3f(&mut scratch);
        u.mul(qt0);
        u12s.transpose().mul(u012s);
        // Java's scratch = u012s: the next rotation goes into input.
        p = if zero_column0 {
            MatrixUtil::qr_givens_quat(u12s.m22, -u12s.m20)
        } else {
            MatrixUtil::qr_givens_quat(u12s.m00, u12s.m02)
        };

        p = p.inverse();
        let qt1 = p.around_y_quaternionf(&mut tmp_q);
        let u2s = p.around_y_matrix3f(u012s);
        u.mul(qt1);
        u2s.transpose().mul(u12s);
        // Java's scratch = u12s: the next rotation goes back into scratch.
        p = if zero_column1 {
            MatrixUtil::qr_givens_quat(u2s.m22, -u2s.m21)
        } else {
            MatrixUtil::qr_givens_quat(u2s.m11, u2s.m12)
        };

        let qt2 = p.around_x_quaternionf(&mut tmp_q);
        let sm = p.around_x_matrix3f(u12s);
        u.mul(qt2);
        sm.transpose().mul(u2s);
        s.set_xyz(sm.m00, sm.m11, sm.m22);
        v.conjugate();
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
    use crate::org::joml::matrix3f::Matrix3f;
    use crate::org::joml::matrix4f::Matrix4f;
    use crate::org::joml::quaternionf::Quaternionf;
    use crate::org::joml::vector3f::Vector3f;

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
        golden.check_mut(
            "eigenvalueJacobi(Lorg/joml/Matrix3f;ILorg/joml/Quaternionf;)V",
            |in_out: &mut Matrix3f, steps: &mut i32, result: &mut Quaternionf| {
                MatrixUtil::eigenvalue_jacobi(in_out, *steps, result)
            },
        );
        golden.check_mut(
            "svdDecompose(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;Lorg/joml/Quaternionf;Lorg/joml/Vector3f;Lorg/joml/Quaternionf;)V",
            |input: &mut Matrix4f,
             t: &mut Vector3f,
             u: &mut Quaternionf,
             s: &mut Vector3f,
             v: &mut Quaternionf| MatrixUtil::svd_decompose(input, t, u, s, v),
        );
        golden.finish();
    }
}
