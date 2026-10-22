//! net/minecraft/core/Position.java

/// A point in the world, in blocks. `Vec3` is one, and so are entities' positions.
pub trait Position {
    fn x(&self) -> f64;

    fn y(&self) -> f64;

    fn z(&self) -> f64;
}
