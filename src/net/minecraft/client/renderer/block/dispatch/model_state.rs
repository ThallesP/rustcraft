//! net/minecraft/client/renderer/block/dispatch/ModelState.java
//!
//! The game's model states are `BlockModelRotation`s and their `withUvLock()`s, 96 of them made
//! once, so a model state is a `&'static dyn ModelState`, which the bakery's caches key by address
//! like Java's maps key them by reference (`ModelState::key`).

use crate::com::mojang::math::transformation::{self, Transformation};
use crate::net::minecraft::core::direction::Direction;
use crate::org::joml::matrix4f::Matrix4f;

/// `NO_TRANSFORM`, the identity.
pub static NO_TRANSFORM: Matrix4f = Matrix4f::new();

/// How a model is turned for a block state, and how its faces' textures turn with it.
pub trait ModelState: Send + Sync {
    fn transformation(&self) -> &Transformation {
        &transformation::IDENTITY
    }

    fn face_transformation(&self, _face: Direction) -> &Matrix4f {
        &NO_TRANSFORM
    }

    fn inverse_face_transformation(&self, _face: Direction) -> &Matrix4f {
        &NO_TRANSFORM
    }
}

impl dyn ModelState {
    /// The state's identity, which Java's maps key by.
    pub fn key(&self) -> usize {
        std::ptr::from_ref(self).cast::<()>() as usize
    }
}
