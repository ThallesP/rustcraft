//! org/joml/Matrix4f.java (JOML 1.10.9)
//!
//! Column-major like the shaders: `m30` is column 3, row 0, the x translation. Methods take
//! shortcuts based on `properties` (see `matrix4fc`), which can give different bits than the
//! full computation would (an identity with -0.0 in it stays that way), so they're ported too.
//! Only the methods the client calls so far: the golden suite in oracle/golden/ lists them.
//!
//! A fluent overload passes `this` as JOML's `dest`, so it's ported as writing into `self` from a
//! copy of it: each of these reads all of `this` before writing `dest`. `translate` has a body of
//! its own, and `rotation_around` reads back what it wrote, so they're ported on `&mut self`.

use crate::java::lang::float::Float;
use crate::org::joml::math::Math;
use crate::org::joml::matrix3x2f::Matrix3x2f;
use crate::org::joml::matrix4fc::{
    PROPERTY_AFFINE, PROPERTY_IDENTITY, PROPERTY_ORTHONORMAL, PROPERTY_PERSPECTIVE,
    PROPERTY_TRANSLATION, PROPERTY_UNKNOWN,
};
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;
use crate::org::joml::vector4f::Vector4f;

#[derive(Clone, Copy, Debug)]
pub struct Matrix4f {
    properties: i32,
    m00: f32,
    m01: f32,
    m02: f32,
    m03: f32,
    m10: f32,
    m11: f32,
    m12: f32,
    m13: f32,
    m20: f32,
    m21: f32,
    m22: f32,
    m23: f32,
    m30: f32,
    m31: f32,
    m32: f32,
    m33: f32,
}

impl Default for Matrix4f {
    fn default() -> Matrix4f {
        Matrix4f::new()
    }
}

/// JOML's `equals`: the components compare by `Float.floatToIntBits`, so -0.0 differs from 0.0
/// and NaN equals NaN, and the properties don't count.
impl PartialEq for Matrix4f {
    fn eq(&self, other: &Matrix4f) -> bool {
        self.fields().map(Float::float_to_int_bits) == other.fields().map(Float::float_to_int_bits)
    }
}

impl Eq for Matrix4f {}

macro_rules! getters {
    ($($field:ident),*) => {$(
        pub fn $field(&self) -> f32 {
            self.$field
        }
    )*};
}

impl Matrix4f {
    const ZERO: Matrix4f = Matrix4f {
        properties: PROPERTY_UNKNOWN,
        m00: 0.0,
        m01: 0.0,
        m02: 0.0,
        m03: 0.0,
        m10: 0.0,
        m11: 0.0,
        m12: 0.0,
        m13: 0.0,
        m20: 0.0,
        m21: 0.0,
        m22: 0.0,
        m23: 0.0,
        m30: 0.0,
        m31: 0.0,
        m32: 0.0,
        m33: 0.0,
    };

    /// The identity.
    pub const fn new() -> Matrix4f {
        Matrix4f {
            properties: PROPERTY_IDENTITY
                | PROPERTY_AFFINE
                | PROPERTY_TRANSLATION
                | PROPERTY_ORTHONORMAL,
            m00: 1.0,
            m01: 0.0,
            m02: 0.0,
            m03: 0.0,
            m10: 0.0,
            m11: 1.0,
            m12: 0.0,
            m13: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 0.0,
            m30: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
        }
    }

    pub fn from_matrix4f(mat: &Matrix4f) -> Matrix4f {
        *mat
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_values(
        m00: f32,
        m01: f32,
        m02: f32,
        m03: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m20: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m30: f32,
        m31: f32,
        m32: f32,
        m33: f32,
    ) -> Matrix4f {
        let mut matrix = Matrix4f {
            properties: PROPERTY_UNKNOWN,
            m00,
            m01,
            m02,
            m03,
            m10,
            m11,
            m12,
            m13,
            m20,
            m21,
            m22,
            m23,
            m30,
            m31,
            m32,
            m33,
        };
        matrix.determine_properties();
        matrix
    }

    pub fn determine_properties(&mut self) -> &mut Self {
        let mut properties = PROPERTY_UNKNOWN;
        if self.m03 == 0.0 && self.m13 == 0.0 {
            if self.m23 == 0.0 && self.m33 == 1.0 {
                properties |= PROPERTY_AFFINE;
                if self.m00 == 1.0
                    && self.m01 == 0.0
                    && self.m02 == 0.0
                    && self.m10 == 0.0
                    && self.m11 == 1.0
                    && self.m12 == 0.0
                    && self.m20 == 0.0
                    && self.m21 == 0.0
                    && self.m22 == 1.0
                {
                    properties |= PROPERTY_TRANSLATION | PROPERTY_ORTHONORMAL;
                    if self.m30 == 0.0 && self.m31 == 0.0 && self.m32 == 0.0 {
                        properties |= PROPERTY_IDENTITY;
                    }
                }
                // Orthonormality isn't determined: it would take epsilons.
            } else if self.m01 == 0.0
                && self.m02 == 0.0
                && self.m10 == 0.0
                && self.m12 == 0.0
                && self.m20 == 0.0
                && self.m21 == 0.0
                && self.m30 == 0.0
                && self.m31 == 0.0
                && self.m33 == 0.0
            {
                properties |= PROPERTY_PERSPECTIVE;
            }
        }
        self.properties = properties;
        self
    }

    pub fn properties(&self) -> i32 {
        self.properties
    }

    pub fn is_finite(&self) -> bool {
        self.fields().iter().all(|&m| Math::is_finite_f32(m))
    }

    /// Like `Float.hashCode` of each component in turn: every NaN hashes the same.
    pub fn hash_code(&self) -> i32 {
        self.fields().iter().fold(1i32, |result, &m| {
            result
                .wrapping_mul(31)
                .wrapping_add(Float::float_to_int_bits(m))
        })
    }

    fn fields(&self) -> [f32; 16] {
        [
            self.m00, self.m01, self.m02, self.m03, self.m10, self.m11, self.m12, self.m13,
            self.m20, self.m21, self.m22, self.m23, self.m30, self.m31, self.m32, self.m33,
        ]
    }

    /// Sets the properties without checking them, so they can promise more than the matrix is.
    pub fn assume(&mut self, properties: i32) -> &mut Self {
        self.properties = properties;
        self
    }

    getters!(
        m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33
    );

    // MemUtil.INSTANCE.identity(this): the fields only, not the properties.
    fn identity_fields(&mut self) {
        let properties = self.properties;
        *self = Matrix4f {
            properties,
            ..Matrix4f::new()
        };
    }

    // MemUtil.INSTANCE.zero(this): the fields only, not the properties.
    fn zero_fields(&mut self) {
        let properties = self.properties;
        *self = Matrix4f {
            properties,
            ..Matrix4f::ZERO
        };
    }

    pub fn identity(&mut self) -> &mut Self {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            return self;
        }
        *self = Matrix4f::new();
        self
    }

    pub fn set(&mut self, m: &Matrix4f) -> &mut Self {
        *self = *m;
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_values(
        &mut self,
        m00: f32,
        m01: f32,
        m02: f32,
        m03: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m20: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m30: f32,
        m31: f32,
        m32: f32,
        m33: f32,
    ) -> &mut Self {
        *self = Matrix4f::from_values(
            m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33,
        );
        self
    }

    pub fn mul(&mut self, right: &Matrix4f) -> &mut Self {
        let this = *self;
        this.mul_into(right, self)
    }

    pub fn mul_into<'a>(&self, right: &Matrix4f, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            dest.set(right)
        } else if (right.properties() & PROPERTY_IDENTITY) != 0 {
            dest.set(self)
        } else if (self.properties & PROPERTY_TRANSLATION) != 0
            && (right.properties() & PROPERTY_AFFINE) != 0
        {
            self.mul_translation_affine_into(right, dest)
        } else if (self.properties & PROPERTY_AFFINE) != 0
            && (right.properties() & PROPERTY_AFFINE) != 0
        {
            self.mul_affine_into(right, dest)
        } else if (self.properties & PROPERTY_PERSPECTIVE) != 0
            && (right.properties() & PROPERTY_AFFINE) != 0
        {
            self.mul_perspective_affine_into(right, dest)
        } else if (right.properties() & PROPERTY_AFFINE) != 0 {
            self.mul_affine_r_into(right, dest)
        } else {
            self.mul0_into(right, dest)
        }
    }

    /// Multiplies by `right` as the 4x4 matrix with its 2x2 part top left, its translation in
    /// the last column, and z left alone, which is how the GUI's glyphs go from 2D to 3D.
    pub fn mul_matrix3x2f(&mut self, right: &Matrix3x2f) -> &mut Self {
        let this = *self;
        this.mul_matrix3x2f_into(right, self)
    }

    pub fn mul_matrix3x2f_into<'a>(
        &self,
        right: &Matrix3x2f,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let nm00 = self.m00 * right.m00 + self.m10 * right.m01;
        let nm01 = self.m01 * right.m00 + self.m11 * right.m01;
        let nm02 = self.m02 * right.m00 + self.m12 * right.m01;
        let nm03 = self.m03 * right.m00 + self.m13 * right.m01;
        let nm10 = self.m00 * right.m10 + self.m10 * right.m11;
        let nm11 = self.m01 * right.m10 + self.m11 * right.m11;
        let nm12 = self.m02 * right.m10 + self.m12 * right.m11;
        let nm13 = self.m03 * right.m10 + self.m13 * right.m11;
        let nm30 = self.m00 * right.m20 + self.m10 * right.m21 + self.m30;
        let nm31 = self.m01 * right.m20 + self.m11 * right.m21 + self.m31;
        let nm32 = self.m02 * right.m20 + self.m12 * right.m21 + self.m32;
        let nm33 = self.m03 * right.m20 + self.m13 * right.m21 + self.m33;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = nm03;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m13 = nm13;
        dest.m20 = self.m20;
        dest.m21 = self.m21;
        dest.m22 = self.m22;
        dest.m23 = self.m23;
        dest.m30 = nm30;
        dest.m31 = nm31;
        dest.m32 = nm32;
        dest.m33 = nm33;
        dest.properties = self.properties
            & !(PROPERTY_IDENTITY
                | PROPERTY_PERSPECTIVE
                | PROPERTY_TRANSLATION
                | PROPERTY_ORTHONORMAL);
        dest
    }

    fn mul0_into<'a>(&self, right: &Matrix4f, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        let fma = Math::fma_f32;
        let nm00 = fma(
            self.m00,
            right.m00,
            fma(
                self.m10,
                right.m01,
                fma(self.m20, right.m02, self.m30 * right.m03),
            ),
        );
        let nm01 = fma(
            self.m01,
            right.m00,
            fma(
                self.m11,
                right.m01,
                fma(self.m21, right.m02, self.m31 * right.m03),
            ),
        );
        let nm02 = fma(
            self.m02,
            right.m00,
            fma(
                self.m12,
                right.m01,
                fma(self.m22, right.m02, self.m32 * right.m03),
            ),
        );
        let nm03 = fma(
            self.m03,
            right.m00,
            fma(
                self.m13,
                right.m01,
                fma(self.m23, right.m02, self.m33 * right.m03),
            ),
        );
        let nm10 = fma(
            self.m00,
            right.m10,
            fma(
                self.m10,
                right.m11,
                fma(self.m20, right.m12, self.m30 * right.m13),
            ),
        );
        let nm11 = fma(
            self.m01,
            right.m10,
            fma(
                self.m11,
                right.m11,
                fma(self.m21, right.m12, self.m31 * right.m13),
            ),
        );
        let nm12 = fma(
            self.m02,
            right.m10,
            fma(
                self.m12,
                right.m11,
                fma(self.m22, right.m12, self.m32 * right.m13),
            ),
        );
        let nm13 = fma(
            self.m03,
            right.m10,
            fma(
                self.m13,
                right.m11,
                fma(self.m23, right.m12, self.m33 * right.m13),
            ),
        );
        let nm20 = fma(
            self.m00,
            right.m20,
            fma(
                self.m10,
                right.m21,
                fma(self.m20, right.m22, self.m30 * right.m23),
            ),
        );
        let nm21 = fma(
            self.m01,
            right.m20,
            fma(
                self.m11,
                right.m21,
                fma(self.m21, right.m22, self.m31 * right.m23),
            ),
        );
        let nm22 = fma(
            self.m02,
            right.m20,
            fma(
                self.m12,
                right.m21,
                fma(self.m22, right.m22, self.m32 * right.m23),
            ),
        );
        let nm23 = fma(
            self.m03,
            right.m20,
            fma(
                self.m13,
                right.m21,
                fma(self.m23, right.m22, self.m33 * right.m23),
            ),
        );
        let nm30 = fma(
            self.m00,
            right.m30,
            fma(
                self.m10,
                right.m31,
                fma(self.m20, right.m32, self.m30 * right.m33),
            ),
        );
        let nm31 = fma(
            self.m01,
            right.m30,
            fma(
                self.m11,
                right.m31,
                fma(self.m21, right.m32, self.m31 * right.m33),
            ),
        );
        let nm32 = fma(
            self.m02,
            right.m30,
            fma(
                self.m12,
                right.m31,
                fma(self.m22, right.m32, self.m32 * right.m33),
            ),
        );
        let nm33 = fma(
            self.m03,
            right.m30,
            fma(
                self.m13,
                right.m31,
                fma(self.m23, right.m32, self.m33 * right.m33),
            ),
        );
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = nm03;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m13 = nm13;
        dest.m20 = nm20;
        dest.m21 = nm21;
        dest.m22 = nm22;
        dest.m23 = nm23;
        dest.m30 = nm30;
        dest.m31 = nm31;
        dest.m32 = nm32;
        dest.m33 = nm33;
        dest.properties = PROPERTY_UNKNOWN;
        dest
    }

    fn mul_perspective_affine_into<'a>(
        &self,
        view: &Matrix4f,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let (nm00, nm01, nm02, nm03) = (
            self.m00 * view.m00,
            self.m11 * view.m01,
            self.m22 * view.m02,
            self.m23 * view.m02,
        );
        let (nm10, nm11, nm12, nm13) = (
            self.m00 * view.m10,
            self.m11 * view.m11,
            self.m22 * view.m12,
            self.m23 * view.m12,
        );
        let (nm20, nm21, nm22, nm23) = (
            self.m00 * view.m20,
            self.m11 * view.m21,
            self.m22 * view.m22,
            self.m23 * view.m22,
        );
        let (nm30, nm31, nm32, nm33) = (
            self.m00 * view.m30,
            self.m11 * view.m31,
            self.m22 * view.m32 + self.m32,
            self.m23 * view.m32,
        );
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = nm03;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m13 = nm13;
        dest.m20 = nm20;
        dest.m21 = nm21;
        dest.m22 = nm22;
        dest.m23 = nm23;
        dest.m30 = nm30;
        dest.m31 = nm31;
        dest.m32 = nm32;
        dest.m33 = nm33;
        dest.properties = PROPERTY_UNKNOWN;
        dest
    }

    fn mul_affine_r_into<'a>(&self, right: &Matrix4f, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        let fma = Math::fma_f32;
        let nm00 = fma(
            self.m00,
            right.m00,
            fma(self.m10, right.m01, self.m20 * right.m02),
        );
        let nm01 = fma(
            self.m01,
            right.m00,
            fma(self.m11, right.m01, self.m21 * right.m02),
        );
        let nm02 = fma(
            self.m02,
            right.m00,
            fma(self.m12, right.m01, self.m22 * right.m02),
        );
        let nm03 = fma(
            self.m03,
            right.m00,
            fma(self.m13, right.m01, self.m23 * right.m02),
        );
        let nm10 = fma(
            self.m00,
            right.m10,
            fma(self.m10, right.m11, self.m20 * right.m12),
        );
        let nm11 = fma(
            self.m01,
            right.m10,
            fma(self.m11, right.m11, self.m21 * right.m12),
        );
        let nm12 = fma(
            self.m02,
            right.m10,
            fma(self.m12, right.m11, self.m22 * right.m12),
        );
        let nm13 = fma(
            self.m03,
            right.m10,
            fma(self.m13, right.m11, self.m23 * right.m12),
        );
        let nm20 = fma(
            self.m00,
            right.m20,
            fma(self.m10, right.m21, self.m20 * right.m22),
        );
        let nm21 = fma(
            self.m01,
            right.m20,
            fma(self.m11, right.m21, self.m21 * right.m22),
        );
        let nm22 = fma(
            self.m02,
            right.m20,
            fma(self.m12, right.m21, self.m22 * right.m22),
        );
        let nm23 = fma(
            self.m03,
            right.m20,
            fma(self.m13, right.m21, self.m23 * right.m22),
        );
        let nm30 = fma(
            self.m00,
            right.m30,
            fma(self.m10, right.m31, fma(self.m20, right.m32, self.m30)),
        );
        let nm31 = fma(
            self.m01,
            right.m30,
            fma(self.m11, right.m31, fma(self.m21, right.m32, self.m31)),
        );
        let nm32 = fma(
            self.m02,
            right.m30,
            fma(self.m12, right.m31, fma(self.m22, right.m32, self.m32)),
        );
        let nm33 = fma(
            self.m03,
            right.m30,
            fma(self.m13, right.m31, fma(self.m23, right.m32, self.m33)),
        );
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = nm03;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m13 = nm13;
        dest.m20 = nm20;
        dest.m21 = nm21;
        dest.m22 = nm22;
        dest.m23 = nm23;
        dest.m30 = nm30;
        dest.m31 = nm31;
        dest.m32 = nm32;
        dest.m33 = nm33;
        dest.properties = self.properties
            & !(PROPERTY_IDENTITY
                | PROPERTY_PERSPECTIVE
                | PROPERTY_TRANSLATION
                | PROPERTY_ORTHONORMAL);
        dest
    }

    fn mul_affine_into<'a>(&self, right: &Matrix4f, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        let fma = Math::fma_f32;
        let (m00, m01, m02) = (self.m00, self.m01, self.m02);
        let (m10, m11, m12) = (self.m10, self.m11, self.m12);
        let (m20, m21, m22) = (self.m20, self.m21, self.m22);
        let (rm00, rm01, rm02) = (right.m00, right.m01, right.m02);
        let (rm10, rm11, rm12) = (right.m10, right.m11, right.m12);
        let (rm20, rm21, rm22) = (right.m20, right.m21, right.m22);
        let (rm30, rm31, rm32) = (right.m30, right.m31, right.m32);
        dest.m00 = fma(m00, rm00, fma(m10, rm01, m20 * rm02));
        dest.m01 = fma(m01, rm00, fma(m11, rm01, m21 * rm02));
        dest.m02 = fma(m02, rm00, fma(m12, rm01, m22 * rm02));
        dest.m03 = self.m03;
        dest.m10 = fma(m00, rm10, fma(m10, rm11, m20 * rm12));
        dest.m11 = fma(m01, rm10, fma(m11, rm11, m21 * rm12));
        dest.m12 = fma(m02, rm10, fma(m12, rm11, m22 * rm12));
        dest.m13 = self.m13;
        dest.m20 = fma(m00, rm20, fma(m10, rm21, m20 * rm22));
        dest.m21 = fma(m01, rm20, fma(m11, rm21, m21 * rm22));
        dest.m22 = fma(m02, rm20, fma(m12, rm21, m22 * rm22));
        dest.m23 = self.m23;
        dest.m30 = fma(m00, rm30, fma(m10, rm31, fma(m20, rm32, self.m30)));
        dest.m31 = fma(m01, rm30, fma(m11, rm31, fma(m21, rm32, self.m31)));
        dest.m32 = fma(m02, rm30, fma(m12, rm31, fma(m22, rm32, self.m32)));
        dest.m33 = self.m33;
        dest.properties =
            PROPERTY_AFFINE | (self.properties & right.properties() & PROPERTY_ORTHONORMAL);
        dest
    }

    fn mul_translation_affine_into<'a>(
        &self,
        right: &Matrix4f,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        dest.m00 = right.m00;
        dest.m01 = right.m01;
        dest.m02 = right.m02;
        dest.m03 = self.m03;
        dest.m10 = right.m10;
        dest.m11 = right.m11;
        dest.m12 = right.m12;
        dest.m13 = self.m13;
        dest.m20 = right.m20;
        dest.m21 = right.m21;
        dest.m22 = right.m22;
        dest.m23 = self.m23;
        dest.m30 = right.m30 + self.m30;
        dest.m31 = right.m31 + self.m31;
        dest.m32 = right.m32 + self.m32;
        dest.m33 = self.m33;
        dest.properties = PROPERTY_AFFINE | (right.properties() & PROPERTY_ORTHONORMAL);
        dest
    }

    pub fn invert_into<'a>(&self, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            dest.identity()
        } else if (self.properties & PROPERTY_TRANSLATION) != 0 {
            self.invert_translation_into(dest)
        } else if (self.properties & PROPERTY_ORTHONORMAL) != 0 {
            self.invert_orthonormal_into(dest)
        } else if (self.properties & PROPERTY_AFFINE) != 0 {
            self.invert_affine_into(dest)
        } else if (self.properties & PROPERTY_PERSPECTIVE) != 0 {
            self.invert_perspective_into(dest)
        } else {
            self.invert_generic_into(dest)
        }
    }

    fn invert_translation_into<'a>(&self, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        dest.set(self);
        dest.m30 = -self.m30;
        dest.m31 = -self.m31;
        dest.m32 = -self.m32;
        dest.properties = PROPERTY_AFFINE | PROPERTY_TRANSLATION | PROPERTY_ORTHONORMAL;
        dest
    }

    fn invert_orthonormal_into<'a>(&self, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        let nm30 = -(self.m00 * self.m30 + self.m01 * self.m31 + self.m02 * self.m32);
        let nm31 = -(self.m10 * self.m30 + self.m11 * self.m31 + self.m12 * self.m32);
        let nm32 = -(self.m20 * self.m30 + self.m21 * self.m31 + self.m22 * self.m32);
        dest.m00 = self.m00;
        dest.m01 = self.m10;
        dest.m02 = self.m20;
        dest.m03 = 0.0;
        dest.m10 = self.m01;
        dest.m11 = self.m11;
        dest.m12 = self.m21;
        dest.m13 = 0.0;
        dest.m20 = self.m02;
        dest.m21 = self.m12;
        dest.m22 = self.m22;
        dest.m23 = 0.0;
        dest.m30 = nm30;
        dest.m31 = nm31;
        dest.m32 = nm32;
        dest.m33 = 1.0;
        dest.properties = PROPERTY_AFFINE | PROPERTY_ORTHONORMAL;
        dest
    }

    // invertGenericNonThis. JOML's invertGenericThis computes the same into temporaries for
    // dest == this, which Rust's borrows rule out.
    fn invert_generic_into<'a>(&self, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        let fma = Math::fma_f32;
        let a = self.m00 * self.m11 - self.m01 * self.m10;
        let b = self.m00 * self.m12 - self.m02 * self.m10;
        let c = self.m00 * self.m13 - self.m03 * self.m10;
        let d = self.m01 * self.m12 - self.m02 * self.m11;
        let e = self.m01 * self.m13 - self.m03 * self.m11;
        let f = self.m02 * self.m13 - self.m03 * self.m12;
        let g = self.m20 * self.m31 - self.m21 * self.m30;
        let h = self.m20 * self.m32 - self.m22 * self.m30;
        let i = self.m20 * self.m33 - self.m23 * self.m30;
        let j = self.m21 * self.m32 - self.m22 * self.m31;
        let k = self.m21 * self.m33 - self.m23 * self.m31;
        let l = self.m22 * self.m33 - self.m23 * self.m32;
        let mut det = a * l - b * k + c * j + d * i - e * h + f * g;
        det = 1.0 / det;
        dest.m00 = fma(self.m11, l, fma(-self.m12, k, self.m13 * j)) * det;
        dest.m01 = fma(-self.m01, l, fma(self.m02, k, -self.m03 * j)) * det;
        dest.m02 = fma(self.m31, f, fma(-self.m32, e, self.m33 * d)) * det;
        dest.m03 = fma(-self.m21, f, fma(self.m22, e, -self.m23 * d)) * det;
        dest.m10 = fma(-self.m10, l, fma(self.m12, i, -self.m13 * h)) * det;
        dest.m11 = fma(self.m00, l, fma(-self.m02, i, self.m03 * h)) * det;
        dest.m12 = fma(-self.m30, f, fma(self.m32, c, -self.m33 * b)) * det;
        dest.m13 = fma(self.m20, f, fma(-self.m22, c, self.m23 * b)) * det;
        dest.m20 = fma(self.m10, k, fma(-self.m11, i, self.m13 * g)) * det;
        dest.m21 = fma(-self.m00, k, fma(self.m01, i, -self.m03 * g)) * det;
        dest.m22 = fma(self.m30, e, fma(-self.m31, c, self.m33 * a)) * det;
        dest.m23 = fma(-self.m20, e, fma(self.m21, c, -self.m23 * a)) * det;
        dest.m30 = fma(-self.m10, j, fma(self.m11, h, -self.m12 * g)) * det;
        dest.m31 = fma(self.m00, j, fma(-self.m01, h, self.m02 * g)) * det;
        dest.m32 = fma(-self.m30, d, fma(self.m31, b, -self.m32 * a)) * det;
        dest.m33 = fma(self.m20, d, fma(-self.m21, b, self.m22 * a)) * det;
        dest.properties = PROPERTY_UNKNOWN;
        dest
    }

    pub fn invert(&mut self) -> &mut Self {
        let this = *self;
        this.invert_into(self)
    }

    fn invert_perspective_into<'a>(&self, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        let a = 1.0 / (self.m00 * self.m11);
        let l = -1.0 / (self.m23 * self.m32);
        dest.set_values(
            self.m11 * a,
            0.0,
            0.0,
            0.0,
            0.0,
            self.m00 * a,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            -self.m23 * l,
            0.0,
            0.0,
            -self.m32 * l,
            self.m22 * l,
        );
        dest.properties = PROPERTY_UNKNOWN;
        dest
    }

    /// Inverts the matrix as an affine one, whatever its last row and properties say.
    pub fn invert_affine(&mut self) -> &mut Self {
        let this = *self;
        this.invert_affine_into(self)
    }

    pub fn invert_affine_into<'a>(&self, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        let (m11m00, m10m01, m10m02) = (
            self.m00 * self.m11,
            self.m01 * self.m10,
            self.m02 * self.m10,
        );
        let (m12m00, m12m01, m11m02) = (
            self.m00 * self.m12,
            self.m01 * self.m12,
            self.m02 * self.m11,
        );
        let det = (m11m00 - m10m01) * self.m22
            + (m10m02 - m12m00) * self.m21
            + (m12m01 - m11m02) * self.m20;
        let s = 1.0 / det;
        let (m10m22, m10m21, m11m22) = (
            self.m10 * self.m22,
            self.m10 * self.m21,
            self.m11 * self.m22,
        );
        let (m11m20, m12m21, m12m20) = (
            self.m11 * self.m20,
            self.m12 * self.m21,
            self.m12 * self.m20,
        );
        let (m20m02, m20m01, m21m02) = (
            self.m20 * self.m02,
            self.m20 * self.m01,
            self.m21 * self.m02,
        );
        let (m21m00, m22m01, m22m00) = (
            self.m21 * self.m00,
            self.m22 * self.m01,
            self.m22 * self.m00,
        );
        let nm31 = (m20m02 * self.m31 - m20m01 * self.m32 + m21m00 * self.m32 - m21m02 * self.m30
            + m22m01 * self.m30
            - m22m00 * self.m31)
            * s;
        let nm32 = (m11m02 * self.m30 - m12m01 * self.m30 + m12m00 * self.m31 - m10m02 * self.m31
            + m10m01 * self.m32
            - m11m00 * self.m32)
            * s;
        dest.m00 = (m11m22 - m12m21) * s;
        dest.m01 = (m21m02 - m22m01) * s;
        dest.m02 = (m12m01 - m11m02) * s;
        dest.m03 = 0.0;
        dest.m10 = (m12m20 - m10m22) * s;
        dest.m11 = (m22m00 - m20m02) * s;
        dest.m12 = (m10m02 - m12m00) * s;
        dest.m13 = 0.0;
        dest.m20 = (m10m21 - m11m20) * s;
        dest.m21 = (m20m01 - m21m00) * s;
        dest.m22 = (m11m00 - m10m01) * s;
        dest.m23 = 0.0;
        dest.m30 = (m10m22 * self.m31 - m10m21 * self.m32 + m11m20 * self.m32 - m11m22 * self.m30
            + m12m21 * self.m30
            - m12m20 * self.m31)
            * s;
        dest.m31 = nm31;
        dest.m32 = nm32;
        dest.m33 = 1.0;
        dest.properties = PROPERTY_AFFINE;
        dest
    }

    pub fn translation(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        if (self.properties & PROPERTY_IDENTITY) == 0 {
            self.identity_fields();
        }
        self.m30 = x;
        self.m31 = y;
        self.m32 = z;
        self.properties = PROPERTY_AFFINE | PROPERTY_TRANSLATION | PROPERTY_ORTHONORMAL;
        self
    }

    pub fn translation_vector3f(&mut self, offset: &Vector3f) -> &mut Self {
        self.translation(offset.x, offset.y, offset.z)
    }

    pub fn get_translation<'a>(&self, dest: &'a mut Vector3f) -> &'a mut Vector3f {
        dest.x = self.m30;
        dest.y = self.m31;
        dest.z = self.m32;
        dest
    }

    pub fn set_translation(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.m30 = x;
        self.m31 = y;
        self.m32 = z;
        self.properties &= !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY);
        self
    }

    pub fn scaling(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        if (self.properties & PROPERTY_IDENTITY) == 0 {
            self.identity_fields();
        }
        let one = Math::abs_equals_one_f32(x)
            && Math::abs_equals_one_f32(y)
            && Math::abs_equals_one_f32(z);
        self.m00 = x;
        self.m11 = y;
        self.m22 = z;
        self.properties = PROPERTY_AFFINE | if one { PROPERTY_ORTHONORMAL } else { 0 };
        self
    }

    pub fn rotation_axis(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.rotation_axis_xyz(angle, axis.x, axis.y, axis.z)
    }

    pub fn rotation_axis_xyz(&mut self, angle: f32, x: f32, y: f32, z: f32) -> &mut Self {
        if y == 0.0 && z == 0.0 && Math::abs_equals_one_f32(x) {
            self.rotation_x(x * angle)
        } else if x == 0.0 && z == 0.0 && Math::abs_equals_one_f32(y) {
            self.rotation_y(y * angle)
        } else if x == 0.0 && y == 0.0 && Math::abs_equals_one_f32(z) {
            self.rotation_z(z * angle)
        } else {
            self.rotation_internal(angle, x, y, z)
        }
    }

    fn rotation_internal(&mut self, angle: f32, x: f32, y: f32, z: f32) -> &mut Self {
        let sin = Math::sin_f32(angle);
        let cos = Math::cos_from_sin_f32(sin, angle);
        let one_minus_cos = 1.0 - cos;
        let (xy, xz, yz) = (x * y, x * z, y * z);
        if (self.properties & PROPERTY_IDENTITY) == 0 {
            self.identity_fields();
        }
        self.m00 = cos + x * x * one_minus_cos;
        self.m10 = xy * one_minus_cos - z * sin;
        self.m20 = xz * one_minus_cos + y * sin;
        self.m01 = xy * one_minus_cos + z * sin;
        self.m11 = cos + y * y * one_minus_cos;
        self.m21 = yz * one_minus_cos - x * sin;
        self.m02 = xz * one_minus_cos - y * sin;
        self.m12 = yz * one_minus_cos + x * sin;
        self.m22 = cos + z * z * one_minus_cos;
        self.properties = PROPERTY_AFFINE | PROPERTY_ORTHONORMAL;
        self
    }

    pub fn rotation_x(&mut self, ang: f32) -> &mut Self {
        let sin = Math::sin_f32(ang);
        let cos = Math::cos_from_sin_f32(sin, ang);
        if (self.properties & PROPERTY_IDENTITY) == 0 {
            self.identity_fields();
        }
        self.m11 = cos;
        self.m12 = sin;
        self.m21 = -sin;
        self.m22 = cos;
        self.properties = PROPERTY_AFFINE | PROPERTY_ORTHONORMAL;
        self
    }

    pub fn rotation_y(&mut self, ang: f32) -> &mut Self {
        let sin = Math::sin_f32(ang);
        let cos = Math::cos_from_sin_f32(sin, ang);
        if (self.properties & PROPERTY_IDENTITY) == 0 {
            self.identity_fields();
        }
        self.m00 = cos;
        self.m02 = -sin;
        self.m20 = sin;
        self.m22 = cos;
        self.properties = PROPERTY_AFFINE | PROPERTY_ORTHONORMAL;
        self
    }

    pub fn rotation_z(&mut self, ang: f32) -> &mut Self {
        let sin = Math::sin_f32(ang);
        let cos = Math::cos_from_sin_f32(sin, ang);
        if (self.properties & PROPERTY_IDENTITY) == 0 {
            self.identity_fields();
        }
        self.m00 = cos;
        self.m01 = sin;
        self.m10 = -sin;
        self.m11 = cos;
        self.properties = PROPERTY_AFFINE | PROPERTY_ORTHONORMAL;
        self
    }

    pub fn rotation_towards_xy(&mut self, dir_x: f32, dir_y: f32) -> &mut Self {
        if (self.properties & PROPERTY_IDENTITY) == 0 {
            self.identity_fields();
        }
        self.m00 = dir_y;
        self.m01 = dir_x;
        self.m10 = -dir_x;
        self.m11 = dir_y;
        self.properties = PROPERTY_AFFINE | PROPERTY_ORTHONORMAL;
        self
    }

    pub fn rotation(&mut self, quat: &Quaternionf) -> &mut Self {
        let w2 = quat.w * quat.w;
        let x2 = quat.x * quat.x;
        let y2 = quat.y * quat.y;
        let z2 = quat.z * quat.z;
        let zw = quat.z * quat.w;
        let dzw = zw + zw;
        let xy = quat.x * quat.y;
        let dxy = xy + xy;
        let xz = quat.x * quat.z;
        let dxz = xz + xz;
        let yw = quat.y * quat.w;
        let dyw = yw + yw;
        let yz = quat.y * quat.z;
        let dyz = yz + yz;
        let xw = quat.x * quat.w;
        let dxw = xw + xw;
        if (self.properties & PROPERTY_IDENTITY) == 0 {
            self.identity_fields();
        }
        self.m00 = w2 + x2 - z2 - y2;
        self.m01 = dxy + dzw;
        self.m02 = dxz - dyw;
        self.m10 = -dzw + dxy;
        self.m11 = y2 - z2 + w2 - x2;
        self.m12 = dyz + dxw;
        self.m20 = dyw + dxz;
        self.m21 = dyz - dxw;
        self.m22 = z2 - y2 - x2 + w2;
        self.properties = PROPERTY_AFFINE | PROPERTY_ORTHONORMAL;
        self
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let this = *self;
        this.scale_into(x, y, z, self)
    }

    pub fn scale_vector3f(&mut self, xyz: &Vector3f) -> &mut Self {
        self.scale(xyz.x, xyz.y, xyz.z)
    }

    pub fn scale_into<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            return dest.scaling(x, y, z);
        }
        self.scale_generic_into(x, y, z, dest)
    }

    fn scale_generic_into<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let one = Math::abs_equals_one_f32(x)
            && Math::abs_equals_one_f32(y)
            && Math::abs_equals_one_f32(z);
        dest.m00 = self.m00 * x;
        dest.m01 = self.m01 * x;
        dest.m02 = self.m02 * x;
        dest.m03 = self.m03 * x;
        dest.m10 = self.m10 * y;
        dest.m11 = self.m11 * y;
        dest.m12 = self.m12 * y;
        dest.m13 = self.m13 * y;
        dest.m20 = self.m20 * z;
        dest.m21 = self.m21 * z;
        dest.m22 = self.m22 * z;
        dest.m23 = self.m23 * z;
        dest.m30 = self.m30;
        dest.m31 = self.m31;
        dest.m32 = self.m32;
        dest.m33 = self.m33;
        dest.properties = self.properties
            & !(PROPERTY_PERSPECTIVE
                | PROPERTY_IDENTITY
                | PROPERTY_TRANSLATION
                | if one { 0 } else { PROPERTY_ORTHONORMAL });
        dest
    }

    pub fn rotate_x_into<'a>(&self, ang: f32, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            dest.rotation_x(ang)
        } else if (self.properties & PROPERTY_TRANSLATION) != 0 {
            let (x, y, z) = (self.m30, self.m31, self.m32);
            dest.rotation_x(ang).set_translation(x, y, z)
        } else {
            self.rotate_x_internal(ang, dest)
        }
    }

    fn rotate_x_internal<'a>(&self, ang: f32, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        let fma = Math::fma_f32;
        let sin = Math::sin_f32(ang);
        let cos = Math::cos_from_sin_f32(sin, ang);
        let (lm10, lm11, lm12, lm13) = (self.m10, self.m11, self.m12, self.m13);
        let (lm20, lm21, lm22, lm23) = (self.m20, self.m21, self.m22, self.m23);
        dest.m20 = fma(lm10, -sin, lm20 * cos);
        dest.m21 = fma(lm11, -sin, lm21 * cos);
        dest.m22 = fma(lm12, -sin, lm22 * cos);
        dest.m23 = fma(lm13, -sin, lm23 * cos);
        dest.m10 = fma(lm10, cos, lm20 * sin);
        dest.m11 = fma(lm11, cos, lm21 * sin);
        dest.m12 = fma(lm12, cos, lm22 * sin);
        dest.m13 = fma(lm13, cos, lm23 * sin);
        dest.m00 = self.m00;
        dest.m01 = self.m01;
        dest.m02 = self.m02;
        dest.m03 = self.m03;
        dest.m30 = self.m30;
        dest.m31 = self.m31;
        dest.m32 = self.m32;
        dest.m33 = self.m33;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    pub fn rotate_x(&mut self, ang: f32) -> &mut Self {
        let this = *self;
        this.rotate_x_into(ang, self)
    }

    pub fn rotate_y_into<'a>(&self, ang: f32, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            dest.rotation_y(ang)
        } else if (self.properties & PROPERTY_TRANSLATION) != 0 {
            let (x, y, z) = (self.m30, self.m31, self.m32);
            dest.rotation_y(ang).set_translation(x, y, z)
        } else {
            self.rotate_y_internal(ang, dest)
        }
    }

    fn rotate_y_internal<'a>(&self, ang: f32, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        let fma = Math::fma_f32;
        let sin = Math::sin_f32(ang);
        let cos = Math::cos_from_sin_f32(sin, ang);
        let nm00 = fma(self.m00, cos, self.m20 * -sin);
        let nm01 = fma(self.m01, cos, self.m21 * -sin);
        let nm02 = fma(self.m02, cos, self.m22 * -sin);
        let nm03 = fma(self.m03, cos, self.m23 * -sin);
        dest.m20 = fma(self.m00, sin, self.m20 * cos);
        dest.m21 = fma(self.m01, sin, self.m21 * cos);
        dest.m22 = fma(self.m02, sin, self.m22 * cos);
        dest.m23 = fma(self.m03, sin, self.m23 * cos);
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = nm03;
        dest.m10 = self.m10;
        dest.m11 = self.m11;
        dest.m12 = self.m12;
        dest.m13 = self.m13;
        dest.m30 = self.m30;
        dest.m31 = self.m31;
        dest.m32 = self.m32;
        dest.m33 = self.m33;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    pub fn rotate_y(&mut self, ang: f32) -> &mut Self {
        let this = *self;
        this.rotate_y_into(ang, self)
    }

    pub fn rotate_z_into<'a>(&self, ang: f32, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            dest.rotation_z(ang)
        } else if (self.properties & PROPERTY_TRANSLATION) != 0 {
            let (x, y, z) = (self.m30, self.m31, self.m32);
            dest.rotation_z(ang).set_translation(x, y, z)
        } else {
            self.rotate_z_internal(ang, dest)
        }
    }

    fn rotate_z_internal<'a>(&self, ang: f32, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        let sin = Math::sin_f32(ang);
        let cos = Math::cos_from_sin_f32(sin, ang);
        self.rotate_towards_xy_into(sin, cos, dest)
    }

    pub fn rotate_z(&mut self, ang: f32) -> &mut Self {
        let this = *self;
        this.rotate_z_into(ang, self)
    }

    pub fn rotate_towards_xy_into<'a>(
        &self,
        dir_x: f32,
        dir_y: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let fma = Math::fma_f32;
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            return dest.rotation_towards_xy(dir_x, dir_y);
        }
        let nm00 = fma(self.m00, dir_y, self.m10 * dir_x);
        let nm01 = fma(self.m01, dir_y, self.m11 * dir_x);
        let nm02 = fma(self.m02, dir_y, self.m12 * dir_x);
        let nm03 = fma(self.m03, dir_y, self.m13 * dir_x);
        dest.m10 = fma(self.m00, -dir_x, self.m10 * dir_y);
        dest.m11 = fma(self.m01, -dir_x, self.m11 * dir_y);
        dest.m12 = fma(self.m02, -dir_x, self.m12 * dir_y);
        dest.m13 = fma(self.m03, -dir_x, self.m13 * dir_y);
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = nm03;
        dest.m20 = self.m20;
        dest.m21 = self.m21;
        dest.m22 = self.m22;
        dest.m23 = self.m23;
        dest.m30 = self.m30;
        dest.m31 = self.m31;
        dest.m32 = self.m32;
        dest.m33 = self.m33;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    pub fn rotate_axis_xyz_into<'a>(
        &self,
        ang: f32,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            dest.rotation_axis_xyz(ang, x, y, z)
        } else if (self.properties & PROPERTY_TRANSLATION) != 0 {
            self.rotate_translation_axis_xyz_into(ang, x, y, z, dest)
        } else if (self.properties & PROPERTY_AFFINE) != 0 {
            self.rotate_affine_axis_xyz_into(ang, x, y, z, dest)
        } else {
            self.rotate_generic_axis_xyz_into(ang, x, y, z, dest)
        }
    }

    fn rotate_generic_axis_xyz_into<'a>(
        &self,
        ang: f32,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        if y == 0.0 && z == 0.0 && Math::abs_equals_one_f32(x) {
            self.rotate_x_into(x * ang, dest)
        } else if x == 0.0 && z == 0.0 && Math::abs_equals_one_f32(y) {
            self.rotate_y_into(y * ang, dest)
        } else if x == 0.0 && y == 0.0 && Math::abs_equals_one_f32(z) {
            self.rotate_z_into(z * ang, dest)
        } else {
            self.rotate_generic_internal_axis_xyz_into(ang, x, y, z, dest)
        }
    }

    fn rotate_generic_internal_axis_xyz_into<'a>(
        &self,
        ang: f32,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let s = Math::sin_f32(ang);
        let c = Math::cos_from_sin_f32(s, ang);
        let one_minus_c = 1.0 - c;
        let (xx, xy, xz) = (x * x, x * y, x * z);
        let (yy, yz) = (y * y, y * z);
        let zz = z * z;
        let rm00 = xx * one_minus_c + c;
        let rm01 = xy * one_minus_c + z * s;
        let rm02 = xz * one_minus_c - y * s;
        let rm10 = xy * one_minus_c - z * s;
        let rm11 = yy * one_minus_c + c;
        let rm12 = yz * one_minus_c + x * s;
        let rm20 = xz * one_minus_c + y * s;
        let rm21 = yz * one_minus_c - x * s;
        let rm22 = zz * one_minus_c + c;
        let nm00 = self.m00 * rm00 + self.m10 * rm01 + self.m20 * rm02;
        let nm01 = self.m01 * rm00 + self.m11 * rm01 + self.m21 * rm02;
        let nm02 = self.m02 * rm00 + self.m12 * rm01 + self.m22 * rm02;
        let nm03 = self.m03 * rm00 + self.m13 * rm01 + self.m23 * rm02;
        let nm10 = self.m00 * rm10 + self.m10 * rm11 + self.m20 * rm12;
        let nm11 = self.m01 * rm10 + self.m11 * rm11 + self.m21 * rm12;
        let nm12 = self.m02 * rm10 + self.m12 * rm11 + self.m22 * rm12;
        let nm13 = self.m03 * rm10 + self.m13 * rm11 + self.m23 * rm12;
        dest.m20 = self.m00 * rm20 + self.m10 * rm21 + self.m20 * rm22;
        dest.m21 = self.m01 * rm20 + self.m11 * rm21 + self.m21 * rm22;
        dest.m22 = self.m02 * rm20 + self.m12 * rm21 + self.m22 * rm22;
        dest.m23 = self.m03 * rm20 + self.m13 * rm21 + self.m23 * rm22;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = nm03;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m13 = nm13;
        dest.m30 = self.m30;
        dest.m31 = self.m31;
        dest.m32 = self.m32;
        dest.m33 = self.m33;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    pub fn rotate_axis_xyz(&mut self, ang: f32, x: f32, y: f32, z: f32) -> &mut Self {
        let this = *self;
        this.rotate_axis_xyz_into(ang, x, y, z, self)
    }

    fn rotate_translation_axis_xyz_into<'a>(
        &self,
        ang: f32,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let (tx, ty, tz) = (self.m30, self.m31, self.m32);
        if y == 0.0 && z == 0.0 && Math::abs_equals_one_f32(x) {
            return dest.rotation_x(x * ang).set_translation(tx, ty, tz);
        } else if x == 0.0 && z == 0.0 && Math::abs_equals_one_f32(y) {
            return dest.rotation_y(y * ang).set_translation(tx, ty, tz);
        } else if x == 0.0 && y == 0.0 && Math::abs_equals_one_f32(z) {
            return dest.rotation_z(z * ang).set_translation(tx, ty, tz);
        }
        self.rotate_translation_internal_axis_xyz_into(ang, x, y, z, dest)
    }

    fn rotate_translation_internal_axis_xyz_into<'a>(
        &self,
        ang: f32,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let s = Math::sin_f32(ang);
        let c = Math::cos_from_sin_f32(s, ang);
        let one_minus_c = 1.0 - c;
        let (xx, xy, xz) = (x * x, x * y, x * z);
        let (yy, yz) = (y * y, y * z);
        let zz = z * z;
        let rm00 = xx * one_minus_c + c;
        let rm01 = xy * one_minus_c + z * s;
        let rm02 = xz * one_minus_c - y * s;
        let rm10 = xy * one_minus_c - z * s;
        let rm11 = yy * one_minus_c + c;
        let rm12 = yz * one_minus_c + x * s;
        let rm20 = xz * one_minus_c + y * s;
        let rm21 = yz * one_minus_c - x * s;
        let rm22 = zz * one_minus_c + c;
        dest.m20 = rm20;
        dest.m21 = rm21;
        dest.m22 = rm22;
        dest.m23 = 0.0;
        dest.m00 = rm00;
        dest.m01 = rm01;
        dest.m02 = rm02;
        dest.m03 = 0.0;
        dest.m10 = rm10;
        dest.m11 = rm11;
        dest.m12 = rm12;
        dest.m13 = 0.0;
        dest.m30 = self.m30;
        dest.m31 = self.m31;
        dest.m32 = self.m32;
        dest.m33 = 1.0;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    fn rotate_affine_axis_xyz_into<'a>(
        &self,
        ang: f32,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        if y == 0.0 && z == 0.0 && Math::abs_equals_one_f32(x) {
            self.rotate_x_into(x * ang, dest)
        } else if x == 0.0 && z == 0.0 && Math::abs_equals_one_f32(y) {
            self.rotate_y_into(y * ang, dest)
        } else if x == 0.0 && y == 0.0 && Math::abs_equals_one_f32(z) {
            self.rotate_z_into(z * ang, dest)
        } else {
            self.rotate_affine_internal_axis_xyz_into(ang, x, y, z, dest)
        }
    }

    fn rotate_affine_internal_axis_xyz_into<'a>(
        &self,
        ang: f32,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let s = Math::sin_f32(ang);
        let c = Math::cos_from_sin_f32(s, ang);
        let one_minus_c = 1.0 - c;
        let (xx, xy, xz) = (x * x, x * y, x * z);
        let (yy, yz) = (y * y, y * z);
        let zz = z * z;
        let rm00 = xx * one_minus_c + c;
        let rm01 = xy * one_minus_c + z * s;
        let rm02 = xz * one_minus_c - y * s;
        let rm10 = xy * one_minus_c - z * s;
        let rm11 = yy * one_minus_c + c;
        let rm12 = yz * one_minus_c + x * s;
        let rm20 = xz * one_minus_c + y * s;
        let rm21 = yz * one_minus_c - x * s;
        let rm22 = zz * one_minus_c + c;
        let nm00 = self.m00 * rm00 + self.m10 * rm01 + self.m20 * rm02;
        let nm01 = self.m01 * rm00 + self.m11 * rm01 + self.m21 * rm02;
        let nm02 = self.m02 * rm00 + self.m12 * rm01 + self.m22 * rm02;
        let nm10 = self.m00 * rm10 + self.m10 * rm11 + self.m20 * rm12;
        let nm11 = self.m01 * rm10 + self.m11 * rm11 + self.m21 * rm12;
        let nm12 = self.m02 * rm10 + self.m12 * rm11 + self.m22 * rm12;
        dest.m20 = self.m00 * rm20 + self.m10 * rm21 + self.m20 * rm22;
        dest.m21 = self.m01 * rm20 + self.m11 * rm21 + self.m21 * rm22;
        dest.m22 = self.m02 * rm20 + self.m12 * rm21 + self.m22 * rm22;
        dest.m23 = 0.0;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = 0.0;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m13 = 0.0;
        dest.m30 = self.m30;
        dest.m31 = self.m31;
        dest.m32 = self.m32;
        dest.m33 = 1.0;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    pub fn rotate_axis(&mut self, angle: f32, axis: &Vector3f) -> &mut Self {
        self.rotate_axis_xyz(angle, axis.x, axis.y, axis.z)
    }

    pub fn rotate_axis_into<'a>(
        &self,
        angle: f32,
        axis: &Vector3f,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        self.rotate_axis_xyz_into(angle, axis.x, axis.y, axis.z, dest)
    }

    pub fn translate_into<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            return dest.translation(x, y, z);
        }
        self.translate_generic_into(x, y, z, dest)
    }

    fn translate_generic_into<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let fma = Math::fma_f32;
        // MemUtil.INSTANCE.copy(this, dest), then the properties below.
        *dest = *self;
        dest.m30 = fma(self.m00, x, fma(self.m10, y, fma(self.m20, z, self.m30)));
        dest.m31 = fma(self.m01, x, fma(self.m11, y, fma(self.m21, z, self.m31)));
        dest.m32 = fma(self.m02, x, fma(self.m12, y, fma(self.m22, z, self.m32)));
        dest.m33 = fma(self.m03, x, fma(self.m13, y, fma(self.m23, z, self.m33)));
        dest.properties = self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY);
        dest
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            return self.translation(x, y, z);
        }
        self.translate_generic(x, y, z)
    }

    fn translate_generic(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let fma = Math::fma_f32;
        self.m30 = fma(self.m00, x, fma(self.m10, y, fma(self.m20, z, self.m30)));
        self.m31 = fma(self.m01, x, fma(self.m11, y, fma(self.m21, z, self.m31)));
        self.m32 = fma(self.m02, x, fma(self.m12, y, fma(self.m22, z, self.m32)));
        self.m33 = fma(self.m03, x, fma(self.m13, y, fma(self.m23, z, self.m33)));
        self.properties &= !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_ortho(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        z_near: f32,
        z_far: f32,
        z_zero_to_one: bool,
    ) -> &mut Self {
        if (self.properties & PROPERTY_IDENTITY) == 0 {
            self.identity_fields();
        }
        self.m00 = 2.0 / (right - left);
        self.m11 = 2.0 / (top - bottom);
        self.m22 = (if z_zero_to_one { 1.0 } else { 2.0 }) / (z_near - z_far);
        self.m30 = (right + left) / (left - right);
        self.m31 = (top + bottom) / (bottom - top);
        self.m32 = (if z_zero_to_one {
            z_near
        } else {
            z_far + z_near
        }) / (z_near - z_far);
        self.properties = PROPERTY_AFFINE;
        self
    }

    pub fn perspective_into<'a>(
        &self,
        fovy: f32,
        aspect: f32,
        z_near: f32,
        z_far: f32,
        z_zero_to_one: bool,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            return dest.set_perspective(fovy, aspect, z_near, z_far, z_zero_to_one);
        }
        self.perspective_generic_into(fovy, aspect, z_near, z_far, z_zero_to_one, dest)
    }

    fn perspective_generic_into<'a>(
        &self,
        fovy: f32,
        aspect: f32,
        z_near: f32,
        z_far: f32,
        z_zero_to_one: bool,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let h = Math::tan_f32(fovy * 0.5);
        let rm00 = 1.0 / (h * aspect);
        let rm11 = 1.0 / h;
        let far_inf = z_far > 0.0 && z_far.is_infinite();
        let near_inf = z_near > 0.0 && z_near.is_infinite();
        let (rm22, rm32) = if far_inf {
            // See: "Infinite Projection Matrix" (http://www.terathon.com/gdc07_lengyel.pdf)
            let e = 1E-6f32;
            (
                e - 1.0,
                (e - if z_zero_to_one { 1.0 } else { 2.0 }) * z_near,
            )
        } else if near_inf {
            let e = 1E-6f32;
            (
                (if z_zero_to_one { 0.0 } else { 1.0 }) - e,
                ((if z_zero_to_one { 1.0 } else { 2.0 }) - e) * z_far,
            )
        } else {
            (
                (if z_zero_to_one { z_far } else { z_far + z_near }) / (z_near - z_far),
                (if z_zero_to_one { z_far } else { z_far + z_far }) * z_near / (z_near - z_far),
            )
        };
        let nm20 = self.m20 * rm22 - self.m30;
        let nm21 = self.m21 * rm22 - self.m31;
        let nm22 = self.m22 * rm22 - self.m32;
        let nm23 = self.m23 * rm22 - self.m33;
        dest.m00 = self.m00 * rm00;
        dest.m01 = self.m01 * rm00;
        dest.m02 = self.m02 * rm00;
        dest.m03 = self.m03 * rm00;
        dest.m10 = self.m10 * rm11;
        dest.m11 = self.m11 * rm11;
        dest.m12 = self.m12 * rm11;
        dest.m13 = self.m13 * rm11;
        dest.m30 = self.m20 * rm32;
        dest.m31 = self.m21 * rm32;
        dest.m32 = self.m22 * rm32;
        dest.m33 = self.m23 * rm32;
        dest.m20 = nm20;
        dest.m21 = nm21;
        dest.m22 = nm22;
        dest.m23 = nm23;
        dest.properties = self.properties
            & !(PROPERTY_AFFINE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION | PROPERTY_ORTHONORMAL);
        dest
    }

    pub fn perspective(
        &mut self,
        fovy: f32,
        aspect: f32,
        z_near: f32,
        z_far: f32,
        z_zero_to_one: bool,
    ) -> &mut Self {
        let this = *self;
        this.perspective_into(fovy, aspect, z_near, z_far, z_zero_to_one, self)
    }

    pub fn set_perspective(
        &mut self,
        fovy: f32,
        aspect: f32,
        z_near: f32,
        z_far: f32,
        z_zero_to_one: bool,
    ) -> &mut Self {
        self.zero_fields();
        let h = Math::tan_f32(fovy * 0.5);
        self.m00 = 1.0 / (h * aspect);
        self.m11 = 1.0 / h;
        let far_inf = z_far > 0.0 && z_far.is_infinite();
        let near_inf = z_near > 0.0 && z_near.is_infinite();
        if far_inf {
            // See: "Infinite Projection Matrix" (http://www.terathon.com/gdc07_lengyel.pdf)
            let e = 1E-6f32;
            self.m22 = e - 1.0;
            self.m32 = (e - if z_zero_to_one { 1.0 } else { 2.0 }) * z_near;
        } else if near_inf {
            let e = 1E-6f32;
            self.m22 = (if z_zero_to_one { 0.0 } else { 1.0 }) - e;
            self.m32 = ((if z_zero_to_one { 1.0 } else { 2.0 }) - e) * z_far;
        } else {
            self.m22 = (if z_zero_to_one { z_far } else { z_far + z_near }) / (z_near - z_far);
            self.m32 =
                (if z_zero_to_one { z_far } else { z_far + z_far }) * z_near / (z_near - z_far);
        }
        self.m23 = -1.0;
        self.properties = PROPERTY_PERSPECTIVE;
        self
    }

    pub fn rotate_into<'a>(&self, quat: &Quaternionf, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            dest.rotation(quat)
        } else if (self.properties & PROPERTY_TRANSLATION) != 0 {
            self.rotate_translation_into(quat, dest)
        } else if (self.properties & PROPERTY_AFFINE) != 0 {
            self.rotate_affine_into(quat, dest)
        } else {
            self.rotate_generic_into(quat, dest)
        }
    }

    fn rotate_generic_into<'a>(
        &self,
        quat: &Quaternionf,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let fma = Math::fma_f32;
        let [rm00, rm01, rm02, rm10, rm11, rm12, rm20, rm21, rm22] = rotation_of(quat);
        let nm00 = fma(self.m00, rm00, fma(self.m10, rm01, self.m20 * rm02));
        let nm01 = fma(self.m01, rm00, fma(self.m11, rm01, self.m21 * rm02));
        let nm02 = fma(self.m02, rm00, fma(self.m12, rm01, self.m22 * rm02));
        let nm03 = fma(self.m03, rm00, fma(self.m13, rm01, self.m23 * rm02));
        let nm10 = fma(self.m00, rm10, fma(self.m10, rm11, self.m20 * rm12));
        let nm11 = fma(self.m01, rm10, fma(self.m11, rm11, self.m21 * rm12));
        let nm12 = fma(self.m02, rm10, fma(self.m12, rm11, self.m22 * rm12));
        let nm13 = fma(self.m03, rm10, fma(self.m13, rm11, self.m23 * rm12));
        dest.m20 = fma(self.m00, rm20, fma(self.m10, rm21, self.m20 * rm22));
        dest.m21 = fma(self.m01, rm20, fma(self.m11, rm21, self.m21 * rm22));
        dest.m22 = fma(self.m02, rm20, fma(self.m12, rm21, self.m22 * rm22));
        dest.m23 = fma(self.m03, rm20, fma(self.m13, rm21, self.m23 * rm22));
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = nm03;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m13 = nm13;
        dest.m30 = self.m30;
        dest.m31 = self.m31;
        dest.m32 = self.m32;
        dest.m33 = self.m33;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    pub fn rotate(&mut self, quat: &Quaternionf) -> &mut Self {
        let this = *self;
        this.rotate_into(quat, self)
    }

    fn rotate_affine_into<'a>(
        &self,
        quat: &Quaternionf,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let [rm00, rm01, rm02, rm10, rm11, rm12, rm20, rm21, rm22] = rotation_of(quat);
        let nm00 = self.m00 * rm00 + self.m10 * rm01 + self.m20 * rm02;
        let nm01 = self.m01 * rm00 + self.m11 * rm01 + self.m21 * rm02;
        let nm02 = self.m02 * rm00 + self.m12 * rm01 + self.m22 * rm02;
        let nm10 = self.m00 * rm10 + self.m10 * rm11 + self.m20 * rm12;
        let nm11 = self.m01 * rm10 + self.m11 * rm11 + self.m21 * rm12;
        let nm12 = self.m02 * rm10 + self.m12 * rm11 + self.m22 * rm12;
        dest.m20 = self.m00 * rm20 + self.m10 * rm21 + self.m20 * rm22;
        dest.m21 = self.m01 * rm20 + self.m11 * rm21 + self.m21 * rm22;
        dest.m22 = self.m02 * rm20 + self.m12 * rm21 + self.m22 * rm22;
        dest.m23 = 0.0;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = 0.0;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m13 = 0.0;
        dest.m30 = self.m30;
        dest.m31 = self.m31;
        dest.m32 = self.m32;
        dest.m33 = self.m33;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    fn rotate_translation_into<'a>(
        &self,
        quat: &Quaternionf,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let [rm00, rm01, rm02, rm10, rm11, rm12, rm20, rm21, rm22] = rotation_of(quat);
        dest.m20 = rm20;
        dest.m21 = rm21;
        dest.m22 = rm22;
        dest.m23 = 0.0;
        dest.m00 = rm00;
        dest.m01 = rm01;
        dest.m02 = rm02;
        dest.m03 = 0.0;
        dest.m10 = rm10;
        dest.m11 = rm11;
        dest.m12 = rm12;
        dest.m13 = 0.0;
        dest.m30 = self.m30;
        dest.m31 = self.m31;
        dest.m32 = self.m32;
        dest.m33 = self.m33;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    pub fn rotate_around(&mut self, quat: &Quaternionf, ox: f32, oy: f32, oz: f32) -> &mut Self {
        // rotateAround(quat, ox, oy, oz, this)
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            return self.rotation_around(quat, ox, oy, oz);
        }
        let this = *self;
        if (this.properties & PROPERTY_AFFINE) != 0 {
            return this.rotate_around_affine_into(quat, ox, oy, oz, self);
        }
        this.rotate_around_generic_into(quat, ox, oy, oz, self)
    }

    fn rotate_around_affine_into<'a>(
        &self,
        quat: &Quaternionf,
        ox: f32,
        oy: f32,
        oz: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let [rm00, rm01, rm02, rm10, rm11, rm12, rm20, rm21, rm22] = rotation_of(quat);
        let tm30 = self.m00 * ox + self.m10 * oy + self.m20 * oz + self.m30;
        let tm31 = self.m01 * ox + self.m11 * oy + self.m21 * oz + self.m31;
        let tm32 = self.m02 * ox + self.m12 * oy + self.m22 * oz + self.m32;
        let nm00 = self.m00 * rm00 + self.m10 * rm01 + self.m20 * rm02;
        let nm01 = self.m01 * rm00 + self.m11 * rm01 + self.m21 * rm02;
        let nm02 = self.m02 * rm00 + self.m12 * rm01 + self.m22 * rm02;
        let nm10 = self.m00 * rm10 + self.m10 * rm11 + self.m20 * rm12;
        let nm11 = self.m01 * rm10 + self.m11 * rm11 + self.m21 * rm12;
        let nm12 = self.m02 * rm10 + self.m12 * rm11 + self.m22 * rm12;
        let nm20 = self.m00 * rm20 + self.m10 * rm21 + self.m20 * rm22;
        let nm21 = self.m01 * rm20 + self.m11 * rm21 + self.m21 * rm22;
        let nm22 = self.m02 * rm20 + self.m12 * rm21 + self.m22 * rm22;
        dest.m20 = nm20;
        dest.m21 = nm21;
        dest.m22 = nm22;
        dest.m23 = 0.0;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = 0.0;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m13 = 0.0;
        dest.m30 = -nm00 * ox - nm10 * oy - nm20 * oz + tm30;
        dest.m31 = -nm01 * ox - nm11 * oy - nm21 * oz + tm31;
        dest.m32 = -nm02 * ox - nm12 * oy - nm22 * oz + tm32;
        dest.m33 = 1.0;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    /// Returns `self`, not `dest`, when `self` is the identity: JOML then sets `this` to the
    /// rotation and leaves `dest` alone.
    pub fn rotate_around_into<'a>(
        &'a mut self,
        quat: &Quaternionf,
        ox: f32,
        oy: f32,
        oz: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        if (self.properties & PROPERTY_IDENTITY) != 0 {
            self.rotation_around(quat, ox, oy, oz)
        } else if (self.properties & PROPERTY_AFFINE) != 0 {
            self.rotate_around_affine_into(quat, ox, oy, oz, dest)
        } else {
            self.rotate_around_generic_into(quat, ox, oy, oz, dest)
        }
    }

    fn rotate_around_generic_into<'a>(
        &self,
        quat: &Quaternionf,
        ox: f32,
        oy: f32,
        oz: f32,
        dest: &'a mut Matrix4f,
    ) -> &'a mut Matrix4f {
        let [rm00, rm01, rm02, rm10, rm11, rm12, rm20, rm21, rm22] = rotation_of(quat);
        let tm30 = self.m00 * ox + self.m10 * oy + self.m20 * oz + self.m30;
        let tm31 = self.m01 * ox + self.m11 * oy + self.m21 * oz + self.m31;
        let tm32 = self.m02 * ox + self.m12 * oy + self.m22 * oz + self.m32;
        let nm00 = self.m00 * rm00 + self.m10 * rm01 + self.m20 * rm02;
        let nm01 = self.m01 * rm00 + self.m11 * rm01 + self.m21 * rm02;
        let nm02 = self.m02 * rm00 + self.m12 * rm01 + self.m22 * rm02;
        let nm03 = self.m03 * rm00 + self.m13 * rm01 + self.m23 * rm02;
        let nm10 = self.m00 * rm10 + self.m10 * rm11 + self.m20 * rm12;
        let nm11 = self.m01 * rm10 + self.m11 * rm11 + self.m21 * rm12;
        let nm12 = self.m02 * rm10 + self.m12 * rm11 + self.m22 * rm12;
        let nm13 = self.m03 * rm10 + self.m13 * rm11 + self.m23 * rm12;
        let nm20 = self.m00 * rm20 + self.m10 * rm21 + self.m20 * rm22;
        let nm21 = self.m01 * rm20 + self.m11 * rm21 + self.m21 * rm22;
        let nm22 = self.m02 * rm20 + self.m12 * rm21 + self.m22 * rm22;
        let nm23 = self.m03 * rm20 + self.m13 * rm21 + self.m23 * rm22;
        dest.m20 = nm20;
        dest.m21 = nm21;
        dest.m22 = nm22;
        dest.m23 = nm23;
        dest.m00 = nm00;
        dest.m01 = nm01;
        dest.m02 = nm02;
        dest.m03 = nm03;
        dest.m10 = nm10;
        dest.m11 = nm11;
        dest.m12 = nm12;
        dest.m13 = nm13;
        dest.m30 = -nm00 * ox - nm10 * oy - nm20 * oz + tm30;
        dest.m31 = -nm01 * ox - nm11 * oy - nm21 * oz + tm31;
        dest.m32 = -nm02 * ox - nm12 * oy - nm22 * oz + tm32;
        dest.m33 = self.m33;
        dest.properties =
            self.properties & !(PROPERTY_PERSPECTIVE | PROPERTY_IDENTITY | PROPERTY_TRANSLATION);
        dest
    }

    /// Reads back the rotation it just wrote to compute the translation.
    pub fn rotation_around(&mut self, quat: &Quaternionf, ox: f32, oy: f32, oz: f32) -> &mut Self {
        let [rm00, rm01, rm02, rm10, rm11, rm12, rm20, rm21, rm22] = rotation_of(quat);
        self.m20 = rm20;
        self.m21 = rm21;
        self.m22 = rm22;
        self.m23 = 0.0;
        self.m00 = rm00;
        self.m01 = rm01;
        self.m02 = rm02;
        self.m03 = 0.0;
        self.m10 = rm10;
        self.m11 = rm11;
        self.m12 = rm12;
        self.m13 = 0.0;
        self.m30 = -self.m00 * ox - self.m10 * oy - self.m20 * oz + ox;
        self.m31 = -self.m01 * ox - self.m11 * oy - self.m21 * oz + oy;
        self.m32 = -self.m02 * ox - self.m12 * oy - self.m22 * oz + oz;
        self.m33 = 1.0;
        self.properties = PROPERTY_AFFINE | PROPERTY_ORTHONORMAL;
        self
    }

    pub fn transform_project<'a>(&self, v: &'a mut Vector3f) -> &'a mut Vector3f {
        v.mul_project(self)
    }

    pub fn transform_project_into<'a>(
        &self,
        v: &Vector3f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        v.mul_project_into(self, dest)
    }

    pub fn transform_project_xyz_into<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        dest.set_xyz(x, y, z).mul_project(self)
    }

    pub fn transform_position<'a>(&self, v: &'a mut Vector3f) -> &'a mut Vector3f {
        v.mul_position(self)
    }

    pub fn transform_position_into<'a>(
        &self,
        v: &Vector3f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        self.transform_position_xyz_into(v.x, v.y, v.z, dest)
    }

    pub fn transform_position_xyz_into<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        dest.set_xyz(x, y, z).mul_position(self)
    }

    pub fn transform_direction<'a>(&self, v: &'a mut Vector3f) -> &'a mut Vector3f {
        let (x, y, z) = (v.x, v.y, v.z);
        self.transform_direction_xyz_into(x, y, z, v)
    }

    pub fn transform_direction_into<'a>(
        &self,
        v: &Vector3f,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        self.transform_direction_xyz_into(v.x, v.y, v.z, dest)
    }

    pub fn transform_direction_xyz_into<'a>(
        &self,
        x: f32,
        y: f32,
        z: f32,
        dest: &'a mut Vector3f,
    ) -> &'a mut Vector3f {
        dest.set_xyz(x, y, z).mul_direction(self)
    }

    pub fn transform<'a>(&self, v: &'a mut Vector4f) -> &'a mut Vector4f {
        v.mul_matrix4f(self)
    }

    pub fn transform_into<'a>(&self, v: &Vector4f, dest: &'a mut Vector4f) -> &'a mut Vector4f {
        v.mul_matrix4f_into(self, dest)
    }

    pub fn transform_transpose<'a>(&self, v: &'a mut Vector4f) -> &'a mut Vector4f {
        v.mul_transpose(self)
    }

    pub fn transform_transpose_into<'a>(
        &self,
        v: &Vector4f,
        dest: &'a mut Vector4f,
    ) -> &'a mut Vector4f {
        v.mul_transpose_into(self, dest)
    }
}

// The rotation matrix of a quaternion, [rm00, rm01, rm02, rm10, … rm22], computed the way each of
// JOML's rotate(Quaternionfc) variants spells it out.
fn rotation_of(quat: &Quaternionf) -> [f32; 9] {
    let (w2, x2) = (quat.w * quat.w, quat.x * quat.x);
    let (y2, z2) = (quat.y * quat.y, quat.z * quat.z);
    let zw = quat.z * quat.w;
    let dzw = zw + zw;
    let xy = quat.x * quat.y;
    let dxy = xy + xy;
    let xz = quat.x * quat.z;
    let dxz = xz + xz;
    let yw = quat.y * quat.w;
    let dyw = yw + yw;
    let yz = quat.y * quat.z;
    let dyz = yz + yz;
    let xw = quat.x * quat.w;
    let dxw = xw + xw;
    [
        w2 + x2 - z2 - y2,
        dxy + dzw,
        dxz - dyw,
        -dzw + dxy,
        y2 - z2 + w2 - x2,
        dyz + dxw,
        dyw + dxz,
        dyz - dxw,
        z2 - y2 - x2 + w2,
    ]
}

#[cfg(test)]
mod tests {
    use super::Matrix4f;
    use crate::golden::{Golden, encode_fields};
    use crate::org::joml::matrix3x2f::Matrix3x2f;
    use crate::org::joml::quaternionf::Quaternionf;
    use crate::org::joml::vector3f::Vector3f;
    use crate::org::joml::vector4f::Vector4f;

    encode_fields!(Matrix4f {
        m00: f32,
        m01: f32,
        m02: f32,
        m03: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m20: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m30: f32,
        m31: f32,
        m32: f32,
        m33: f32,
        properties: i32,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Matrix4f");
        golden.check("<init>()V", Matrix4f::new);
        golden.check("<init>(FFFFFFFFFFFFFFFF)V", Matrix4f::from_values);
        golden.check("<init>(Lorg/joml/Matrix4fc;)V", |mat: Matrix4f| {
            Matrix4f::from_matrix4f(&mat)
        });
        golden.check_method("properties()I", |this: &mut Matrix4f| this.properties());
        golden.check_method(
            "assume(I)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, properties| *this.assume(properties),
        );
        golden.check_method(
            "determineProperties()Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f| *this.determine_properties(),
        );
        golden.check_method("identity()Lorg/joml/Matrix4f;", |this: &mut Matrix4f| {
            *this.identity()
        });
        golden.check_method(
            "set(Lorg/joml/Matrix4fc;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, m: Matrix4f| *this.set(&m),
        );
        golden.check_method(
            "set(FFFFFFFFFFFFFFFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f,
             m00,
             m01,
             m02,
             m03,
             m10,
             m11,
             m12,
             m13,
             m20,
             m21,
             m22,
             m23,
             m30,
             m31,
             m32,
             m33| {
                *this.set_values(
                    m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33,
                )
            },
        );
        golden.check_method(
            "mul(Lorg/joml/Matrix4fc;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, right: Matrix4f| *this.mul(&right),
        );
        golden.check_method(
            "mul(Lorg/joml/Matrix4fc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, right: Matrix4f, mut dest: Matrix4f| {
                *this.mul_into(&right, &mut dest)
            },
        );
        golden.check_method(
            "mul(Lorg/joml/Matrix3x2fc;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, right: Matrix3x2f| *this.mul_matrix3x2f(&right),
        );
        golden.check_method(
            "mul(Lorg/joml/Matrix3x2fc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, right: Matrix3x2f, mut dest: Matrix4f| {
                *this.mul_matrix3x2f_into(&right, &mut dest)
            },
        );
        golden.check_method(
            "translation(FFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, x, y, z| *this.translation(x, y, z),
        );
        golden.check_method(
            "setTranslation(FFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, x, y, z| *this.set_translation(x, y, z),
        );
        golden.check_method(
            "translate(FFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, x, y, z| *this.translate(x, y, z),
        );
        golden.check_method(
            "translate(FFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, x, y, z, mut dest: Matrix4f| {
                *this.translate_into(x, y, z, &mut dest)
            },
        );
        golden.check_method(
            "scaling(FFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, x, y, z| *this.scaling(x, y, z),
        );
        golden.check_method(
            "scale(FFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, x, y, z| *this.scale(x, y, z),
        );
        golden.check_method(
            "scale(FFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, x, y, z, mut dest: Matrix4f| *this.scale_into(x, y, z, &mut dest),
        );
        golden.check_method(
            "rotationX(F)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang| *this.rotation_x(ang),
        );
        golden.check_method(
            "rotationY(F)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang| *this.rotation_y(ang),
        );
        golden.check_method(
            "rotationZ(F)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang| *this.rotation_z(ang),
        );
        golden.check_method(
            "rotateX(F)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang| *this.rotate_x(ang),
        );
        golden.check_method(
            "rotateX(FLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang, mut dest: Matrix4f| *this.rotate_x_into(ang, &mut dest),
        );
        golden.check_method(
            "rotateY(F)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang| *this.rotate_y(ang),
        );
        golden.check_method(
            "rotateY(FLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang, mut dest: Matrix4f| *this.rotate_y_into(ang, &mut dest),
        );
        golden.check_method(
            "rotateZ(F)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang| *this.rotate_z(ang),
        );
        golden.check_method(
            "rotateZ(FLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang, mut dest: Matrix4f| *this.rotate_z_into(ang, &mut dest),
        );
        golden.check_method(
            "rotationTowardsXY(FF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, dir_x, dir_y| *this.rotation_towards_xy(dir_x, dir_y),
        );
        golden.check_method(
            "rotateTowardsXY(FFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, dir_x, dir_y, mut dest: Matrix4f| {
                *this.rotate_towards_xy_into(dir_x, dir_y, &mut dest)
            },
        );
        golden.check_method(
            "rotation(FFFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, angle, x, y, z| *this.rotation_axis_xyz(angle, x, y, z),
        );
        golden.check_method(
            "rotation(FLorg/joml/Vector3fc;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, angle, axis: Vector3f| *this.rotation_axis(angle, &axis),
        );
        golden.check_method(
            "rotate(FFFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang, x, y, z| *this.rotate_axis_xyz(ang, x, y, z),
        );
        golden.check_method(
            "rotate(FFFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, ang, x, y, z, mut dest: Matrix4f| {
                *this.rotate_axis_xyz_into(ang, x, y, z, &mut dest)
            },
        );
        golden.check_method(
            "rotate(FLorg/joml/Vector3fc;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, angle, axis: Vector3f| *this.rotate_axis(angle, &axis),
        );
        golden.check_method(
            "rotate(FLorg/joml/Vector3fc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, angle, axis: Vector3f, mut dest: Matrix4f| {
                *this.rotate_axis_into(angle, &axis, &mut dest)
            },
        );
        golden.check_method(
            "rotation(Lorg/joml/Quaternionfc;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, quat: Quaternionf| *this.rotation(&quat),
        );
        golden.check_method(
            "rotate(Lorg/joml/Quaternionfc;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, quat: Quaternionf| *this.rotate(&quat),
        );
        golden.check_method(
            "rotate(Lorg/joml/Quaternionfc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, quat: Quaternionf, mut dest: Matrix4f| {
                *this.rotate_into(&quat, &mut dest)
            },
        );
        golden.check_method(
            "rotationAround(Lorg/joml/Quaternionfc;FFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, quat: Quaternionf, ox, oy, oz| {
                *this.rotation_around(&quat, ox, oy, oz)
            },
        );
        golden.check_method(
            "rotateAround(Lorg/joml/Quaternionfc;FFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, quat: Quaternionf, ox, oy, oz| {
                *this.rotate_around(&quat, ox, oy, oz)
            },
        );
        golden.check_method(
            "rotateAround(Lorg/joml/Quaternionfc;FFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, quat: Quaternionf, ox, oy, oz, mut dest: Matrix4f| {
                *this.rotate_around_into(&quat, ox, oy, oz, &mut dest)
            },
        );
        golden.check_method("invert()Lorg/joml/Matrix4f;", |this: &mut Matrix4f| {
            *this.invert()
        });
        golden.check_method(
            "invert(Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, mut dest: Matrix4f| *this.invert_into(&mut dest),
        );
        golden.check_method(
            "setPerspective(FFFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, fovy, aspect, z_near, z_far| {
                *this.set_perspective(fovy, aspect, z_near, z_far, false)
            },
        );
        golden.check_method(
            "setPerspective(FFFFZ)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, fovy, aspect, z_near, z_far, z_zero_to_one| {
                *this.set_perspective(fovy, aspect, z_near, z_far, z_zero_to_one)
            },
        );
        golden.check_method(
            "perspective(FFFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, fovy, aspect, z_near, z_far| {
                *this.perspective(fovy, aspect, z_near, z_far, false)
            },
        );
        golden.check_method(
            "perspective(FFFFZ)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, fovy, aspect, z_near, z_far, z_zero_to_one| {
                *this.perspective(fovy, aspect, z_near, z_far, z_zero_to_one)
            },
        );
        golden.check_method(
            "perspective(FFFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, fovy, aspect, z_near, z_far, mut dest: Matrix4f| {
                *this.perspective_into(fovy, aspect, z_near, z_far, false, &mut dest)
            },
        );
        golden.check_method(
            "perspective(FFFFZLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f,
             fovy,
             aspect,
             z_near,
             z_far,
             z_zero_to_one,
             mut dest: Matrix4f| {
                *this.perspective_into(fovy, aspect, z_near, z_far, z_zero_to_one, &mut dest)
            },
        );
        golden.check_method(
            "setOrtho(FFFFFF)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, left, right, bottom, top, z_near, z_far| {
                *this.set_ortho(left, right, bottom, top, z_near, z_far, false)
            },
        );
        golden.check_method(
            "setOrtho(FFFFFFZ)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, left, right, bottom, top, z_near, z_far, z_zero_to_one| {
                *this.set_ortho(left, right, bottom, top, z_near, z_far, z_zero_to_one)
            },
        );
        golden.check_method(
            "transformProject(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix4f, mut v: Vector3f| *this.transform_project(&mut v),
        );
        golden.check_method(
            "transformProject(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix4f, v: Vector3f, mut dest: Vector3f| {
                *this.transform_project_into(&v, &mut dest)
            },
        );
        golden.check_method(
            "transformProject(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix4f, x, y, z, mut dest: Vector3f| {
                *this.transform_project_xyz_into(x, y, z, &mut dest)
            },
        );
        golden.check_method(
            "transformPosition(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix4f, mut v: Vector3f| *this.transform_position(&mut v),
        );
        golden.check_method(
            "transformPosition(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix4f, v: Vector3f, mut dest: Vector3f| {
                *this.transform_position_into(&v, &mut dest)
            },
        );
        golden.check_method(
            "transformPosition(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix4f, x, y, z, mut dest: Vector3f| {
                *this.transform_position_xyz_into(x, y, z, &mut dest)
            },
        );
        golden.check_method(
            "transformDirection(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix4f, mut v: Vector3f| *this.transform_direction(&mut v),
        );
        golden.check_method(
            "transformDirection(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix4f, v: Vector3f, mut dest: Vector3f| {
                *this.transform_direction_into(&v, &mut dest)
            },
        );
        golden.check_method(
            "transformDirection(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix4f, x, y, z, mut dest: Vector3f| {
                *this.transform_direction_xyz_into(x, y, z, &mut dest)
            },
        );
        golden.check_method(
            "transform(Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            |this: &mut Matrix4f, mut v: Vector4f| *this.transform(&mut v),
        );
        golden.check_method(
            "transform(Lorg/joml/Vector4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            |this: &mut Matrix4f, v: Vector4f, mut dest: Vector4f| {
                *this.transform_into(&v, &mut dest)
            },
        );
        golden.check_method(
            "invertAffine()Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f| *this.invert_affine(),
        );
        golden.check_method(
            "invertAffine(Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, mut dest: Matrix4f| *this.invert_affine_into(&mut dest),
        );
        golden.check_method("isFinite()Z", |this: &mut Matrix4f| this.is_finite());
        golden.check_method(
            "translation(Lorg/joml/Vector3fc;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, offset: Vector3f| *this.translation_vector3f(&offset),
        );
        golden.check_method(
            "scale(Lorg/joml/Vector3fc;)Lorg/joml/Matrix4f;",
            |this: &mut Matrix4f, xyz: Vector3f| *this.scale_vector3f(&xyz),
        );
        golden.check_method(
            "getTranslation(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            |this: &mut Matrix4f, mut dest: Vector3f| *this.get_translation(&mut dest),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Matrix4f, obj: Matrix4f| *this == obj,
        );
        golden.check_method("hashCode()I", |this: &mut Matrix4f| this.hash_code());
        golden.check_method(
            "transformTranspose(Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            |this: &mut Matrix4f, mut v: Vector4f| *this.transform_transpose(&mut v),
        );
        golden.check_method(
            "transformTranspose(Lorg/joml/Vector4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            |this: &mut Matrix4f, v: Vector4f, mut dest: Vector4f| {
                *this.transform_transpose_into(&v, &mut dest)
            },
        );
        golden.finish();
    }
}
