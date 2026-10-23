//! com/mojang/blaze3d/vertex/VertexSorting.java
//!
//! How translucent quads are ordered: by a distance function of their centroids, farthest first.
//! Java's lambdas are the distance function a sorting holds.

use std::cmp::Ordering;

use crate::com::mojang::blaze3d::vertex::compact_vector_array::CompactVectorArray;
use crate::java::lang::float::Float;
use crate::org::joml::vector3f::Vector3f;

/// `VertexSorting.DistanceFunction`.
type DistanceFunction = Box<dyn Fn(&Vector3f) -> f32 + Send + Sync>;

pub struct VertexSorting {
    function: DistanceFunction,
}

impl VertexSorting {
    /// `DISTANCE_TO_ORIGIN`.
    pub fn distance_to_origin() -> VertexSorting {
        VertexSorting::by_distance_xyz(0.0, 0.0, 0.0)
    }

    /// `ORTHOGRAPHIC_Z`.
    pub fn orthographic_z() -> VertexSorting {
        VertexSorting::by_distance_function(Box::new(|point: &Vector3f| -point.z))
    }

    pub fn by_distance_xyz(x: f32, y: f32, z: f32) -> VertexSorting {
        VertexSorting::by_distance(Vector3f::from_xyz(x, y, z))
    }

    /// `byDistance(Vector3fc)`: `origin::distanceSquared`.
    pub fn by_distance(origin: Vector3f) -> VertexSorting {
        VertexSorting::by_distance_function(Box::new(move |point: &Vector3f| {
            origin.distance_squared(point)
        }))
    }

    pub fn by_distance_function(function: DistanceFunction) -> VertexSorting {
        VertexSorting { function }
    }

    /// The points' indices, farthest first: fastutil's `IntArrays.mergeSort`, which is stable,
    /// with `Floats.compare` of the keys in reverse.
    pub fn sort(&self, values: &CompactVectorArray) -> Vec<i32> {
        let mut scratch = Vector3f::new();
        let keys: Vec<f32> = (0..values.size())
            .map(|i| (self.function)(values.get(i, &mut scratch)))
            .collect();
        let mut indices: Vec<i32> = (0..values.size() as i32).collect();
        indices.sort_by(|&o1, &o2| compare(keys[o2 as usize], keys[o1 as usize]));
        indices
    }
}

/// Guava's `Floats.compare`, which is `Float.compare`: -0.0 below 0.0, and NaN above everything.
fn compare(f1: f32, f2: f32) -> Ordering {
    if f1 < f2 {
        Ordering::Less
    } else if f1 > f2 {
        Ordering::Greater
    } else {
        Float::float_to_int_bits(f1).cmp(&Float::float_to_int_bits(f2))
    }
}
