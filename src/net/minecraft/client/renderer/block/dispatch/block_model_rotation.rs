//! net/minecraft/client/renderer/block/dispatch/BlockModelRotation.java
//!
//! The 48 rotations are made once, like Java's `BY_GROUP_ORDINAL`, and handed out as `&'static`.
//!
//! The shared matrices: Java's constructor asks `BlockMath.getFaceTransformation` about its own
//! transformation, which adds the properties `MatrixUtil.isIdentity` works out to its matrix, and
//! the constructor owns it here too. The inverse face matrices are only ever read by `FaceBakery`,
//! which asks `isIdentity` about one before anything else reads it, adding the same properties.
//! Those only depend on the matrix, and asking again adds nothing, so the port asks once when it
//! makes them, and they're read-only after: every holder sees them the way Java's do from their
//! first bake on.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::math::matrix_util::MatrixUtil;
use crate::com::mojang::math::octahedral_group::OctahedralGroup;
use crate::com::mojang::math::transformation::{self, Transformation};
use crate::net::minecraft::client::renderer::block::dispatch::model_state::ModelState;
use crate::net::minecraft::core::block_math::BlockMath;
use crate::net::minecraft::core::direction::Direction;
use crate::org::joml::matrix4f::Matrix4f;

static BY_GROUP_ORDINAL: LazyLock<[BlockModelRotation; 48]> =
    LazyLock::new(|| OctahedralGroup::VALUES.map(BlockModelRotation::new));

pub struct BlockModelRotation {
    orientation: OctahedralGroup,
    transformation: Transformation,
    face_mapping: [Matrix4f; 6],
    inverse_face_mapping: [Matrix4f; 6],
    with_uv_lock: WithUvLock,
}

/// `BlockModelRotation.WithUvLock`: the rotation, with each face's texture turned back so it
/// stays put in the world.
pub struct WithUvLock {
    parent: OctahedralGroup,
}

impl BlockModelRotation {
    fn new(orientation: OctahedralGroup) -> BlockModelRotation {
        let mut transformation = if orientation != OctahedralGroup::IDENTITY {
            Transformation::new(Matrix4f::from_matrix3f(orientation.transformation()))
        } else {
            transformation::IDENTITY.clone()
        };

        let mut face_mapping = [Matrix4f::new(); 6];
        let mut inverse_face_mapping = [Matrix4f::new(); 6];
        for face in Direction::VALUES {
            let face_transform =
                *BlockMath::get_face_transformation(&mut transformation, face).get_matrix();
            face_mapping[face as usize] = face_transform;
            let inverse = &mut inverse_face_mapping[face as usize];
            face_transform.invert_affine_into(inverse);
            MatrixUtil::is_identity(inverse);
        }

        BlockModelRotation {
            orientation,
            transformation,
            face_mapping,
            inverse_face_mapping,
            with_uv_lock: WithUvLock {
                parent: orientation,
            },
        }
    }

    pub fn identity() -> &'static BlockModelRotation {
        BlockModelRotation::get(OctahedralGroup::IDENTITY)
    }

    pub fn get(group: OctahedralGroup) -> &'static BlockModelRotation {
        &BY_GROUP_ORDINAL[group as usize]
    }

    pub fn with_uv_lock(&'static self) -> &'static WithUvLock {
        &self.with_uv_lock
    }
}

impl ModelState for BlockModelRotation {
    fn transformation(&self) -> &Transformation {
        &self.transformation
    }
}

impl ModelState for WithUvLock {
    fn transformation(&self) -> &Transformation {
        &BlockModelRotation::get(self.parent).transformation
    }

    fn face_transformation(&self, face: Direction) -> &Matrix4f {
        &BlockModelRotation::get(self.parent).face_mapping[face as usize]
    }

    fn inverse_face_transformation(&self, face: Direction) -> &Matrix4f {
        &BlockModelRotation::get(self.parent).inverse_face_mapping[face as usize]
    }
}

impl fmt::Display for BlockModelRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "simple[{}]", self.orientation.get_serialized_name())
    }
}

impl fmt::Display for WithUvLock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "uvLocked[{}]", self.parent.get_serialized_name())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::BlockModelRotation;
    use crate::com::mojang::math::octahedral_group::OctahedralGroup;
    use crate::golden::{Encode, Golden};
    use crate::net::minecraft::client::renderer::block::dispatch::model_state::ModelState;
    use crate::net::minecraft::core::direction::Direction;

    /// Each rotation's `toString`, transformation and face matrices, and its uv-locked state's,
    /// the inverse face matrices as `FaceBakery` reads them (`ModelSuites`).
    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.client.renderer.block.dispatch.BlockModelRotation");
        golden.check(
            "get(Lcom/mojang/math/OctahedralGroup;)Lnet/minecraft/client/renderer/block/dispatch/BlockModelRotation;",
            |group: OctahedralGroup| -> Value {
                let rotation = BlockModelRotation::get(group);
                let uv_locked = rotation.with_uv_lock();
                let faces = |matrix: &dyn Fn(Direction) -> Value| -> Value {
                    Direction::VALUES.map(matrix).to_vec().into()
                };
                json!([
                    rotation.to_string(),
                    rotation.transformation().encode(),
                    faces(&|face| rotation.face_transformation(face).encode()),
                    faces(&|face| rotation.inverse_face_transformation(face).encode()),
                    uv_locked.to_string(),
                    uv_locked.transformation().encode(),
                    faces(&|face| uv_locked.face_transformation(face).encode()),
                    faces(&|face| uv_locked.inverse_face_transformation(face).encode()),
                ])
            },
        );
        golden.finish();
    }
}
