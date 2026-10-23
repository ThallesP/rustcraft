//! com/mojang/blaze3d/vertex/VertexConsumer.java
//!
//! Where vertices go, an element at a time. Overloads take what they add: `set_color` takes an
//! ARGB int, `set_color_rgba` the four channels and `set_color_f32` floats; `add_vertex_full` is
//! the eleven-argument `addVertex`, and `add_vertex_matrix` the one taking a `Matrix4fc`. The
//! pose overloads of `putBakedQuad` come with entities.

use crate::com::mojang::blaze3d::vertex::quad_instance::QuadInstance;
use crate::net::minecraft::client::model::geom::builders::uv_pair::UVPair;
use crate::net::minecraft::client::resources::model::geometry::baked_quad::BakedQuad;
use crate::org::joml::matrix3x2f::Matrix3x2f;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector2f::Vector2f;
use crate::org::joml::vector3f::Vector3f;

pub trait VertexConsumer {
    fn add_vertex(&mut self, x: f32, y: f32, z: f32) -> &mut dyn VertexConsumer;

    fn set_color_rgba(&mut self, r: i32, g: i32, b: i32, a: i32) -> &mut dyn VertexConsumer;

    fn set_color(&mut self, color: i32) -> &mut dyn VertexConsumer;

    fn set_uv(&mut self, u: f32, v: f32) -> &mut dyn VertexConsumer;

    fn set_uv1(&mut self, u: i32, v: i32) -> &mut dyn VertexConsumer;

    fn set_uv2(&mut self, u: i32, v: i32) -> &mut dyn VertexConsumer;

    fn set_uv3(&mut self, u: f32, v: f32) -> &mut dyn VertexConsumer;

    fn set_normal(&mut self, x: f32, y: f32, z: f32) -> &mut dyn VertexConsumer;

    fn set_line_width(&mut self, width: f32) -> &mut dyn VertexConsumer;

    #[allow(clippy::too_many_arguments)]
    fn add_vertex_full(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        color: i32,
        u: f32,
        v: f32,
        overlay_coords: i32,
        light_coords: i32,
        nx: f32,
        ny: f32,
        nz: f32,
    ) {
        self.add_vertex(x, y, z);
        self.set_color(color);
        self.set_uv(u, v);
        self.set_overlay(overlay_coords);
        self.set_light(light_coords);
        self.set_normal(nx, ny, nz);
    }

    /// A block's quad at `(x, y, z)` in the section, each vertex with the instance's colour and
    /// light (with the quad's own emission), and the quad's facing as its normal.
    fn put_block_baked_quad(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        quad: &BakedQuad,
        instance: &QuadInstance,
    ) {
        let normal = quad.direction.get_unit_vec3f();
        let light_emission = quad.material_info.light_emission;
        for vertex in 0..4 {
            let pos = quad.position(vertex);
            let packed_uv = quad.packed_uv(vertex);
            let vertex_color = instance.get_color(vertex);
            let light = instance.get_light_coords_with_emission(vertex, light_emission);
            let u = UVPair::unpack_u(packed_uv);
            let v = UVPair::unpack_v(packed_uv);
            self.add_vertex_full(
                pos.x + x,
                pos.y + y,
                pos.z + z,
                vertex_color,
                u,
                v,
                instance.overlay_coords(),
                light,
                normal.x,
                normal.y,
                normal.z,
            );
        }
    }

    fn set_color_f32(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut dyn VertexConsumer {
        self.set_color_rgba(
            (r * 255.0) as i32,
            (g * 255.0) as i32,
            (b * 255.0) as i32,
            (a * 255.0) as i32,
        )
    }

    fn set_light(&mut self, packed_light_coords: i32) -> &mut dyn VertexConsumer {
        self.set_uv2(
            packed_light_coords & 65535,
            packed_light_coords >> 16 & 65535,
        )
    }

    fn set_overlay(&mut self, packed_overlay_coords: i32) -> &mut dyn VertexConsumer {
        self.set_uv1(
            packed_overlay_coords & 65535,
            packed_overlay_coords >> 16 & 65535,
        )
    }

    fn add_vertex_matrix(
        &mut self,
        pose: &Matrix4f,
        x: f32,
        y: f32,
        z: f32,
    ) -> &mut dyn VertexConsumer {
        let mut pos = Vector3f::new();
        pose.transform_position_xyz_into(x, y, z, &mut pos);
        self.add_vertex(pos.x, pos.y, pos.z)
    }

    fn add_vertex_with_2d_pose(
        &mut self,
        pose: &Matrix3x2f,
        x: f32,
        y: f32,
    ) -> &mut dyn VertexConsumer {
        let mut pos = Vector2f::new();
        pose.transform_position_xy_into(x, y, &mut pos);
        self.add_vertex(pos.x, pos.y, 0.0)
    }
}
