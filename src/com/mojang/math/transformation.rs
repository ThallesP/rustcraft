//! com/mojang/math/Transformation.java
//!
//! Its codecs (`CODEC`, `EXTENDED_CODEC`) wait for DataFixerUpper.
//!
//! Java holds the matrix and parts it's given by reference, and nothing changes them afterwards,
//! so the port keeps copies. Cloning a `Transformation` stands for copying a Java reference to it:
//! its only state that changes is the lazy decomposition, which comes out the same whenever it
//! runs, and the properties `MatrixUtil.checkProperty` adds to its matrix (`get_matrix_mut`).

use std::sync::{LazyLock, OnceLock};

use crate::com::mojang::math::matrix_util::MatrixUtil;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

const ZERO_TRANSLATION: Vector3f = Vector3f::new();
const UNIT_SCALE: Vector3f = Vector3f::from_xyz(1.0, 1.0, 1.0);
const ZERO_ROTATION: Quaternionf = Quaternionf::new();

pub static IDENTITY: LazyLock<Transformation> = LazyLock::new(|| Transformation {
    matrix: Matrix4f::new(),
    decomposition: OnceLock::from(Decomposition {
        translation: ZERO_TRANSLATION,
        left_rotation: ZERO_ROTATION,
        scale: UNIT_SCALE,
        right_rotation: ZERO_ROTATION,
    }),
    identity: true,
});

/// An affine transformation, and what it's made of: a translation, then a rotation, a scale and
/// another rotation. One built from a matrix works out its parts when first asked.
#[derive(Clone, Debug)]
pub struct Transformation {
    matrix: Matrix4f,
    /// Java's `decomposed` with the four parts it guards.
    decomposition: OnceLock<Decomposition>,
    /// Whether this is `IDENTITY`, which Java compares with by reference (`FaceBakery` skips the
    /// rotation when it's `IDENTITY`, and `inverse` returns it as it is). Its clones keep it.
    identity: bool,
}

#[derive(Clone, Copy, Debug)]
struct Decomposition {
    translation: Vector3f,
    left_rotation: Quaternionf,
    scale: Vector3f,
    right_rotation: Quaternionf,
}

/// Java's `equals`: only the matrices count, compared like JOML's `Matrix4f.equals`.
impl PartialEq for Transformation {
    fn eq(&self, other: &Transformation) -> bool {
        self.matrix == other.matrix
    }
}

impl Eq for Transformation {}

impl Transformation {
    pub fn new(matrix: Matrix4f) -> Transformation {
        Transformation {
            matrix,
            decomposition: OnceLock::new(),
            identity: false,
        }
    }

    /// A missing part means none: no translation, no rotation, a scale of 1.
    pub fn from_components(
        translation: Option<&Vector3f>,
        left_rotation: Option<&Quaternionf>,
        scale: Option<&Vector3f>,
        right_rotation: Option<&Quaternionf>,
    ) -> Transformation {
        Transformation {
            matrix: Transformation::compose_components(
                translation,
                left_rotation,
                scale,
                right_rotation,
            ),
            decomposition: OnceLock::from(Decomposition {
                translation: *translation.unwrap_or(&ZERO_TRANSLATION),
                left_rotation: *left_rotation.unwrap_or(&ZERO_ROTATION),
                scale: *scale.unwrap_or(&UNIT_SCALE),
                right_rotation: *right_rotation.unwrap_or(&ZERO_ROTATION),
            }),
            identity: false,
        }
    }

    pub fn compose(&self, that: &Transformation) -> Transformation {
        let mut matrix = self.get_matrix_copy();
        matrix.mul(that.get_matrix());
        Transformation::new(matrix)
    }

    /// None when the inverse isn't finite.
    pub fn inverse(&self) -> Option<Transformation> {
        if self.identity {
            return Some(self.clone());
        }

        let mut matrix = self.get_matrix_copy();
        matrix.invert_affine();
        matrix.is_finite().then(|| Transformation::new(matrix))
    }

    // ensureDecomposed.
    fn decomposition(&self) -> &Decomposition {
        self.decomposition.get_or_init(|| {
            let mut translation = Vector3f::new();
            let mut left_rotation = Quaternionf::new();
            let mut scale = Vector3f::new();
            let mut right_rotation = Quaternionf::new();
            MatrixUtil::svd_decompose(
                &self.matrix,
                &mut translation,
                &mut left_rotation,
                &mut scale,
                &mut right_rotation,
            );
            Decomposition {
                translation,
                left_rotation,
                scale,
                right_rotation,
            }
        })
    }

    fn compose_components(
        translation: Option<&Vector3f>,
        left_rotation: Option<&Quaternionf>,
        scale: Option<&Vector3f>,
        right_rotation: Option<&Quaternionf>,
    ) -> Matrix4f {
        let mut result = Matrix4f::new();
        if let Some(translation) = translation {
            result.translation_vector3f(translation);
        }

        if let Some(left_rotation) = left_rotation {
            result.rotate(left_rotation);
        }

        if let Some(scale) = scale {
            result.scale_vector3f(scale);
        }

        if let Some(right_rotation) = right_rotation {
            result.rotate(right_rotation);
        }

        result
    }

    pub fn get_matrix(&self) -> &Matrix4f {
        &self.matrix
    }

    /// `getMatrix()` for callers that hand it to `MatrixUtil.checkProperty`, which adds the
    /// properties it works out to the matrix (`PoseStack.mulPose`, `BlockMath`). Java gives out its
    /// own matrix as a `Matrix4fc`, and the check casts it back to a `Matrix4f`.
    pub fn get_matrix_mut(&mut self) -> &mut Matrix4f {
        &mut self.matrix
    }

    pub fn get_matrix_copy(&self) -> Matrix4f {
        Matrix4f::from_matrix4f(&self.matrix)
    }

    pub fn translation(&self) -> &Vector3f {
        &self.decomposition().translation
    }

    pub fn left_rotation(&self) -> &Quaternionf {
        &self.decomposition().left_rotation
    }

    pub fn scale(&self) -> &Vector3f {
        &self.decomposition().scale
    }

    pub fn right_rotation(&self) -> &Quaternionf {
        &self.decomposition().right_rotation
    }

    /// `Objects.hash(matrix)`.
    pub fn hash_code(&self) -> i32 {
        31i32.wrapping_add(self.matrix.hash_code())
    }

    /// Interpolates each part, decomposing both transformations.
    pub fn slerp(&self, that: &Transformation, progress: f32) -> Transformation {
        Transformation::from_components(
            Some(
                self.translation()
                    .lerp_into(that.translation(), progress, &mut Vector3f::new()),
            ),
            Some(self.left_rotation().slerp_into(
                that.left_rotation(),
                progress,
                &mut Quaternionf::new(),
            )),
            Some(
                self.scale()
                    .lerp_into(that.scale(), progress, &mut Vector3f::new()),
            ),
            Some(self.right_rotation().slerp_into(
                that.right_rotation(),
                progress,
                &mut Quaternionf::new(),
            )),
        )
    }

    /// The static `compose(Matrix4fc, Optional<Transformation>)`. Java returns `parent` itself
    /// when there's no transformation; it's read-only there, so a copy does.
    pub fn compose_matrix4f(parent: &Matrix4f, transform: Option<&Transformation>) -> Matrix4f {
        match transform {
            Some(transform) => *parent.mul_into(transform.get_matrix(), &mut Matrix4f::new()),
            None => *parent,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use serde_json::Value;

    use super::{Decomposition, IDENTITY, Transformation};
    use crate::golden::{Decode, Encode, Golden, elements};
    use crate::org::joml::matrix4f::Matrix4f;
    use crate::org::joml::quaternionf::Quaternionf;
    use crate::org::joml::vector3f::Vector3f;

    /// Vanilla encodes `[matrix, decomposed, translation, leftRotation, scale, rightRotation]`,
    /// the parts null until decomposed, then whether it's `IDENTITY`.
    type Fields = (
        Matrix4f,
        bool,
        Option<Vector3f>,
        Option<Quaternionf>,
        Option<Vector3f>,
        Option<Quaternionf>,
        bool,
    );

    impl Transformation {
        fn fields(&self) -> Fields {
            let parts = self.decomposition.get();
            (
                self.matrix,
                parts.is_some(),
                parts.map(|parts| parts.translation),
                parts.map(|parts| parts.left_rotation),
                parts.map(|parts| parts.scale),
                parts.map(|parts| parts.right_rotation),
                self.identity,
            )
        }
    }

    impl Decode for Transformation {
        fn decode(value: &Value) -> Self {
            let [
                matrix,
                decomposed,
                translation,
                left_rotation,
                scale,
                right_rotation,
                identity,
            ] = elements(value);
            let decomposition = OnceLock::new();
            if bool::decode(decomposed) {
                decomposition
                    .set(Decomposition {
                        translation: Vector3f::decode(translation),
                        left_rotation: Quaternionf::decode(left_rotation),
                        scale: Vector3f::decode(scale),
                        right_rotation: Quaternionf::decode(right_rotation),
                    })
                    .expect("a fresh OnceLock is empty");
            }
            Transformation {
                matrix: Matrix4f::decode(matrix),
                decomposition,
                identity: bool::decode(identity),
            }
        }
    }

    impl Encode for Transformation {
        fn encode(&self) -> Value {
            self.fields().encode()
        }

        fn matches(&self, expected: &Value) -> bool {
            self.fields().matches(expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.math.Transformation");
        golden.field("IDENTITY", IDENTITY.clone());
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.pending("EXTENDED_CODEC", "waits for DataFixerUpper's codecs");
        golden.check("<init>(Lorg/joml/Matrix4fc;)V", Transformation::new);
        golden.check(
            "<init>(Lorg/joml/Vector3fc;Lorg/joml/Quaternionfc;Lorg/joml/Vector3fc;Lorg/joml/Quaternionfc;)V",
            |translation: Option<Vector3f>,
             left_rotation: Option<Quaternionf>,
             scale: Option<Vector3f>,
             right_rotation: Option<Quaternionf>| {
                Transformation::from_components(
                    translation.as_ref(),
                    left_rotation.as_ref(),
                    scale.as_ref(),
                    right_rotation.as_ref(),
                )
            },
        );
        golden.check_method(
            "compose(Lcom/mojang/math/Transformation;)Lcom/mojang/math/Transformation;",
            |this: &mut Transformation, that: Transformation| this.compose(&that),
        );
        golden.check_method(
            "inverse()Lcom/mojang/math/Transformation;",
            |this: &mut Transformation| this.inverse(),
        );
        golden.check_method(
            "getMatrix()Lorg/joml/Matrix4fc;",
            |this: &mut Transformation| *this.get_matrix(),
        );
        golden.check_method(
            "getMatrixCopy()Lorg/joml/Matrix4f;",
            |this: &mut Transformation| this.get_matrix_copy(),
        );
        golden.check_method(
            "translation()Lorg/joml/Vector3fc;",
            |this: &mut Transformation| *this.translation(),
        );
        golden.check_method(
            "leftRotation()Lorg/joml/Quaternionfc;",
            |this: &mut Transformation| *this.left_rotation(),
        );
        golden.check_method(
            "scale()Lorg/joml/Vector3fc;",
            |this: &mut Transformation| *this.scale(),
        );
        golden.check_method(
            "rightRotation()Lorg/joml/Quaternionfc;",
            |this: &mut Transformation| *this.right_rotation(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Transformation, o: Transformation| *this == o,
        );
        golden.check_method("hashCode()I", |this: &mut Transformation| this.hash_code());
        golden.check_mut(
            "slerp(Lcom/mojang/math/Transformation;F)Lcom/mojang/math/Transformation;",
            |this: &mut Transformation, that: &mut Transformation, progress: &mut f32| {
                this.slerp(that, *progress)
            },
        );
        golden.check(
            "compose(Lorg/joml/Matrix4fc;Ljava/util/Optional;)Lorg/joml/Matrix4fc;",
            |parent: Matrix4f, transform: Option<Transformation>| {
                Transformation::compose_matrix4f(&parent, transform.as_ref())
            },
        );
        golden.finish();
    }
}
