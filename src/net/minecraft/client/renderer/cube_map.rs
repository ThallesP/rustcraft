//! net/minecraft/client/renderer/CubeMap.java
//!
//! The panorama's cube, seen from its middle: six quads around the camera, drawn into the main
//! target with `RenderPipelines.PANORAMA` sampling the cube texture at its location, under a
//! perspective of 85 degrees turned by the given angles. Java asks the game for the window's size,
//! its main target, the texture manager and the shared quad indices; here they're arguments.

use std::rc::Rc;

use crate::com::mojang::blaze3d::pipeline::main_target::MainTarget;
use crate::com::mojang::blaze3d::systems::render_system::{AutoStorageIndexBuffer, RenderSystem};
use crate::com::mojang::blaze3d::vertex::buffer_builder::BufferBuilder;
use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
use crate::com::mojang::blaze3d::vertex::default_vertex_format::DefaultVertexFormat;
use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::dynamic_gpu_data::Transform;
use crate::net::minecraft::client::renderer::projection::Projection;
use crate::net::minecraft::client::renderer::projection_matrix_buffer::ProjectionMatrixBuffer;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::org::joml::matrix4f::Matrix4f;

pub struct CubeMap {
    vertex_buffer: Rc<dyn GpuBuffer>,
    projection: Projection,
    projection_matrix_ubo: ProjectionMatrixBuffer,
    location: Identifier,
}

impl CubeMap {
    const PROJECTION_Z_NEAR: f32 = 0.05;
    const PROJECTION_Z_FAR: f32 = 10.0;
    const PROJECTION_FOV: f32 = 85.0;

    pub fn new(device: &dyn GpuDevice, base: Identifier) -> CubeMap {
        CubeMap {
            vertex_buffer: CubeMap::initialize_vertices(device),
            projection: Projection::new(),
            projection_matrix_ubo: ProjectionMatrixBuffer::new(device, "cubemap"),
            location: base,
        }
    }

    /// The cube texture's location, which `registerTextures` registers a `CubeMapTexture` at.
    pub fn location(&self) -> &Identifier {
        &self.location
    }

    /// `render`: the cube turned by `rot_x_in_degrees` about x, then `rot_y_in_degrees` about y.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &dyn GpuDevice,
        pipeline: &Rc<dyn CompiledRenderPipeline>,
        main_render_target: &MainTarget,
        quad_indices: &mut AutoStorageIndexBuffer,
        texture: (&GpuTextureView, &Rc<dyn GpuSampler>),
        window_size: (i32, i32),
        rot_x_in_degrees: f32,
        rot_y_in_degrees: f32,
    ) {
        self.projection.setup_perspective(
            CubeMap::PROJECTION_Z_NEAR,
            CubeMap::PROJECTION_Z_FAR,
            CubeMap::PROJECTION_FOV,
            window_size.0 as f32,
            window_size.1 as f32,
        );
        RenderSystem::set_projection_matrix(
            self.projection_matrix_ubo
                .get_buffer(device, &mut self.projection),
        );
        let index_buffer = quad_indices.get_buffer(device, 36);
        // The model view stack pushed, set, written and popped: what's written is the matrix.
        let mut model_view = Matrix4f::new();
        model_view.rotation_x(std::f32::consts::PI);
        model_view.rotate_x(rot_x_in_degrees * (std::f32::consts::PI / 180.0));
        model_view.rotate_y(rot_y_in_degrees * (std::f32::consts::PI / 180.0));
        let dynamic_transforms = device.create_buffer_with_data(
            Some("Dynamic Transforms UBO"),
            gpu_buffer::USAGE_UNIFORM,
            &Transform::of_model_view(model_view).write(),
        );

        let encoder = device.create_command_encoder();
        let mut render_pass = encoder.create_render_pass_with(
            "Cubemap",
            main_render_target
                .get_color_texture_view()
                .expect("the main target has color"),
            None,
            main_render_target.get_depth_texture_view(),
            None,
        );
        render_pass.set_pipeline(pipeline);
        RenderSystem::bind_default_uniforms(&mut *render_pass);
        render_pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(&self.vertex_buffer)));
        render_pass.set_index_buffer(&index_buffer, quad_indices.index_type());
        render_pass.set_uniform_buffer("DynamicTransforms", &dynamic_transforms);
        render_pass.set_uniform_sampler("Sampler0", Some(texture.0), Some(texture.1));
        render_pass.draw_indexed(36, 1, 0, 0, 0);
    }

    fn initialize_vertices(device: &dyn GpuDevice) -> Rc<dyn GpuBuffer> {
        let mut byte_buffer_builder = ByteBufferBuilder::exactly_sized(
            DefaultVertexFormat::POSITION.get_vertex_size() * 4 * 6,
        );
        let mut buffer_builder = BufferBuilder::new(
            &mut byte_buffer_builder,
            PrimitiveTopology::QUADS,
            DefaultVertexFormat::POSITION,
        );
        for [x, y, z] in [
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, 1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, -1.0, -1.0],
            [-1.0, 1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, -1.0],
            [1.0, 1.0, 1.0],
        ] {
            buffer_builder.add_vertex(x, y, z);
        }
        let mesh = buffer_builder.build_or_throw();
        device.create_buffer_with_data(
            Some("Cube map vertex buffer"),
            gpu_buffer::USAGE_VERTEX,
            mesh.vertex_buffer(),
        )
    }
}
