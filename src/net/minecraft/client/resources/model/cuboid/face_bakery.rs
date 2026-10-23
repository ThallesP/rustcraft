//! net/minecraft/client/resources/model/cuboid/FaceBakery.java
//!
//! Java asks `MatrixUtil.isIdentity` about the model state's inverse face matrix, which adds the
//! properties it works out to that shared matrix; `BlockModelRotation` has asked already when it
//! made them (see there), so the port asks about a copy, which answers the same and adds nothing.

use std::sync::Arc;

use crate::com::mojang::blaze3d::platform::transparency::Transparency;
use crate::com::mojang::math::matrix_util::MatrixUtil;
use crate::com::mojang::math::quadrant::Quadrant;
use crate::com::mojang::math::transformation::Transformation;
use crate::java::lang::math::Math;
use crate::net::minecraft::client::model::geom::builders::uv_pair::UVPair;
use crate::net::minecraft::client::renderer::block::dispatch::model_state::ModelState;
use crate::net::minecraft::client::renderer::face_info::FaceInfo;
use crate::net::minecraft::client::resources::model::cuboid::cuboid_face::{CuboidFace, UVs};
use crate::net::minecraft::client::resources::model::cuboid::cuboid_rotation::CuboidRotation;
use crate::net::minecraft::client::resources::model::geometry::baked_quad::{
    BakedQuad, MaterialInfo,
};
use crate::net::minecraft::client::resources::model::model_baker::{Interner, ModelBaker};
use crate::net::minecraft::client::resources::model::sprite::material::Baked;
use crate::net::minecraft::core::direction::Direction;
use crate::org::joml::geometry_utils::GeometryUtils;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector3f::Vector3f;

const BLOCK_MIDDLE: Vector3f = Vector3f::from_xyz(0.5, 0.5, 0.5);

pub struct FaceBakery;

impl FaceBakery {
    /// The UVs a face without any gets: its box's side, seen from outside.
    pub fn default_face_uv(from: &Vector3f, to: &Vector3f, facing: Direction) -> UVs {
        let uvs = |min_u, min_v, max_u, max_v| UVs {
            min_u,
            min_v,
            max_u,
            max_v,
        };
        match facing {
            Direction::DOWN => uvs(from.x, 16.0 - to.z, to.x, 16.0 - from.z),
            Direction::UP => uvs(from.x, from.z, to.x, to.z),
            Direction::NORTH => uvs(16.0 - to.x, 16.0 - to.y, 16.0 - from.x, 16.0 - from.y),
            Direction::SOUTH => uvs(from.x, 16.0 - to.y, to.x, 16.0 - from.y),
            Direction::WEST => uvs(from.z, 16.0 - to.y, to.z, 16.0 - from.y),
            Direction::EAST => uvs(16.0 - to.z, 16.0 - to.y, 16.0 - from.z, 16.0 - from.y),
        }
    }

    fn compute_material_transparency(material: &Baked, uvs: &UVs) -> Transparency {
        if material.force_translucent {
            Transparency::TRANSLUCENT
        } else {
            material.sprite.contents().compute_transparency(
                Math::min_f32(uvs.min_u, uvs.max_u) / 16.0,
                Math::min_f32(uvs.min_v, uvs.max_v) / 16.0,
                Math::max_f32(uvs.min_u, uvs.max_u) / 16.0,
                Math::max_f32(uvs.min_v, uvs.max_v) / 16.0,
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn bake_quad(
        model_baker: &dyn ModelBaker,
        from: &Vector3f,
        to: &Vector3f,
        face: &CuboidFace,
        material: &Baked,
        facing: Direction,
        model_state: &dyn ModelState,
        element_rotation: Option<&CuboidRotation>,
        shade_direction_override: Option<Direction>,
        light_emission: i32,
    ) -> BakedQuad {
        let uvs = face
            .uvs
            .unwrap_or_else(|| FaceBakery::default_face_uv(from, to, facing));
        let transparency = FaceBakery::compute_material_transparency(material, &uvs);
        let interner = model_baker.interner();
        let material_info = interner.material_info(MaterialInfo::of(
            material,
            transparency,
            face.tint_index,
            shade_direction_override,
            light_emission,
        ));
        FaceBakery::bake_quad_with(
            interner,
            from,
            to,
            &uvs,
            face.rotation,
            material_info,
            facing,
            model_state,
            element_rotation,
        )
    }

    /// The overload taking the UVs and the interned material.
    #[allow(clippy::too_many_arguments)]
    pub fn bake_quad_with(
        interner: &dyn Interner,
        from: &Vector3f,
        to: &Vector3f,
        uvs: &UVs,
        uv_rotation: Quadrant,
        material_info: Arc<MaterialInfo>,
        facing: Direction,
        model_state: &dyn ModelState,
        element_rotation: Option<&CuboidRotation>,
    ) -> BakedQuad {
        let uv_transform = model_state.inverse_face_transformation(facing);
        let mut vertex_positions = [Vector3f::new(); 4];
        let mut vertex_packed_uvs = [0i64; 4];
        let face_info = FaceInfo::from_facing(facing);

        for i in 0..4 {
            FaceBakery::bake_vertex(
                i,
                face_info,
                uvs,
                uv_rotation,
                uv_transform,
                from,
                to,
                &material_info,
                model_state.transformation(),
                element_rotation,
                &mut vertex_positions,
                &mut vertex_packed_uvs,
                interner,
            );
        }

        let final_direction = FaceBakery::calculate_facing(&vertex_positions);
        if let (None, Some(final_direction)) = (element_rotation, final_direction) {
            FaceBakery::recalculate_winding(
                &mut vertex_positions,
                &mut vertex_packed_uvs,
                final_direction,
            );
        }

        BakedQuad {
            positions: vertex_positions,
            packed_uvs: vertex_packed_uvs,
            direction: final_direction.unwrap_or(Direction::UP),
            material_info,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn bake_vertex(
        index: usize,
        face_info: FaceInfo,
        uvs: &UVs,
        uv_rotation: Quadrant,
        uv_transform: &Matrix4f,
        from: &Vector3f,
        to: &Vector3f,
        material_info: &MaterialInfo,
        rotation: &Transformation,
        element_rotation: Option<&CuboidRotation>,
        position_output: &mut [Vector3f; 4],
        uv_output: &mut [i64; 4],
        interner: &dyn Interner,
    ) {
        let vertex_info = face_info.get_vertex_info(index);
        let mut vertex = vertex_info.select(from, to);
        vertex.div_scalar(16.0);
        if let Some(element_rotation) = element_rotation {
            FaceBakery::rotate_vertex_by(
                &mut vertex,
                &element_rotation.origin,
                &element_rotation.transform,
            );
        }

        if !rotation.is_identity_constant() {
            FaceBakery::rotate_vertex_by(&mut vertex, &BLOCK_MIDDLE, rotation.get_matrix());
        }

        let raw_u = CuboidFace::get_u(uvs, uv_rotation, index as i32);
        let raw_v = CuboidFace::get_v(uvs, uv_rotation, index as i32);
        let (transformed_u, transformed_v) = if MatrixUtil::is_identity(&mut uv_transform.clone()) {
            (raw_u, raw_v)
        } else {
            let mut transformed_uv = Vector3f::from_xyz(
                FaceBakery::corner_to_center(raw_u),
                FaceBakery::corner_to_center(raw_v),
                0.0,
            );
            uv_transform.transform_position(&mut transformed_uv);
            (
                FaceBakery::center_to_corner(transformed_uv.x),
                FaceBakery::center_to_corner(transformed_uv.y),
            )
        };

        position_output[index] = interner.vector(vertex);
        uv_output[index] = UVPair::pack(
            material_info.sprite.get_u(transformed_u),
            material_info.sprite.get_v(transformed_v),
        );
    }

    fn corner_to_center(value: f32) -> f32 {
        value - 0.5
    }

    fn center_to_corner(value: f32) -> f32 {
        value + 0.5
    }

    fn rotate_vertex_by(vertex: &mut Vector3f, origin: &Vector3f, transformation: &Matrix4f) {
        vertex.sub(origin);
        transformation.transform_position(vertex);
        vertex.add(origin);
    }

    fn calculate_facing(positions: &[Vector3f; 4]) -> Option<Direction> {
        let mut normal = Vector3f::new();
        GeometryUtils::normal(&positions[0], &positions[1], &positions[2], &mut normal);
        FaceBakery::find_closest_direction(&normal)
    }

    fn find_closest_direction(direction: &Vector3f) -> Option<Direction> {
        if !direction.is_finite() {
            return None;
        }

        let mut best = None;
        let mut closest_product = 0.0f32;
        for candidate in Direction::VALUES {
            let product = direction.dot(candidate.get_unit_vec3f());
            if product >= 0.0 && product > closest_product {
                closest_product = product;
                best = Some(candidate);
            }
        }
        best
    }

    /// Puts the vertices back in the order a face toward `direction` has them, by where each
    /// lies in their bounding box.
    fn recalculate_winding(
        positions: &mut [Vector3f; 4],
        uvs: &mut [i64; 4],
        direction: Direction,
    ) {
        let mut min_x = 999.0f32;
        let mut min_y = 999.0f32;
        let mut min_z = 999.0f32;
        let mut max_x = -999.0f32;
        let mut max_y = -999.0f32;
        let mut max_z = -999.0f32;

        for position in positions.iter() {
            let (x, y, z) = (position.x, position.y, position.z);
            if x < min_x {
                min_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if z < min_z {
                min_z = z;
            }
            if x > max_x {
                max_x = x;
            }
            if y > max_y {
                max_y = y;
            }
            if z > max_z {
                max_z = z;
            }
        }

        let info = FaceInfo::from_facing(direction);
        for vertex in 0..4 {
            let vert_info = info.get_vertex_info(vertex);
            let new_x = vert_info
                .x_face
                .select_xyz(min_x, min_y, min_z, max_x, max_y, max_z);
            let new_y = vert_info
                .y_face
                .select_xyz(min_x, min_y, min_z, max_x, max_y, max_z);
            let new_z = vert_info
                .z_face
                .select_xyz(min_x, min_y, min_z, max_x, max_y, max_z);
            let Some(vertex_to_swap) =
                FaceBakery::find_vertex(positions, vertex, new_x, new_y, new_z)
            else {
                panic!("IllegalStateException: Can't find vertex to swap");
            };
            if vertex_to_swap != vertex {
                positions.swap(vertex_to_swap, vertex);
                uvs.swap(vertex_to_swap, vertex);
            }
        }
    }

    fn find_vertex(
        positions: &[Vector3f; 4],
        start: usize,
        x: f32,
        y: f32,
        z: f32,
    ) -> Option<usize> {
        (start..4).find(|&i| {
            let position = &positions[i];
            x == position.x && y == position.y && z == position.z
        })
    }
}
