//! net/minecraft/client/renderer/FaceInfo.java
//!
//! Which corner of a box each of a face's four vertices is, in the order `FaceBakery` bakes them.

use crate::net::minecraft::core::direction::Direction;
use crate::org::joml::vector3f::Vector3f;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FaceInfo {
    DOWN,
    UP,
    NORTH,
    SOUTH,
    WEST,
    EAST,
}

/// `FaceInfo.Extent`: one side of a box along one axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Extent {
    MIN_X,
    MIN_Y,
    MIN_Z,
    MAX_X,
    MAX_Y,
    MAX_Z,
}

/// `FaceInfo.VertexInfo`: a vertex as the box's sides it's on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VertexInfo {
    pub x_face: Extent,
    pub y_face: Extent,
    pub z_face: Extent,
}

const fn vertex(x_face: Extent, y_face: Extent, z_face: Extent) -> VertexInfo {
    VertexInfo {
        x_face,
        y_face,
        z_face,
    }
}

use Extent::{MAX_X, MAX_Y, MAX_Z, MIN_X, MIN_Y, MIN_Z};

const INFOS: [[VertexInfo; 4]; 6] = [
    [
        vertex(MIN_X, MIN_Y, MAX_Z),
        vertex(MIN_X, MIN_Y, MIN_Z),
        vertex(MAX_X, MIN_Y, MIN_Z),
        vertex(MAX_X, MIN_Y, MAX_Z),
    ],
    [
        vertex(MIN_X, MAX_Y, MIN_Z),
        vertex(MIN_X, MAX_Y, MAX_Z),
        vertex(MAX_X, MAX_Y, MAX_Z),
        vertex(MAX_X, MAX_Y, MIN_Z),
    ],
    [
        vertex(MAX_X, MAX_Y, MIN_Z),
        vertex(MAX_X, MIN_Y, MIN_Z),
        vertex(MIN_X, MIN_Y, MIN_Z),
        vertex(MIN_X, MAX_Y, MIN_Z),
    ],
    [
        vertex(MIN_X, MAX_Y, MAX_Z),
        vertex(MIN_X, MIN_Y, MAX_Z),
        vertex(MAX_X, MIN_Y, MAX_Z),
        vertex(MAX_X, MAX_Y, MAX_Z),
    ],
    [
        vertex(MIN_X, MAX_Y, MIN_Z),
        vertex(MIN_X, MIN_Y, MIN_Z),
        vertex(MIN_X, MIN_Y, MAX_Z),
        vertex(MIN_X, MAX_Y, MAX_Z),
    ],
    [
        vertex(MAX_X, MAX_Y, MAX_Z),
        vertex(MAX_X, MIN_Y, MAX_Z),
        vertex(MAX_X, MIN_Y, MIN_Z),
        vertex(MAX_X, MAX_Y, MIN_Z),
    ],
];

impl FaceInfo {
    pub fn from_facing(direction: Direction) -> FaceInfo {
        match direction {
            Direction::DOWN => FaceInfo::DOWN,
            Direction::UP => FaceInfo::UP,
            Direction::NORTH => FaceInfo::NORTH,
            Direction::SOUTH => FaceInfo::SOUTH,
            Direction::WEST => FaceInfo::WEST,
            Direction::EAST => FaceInfo::EAST,
        }
    }

    /// Java throws `ArrayIndexOutOfBoundsException` past 3.
    pub fn get_vertex_info(&self, index: usize) -> VertexInfo {
        INFOS[*self as usize][index]
    }
}

impl Extent {
    pub fn select(&self, min: &Vector3f, max: &Vector3f) -> f32 {
        self.select_xyz(min.x, min.y, min.z, max.x, max.y, max.z)
    }

    pub fn select_xyz(
        &self,
        min_x: f32,
        min_y: f32,
        min_z: f32,
        max_x: f32,
        max_y: f32,
        max_z: f32,
    ) -> f32 {
        match self {
            Extent::MIN_X => min_x,
            Extent::MIN_Y => min_y,
            Extent::MIN_Z => min_z,
            Extent::MAX_X => max_x,
            Extent::MAX_Y => max_y,
            Extent::MAX_Z => max_z,
        }
    }
}

impl VertexInfo {
    pub fn select(&self, min: &Vector3f, max: &Vector3f) -> Vector3f {
        Vector3f::from_xyz(
            self.x_face.select(min, max),
            self.y_face.select(min, max),
            self.z_face.select(min, max),
        )
    }
}
