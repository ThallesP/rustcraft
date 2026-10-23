//! net/minecraft/core/BlockMath.java
//!
//! Java's `getFaceTransformation` asks `MatrixUtil.isIdentity` about the transformation's own
//! matrix, which adds the properties it works out to that matrix, so the port takes the
//! transformation by `&mut`, and hands back a clone of it where Java hands back the object itself.

use std::sync::LazyLock;

use crate::com::mojang::math::matrix_util::MatrixUtil;
use crate::com::mojang::math::transformation::{self, Transformation};
use crate::net::minecraft::core::direction::Direction;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

/// `VANILLA_UV_TRANSFORM_LOCAL_TO_GLOBAL`, by `Direction`'s ordinal.
static VANILLA_UV_TRANSFORM_LOCAL_TO_GLOBAL: LazyLock<[Transformation; 6]> = LazyLock::new(|| {
    let rotated =
        |rotation: &Quaternionf| Transformation::from_components(None, Some(rotation), None, None);
    Direction::VALUES.map(|direction| match direction {
        Direction::SOUTH => transformation::IDENTITY.clone(),
        Direction::EAST => rotated(Quaternionf::new().rotate_y(std::f64::consts::FRAC_PI_2 as f32)),
        Direction::WEST => {
            rotated(Quaternionf::new().rotate_y(-std::f64::consts::FRAC_PI_2 as f32))
        }
        Direction::NORTH => rotated(Quaternionf::new().rotate_y(std::f64::consts::PI as f32)),
        Direction::UP => rotated(Quaternionf::new().rotate_x(-std::f64::consts::FRAC_PI_2 as f32)),
        Direction::DOWN => rotated(Quaternionf::new().rotate_x(std::f64::consts::FRAC_PI_2 as f32)),
    })
});

/// `VANILLA_UV_TRANSFORM_GLOBAL_TO_LOCAL`: each of those inverted.
static VANILLA_UV_TRANSFORM_GLOBAL_TO_LOCAL: LazyLock<[Transformation; 6]> = LazyLock::new(|| {
    VANILLA_UV_TRANSFORM_LOCAL_TO_GLOBAL
        .each_ref()
        .map(|transformation| {
            transformation
                .inverse()
                .expect("a quarter turn has an inverse")
        })
});

pub struct BlockMath;

impl BlockMath {
    pub fn block_center_to_corner(transform: &Transformation) -> Transformation {
        let mut ret = Matrix4f::new();
        ret.translation(0.5, 0.5, 0.5);
        ret.mul(transform.get_matrix());
        ret.translate(-0.5, -0.5, -0.5);
        Transformation::new(ret)
    }

    pub fn block_corner_to_center(transform: &Transformation) -> Transformation {
        let mut ret = Matrix4f::new();
        ret.translation(-0.5, -0.5, -0.5);
        ret.mul(transform.get_matrix());
        ret.translate(0.5, 0.5, 0.5);
        Transformation::new(ret)
    }

    /// How a face's texture turns once the model is transformed: the face's own frame turned
    /// by the transformation, then back into the frame of the side it ends up facing.
    pub fn get_face_transformation(
        transformation: &mut Transformation,
        original_side: Direction,
    ) -> Transformation {
        if MatrixUtil::is_identity(transformation.get_matrix_mut()) {
            return transformation.clone();
        }

        let face_action = &VANILLA_UV_TRANSFORM_LOCAL_TO_GLOBAL[original_side as usize];
        let face_action = transformation.compose(face_action);
        let mut transformed_normal = Vector3f::from_xyz(0.0, 0.0, 1.0);
        face_action
            .get_matrix()
            .transform_direction(&mut transformed_normal);
        let new_side = Direction::get_approximate_nearest_f32(
            transformed_normal.x,
            transformed_normal.y,
            transformed_normal.z,
        );
        VANILLA_UV_TRANSFORM_GLOBAL_TO_LOCAL[new_side as usize].compose(&face_action)
    }
}

#[cfg(test)]
mod tests {
    use super::BlockMath;
    use crate::com::mojang::math::transformation::Transformation;
    use crate::golden::Golden;
    use crate::net::minecraft::core::direction::Direction;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.BlockMath");
        golden.check(
            "blockCenterToCorner(Lcom/mojang/math/Transformation;)Lcom/mojang/math/Transformation;",
            |transform: Transformation| BlockMath::block_center_to_corner(&transform),
        );
        golden.check(
            "blockCornerToCenter(Lcom/mojang/math/Transformation;)Lcom/mojang/math/Transformation;",
            |transform: Transformation| BlockMath::block_corner_to_center(&transform),
        );
        golden.check_mut(
            "getFaceTransformation(Lcom/mojang/math/Transformation;Lnet/minecraft/core/Direction;)Lcom/mojang/math/Transformation;",
            |transformation: &mut Transformation, original_side: &mut Direction| {
                BlockMath::get_face_transformation(transformation, *original_side)
            },
        );
        golden.finish();
    }
}
