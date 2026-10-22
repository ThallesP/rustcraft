//! JOML 1.10.9, the vector and matrix library the client does its transforms with, ported from
//! its sources (MIT) like the game's own classes. Only what the client calls is ported, and
//! each class's golden suite lists those methods.
//!
//! JOML's read-only interfaces (`Matrix4fc`, `Vector3fc`, …) are only implemented by its own
//! classes, so they become shared references (`&Matrix4f`). A method that writes to a `dest`
//! argument instead of `this` gets `_into` (`invert_into(dest)`); its fluent overload, which
//! passes `this`, keeps the name and returns `&mut Self`.

pub mod frustum_intersection;
pub mod math;
pub mod matrix3f;
pub mod matrix3x2f;
pub mod matrix3x2f_stack;
pub mod matrix4f;
pub mod matrix4f_stack;
pub mod matrix4fc;
pub mod quaternionf;
pub mod rounding_mode;
pub mod vector2f;
pub mod vector3f;
pub mod vector3i;
pub mod vector4f;
