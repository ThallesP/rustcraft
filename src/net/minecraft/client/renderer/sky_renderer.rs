//! net/minecraft/client/renderer/SkyRenderer.java
//!
//! The overworld's sky: the sky disc in the sky's colour, faded into the fog, and the dark disc
//! below the horizon when the camera is under it. Not ported yet: the sun and the moon
//! (`renderSun`, `renderMoon`, which need the celestials atlas), the stars and the sunrise and
//! sunset fan (`buildStars`, `buildSunriseFan`), and the end's sky and its flashes. The stars and
//! the fan panic when they'd be drawn (at night, and at dawn and dusk); the sun and the moon are
//! 100 blocks out at the sun's angle and its opposite, at least 73° from the horizon at noon, so
//! they're out of a 70° view that looks no more than 35° up or down. Java's buffers for the
//! transforms are a ring the frame writes into (`DynamicGpuData`); here each draw makes a buffer
//! of its own with the same bytes.

use std::rc::Rc;

use crate::com::mojang::blaze3d::systems::render_system::RenderSystem;
use crate::com::mojang::blaze3d::vertex::buffer_builder::BufferBuilder;
use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
use crate::com::mojang::blaze3d::vertex::default_vertex_format::DefaultVertexFormat;
use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::dynamic_gpu_data::Transform;
use crate::net::minecraft::util::mth::Mth;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector3f::Vector3f;
use crate::org::joml::vector4f::Vector4f;

/// `SkyRenderState`: the sky's part of the level's render state.
#[derive(Clone, Debug, Default)]
pub struct SkyRenderState {
    pub sun_angle: f32,
    pub moon_angle: f32,
    pub star_angle: f32,
    pub rain_brightness: f32,
    pub star_brightness: f32,
    pub sunrise_and_sunset_color: Vector4f,
    pub sky_color: Vector3f,
    pub should_render_dark_disc: bool,
}

/// The sky's pipelines, compiled.
pub struct SkyPipelines {
    pub sky: Rc<dyn CompiledRenderPipeline>,
}

pub struct SkyRenderer {
    top_sky_buffer: Rc<dyn GpuBuffer>,
    bottom_sky_buffer: Rc<dyn GpuBuffer>,
}

impl SkyRenderer {
    const SKY_DISC_RADIUS: f32 = 512.0;

    pub fn new(device: &dyn GpuDevice) -> SkyRenderer {
        let mut builder =
            ByteBufferBuilder::exactly_sized(10 * DefaultVertexFormat::POSITION.get_vertex_size());
        let mut buffer_builder = BufferBuilder::new(
            &mut builder,
            PrimitiveTopology::TRIANGLE_FAN,
            DefaultVertexFormat::POSITION,
        );
        SkyRenderer::build_sky_disc(&mut buffer_builder, 16.0);
        let mesh = buffer_builder.build_or_throw();
        let top_sky_buffer = device.create_buffer_with_data(
            Some("Top sky vertex buffer"),
            gpu_buffer::USAGE_VERTEX,
            mesh.vertex_buffer(),
        );
        let mut buffer_builder = BufferBuilder::new(
            &mut builder,
            PrimitiveTopology::TRIANGLE_FAN,
            DefaultVertexFormat::POSITION,
        );
        SkyRenderer::build_sky_disc(&mut buffer_builder, -16.0);
        let mesh = buffer_builder.build_or_throw();
        let bottom_sky_buffer = device.create_buffer_with_data(
            Some("Bottom sky vertex buffer"),
            gpu_buffer::USAGE_VERTEX,
            mesh.vertex_buffer(),
        );
        SkyRenderer {
            top_sky_buffer,
            bottom_sky_buffer,
        }
    }

    /// The overworld's sky into the main target, the sky's fog bound.
    pub fn render(
        &self,
        device: &dyn GpuDevice,
        pipelines: &SkyPipelines,
        sky_fog: &GpuBufferSlice,
        color_texture: &GpuTextureView,
        depth_texture: &GpuTextureView,
        state: &SkyRenderState,
    ) {
        RenderSystem::set_shader_fog(sky_fog.clone());
        let encoder = device.create_command_encoder();
        let mut render_pass =
            encoder.create_render_pass_with("Sky", color_texture, None, Some(depth_texture), None);
        RenderSystem::bind_default_uniforms(&mut *render_pass);
        self.render_sky_disc(device, pipelines, &mut *render_pass, &state.sky_color);
        self.render_sunrise_and_sunset(&state.sunrise_and_sunset_color);
        self.render_sun_moon_and_stars(state.star_brightness);
        if state.should_render_dark_disc {
            self.render_dark_disc(device, pipelines, &mut *render_pass);
        }
    }

    fn write_transform(
        device: &dyn GpuDevice,
        model_view: Matrix4f,
        color: Vector4f,
    ) -> Rc<dyn GpuBuffer> {
        let mut transform = Transform::of_model_view(model_view);
        transform.color_modulator = color;
        device.create_buffer_with_data(
            Some("Dynamic Transforms UBO"),
            gpu_buffer::USAGE_UNIFORM,
            &transform.write(),
        )
    }

    fn render_sky_disc(
        &self,
        device: &dyn GpuDevice,
        pipelines: &SkyPipelines,
        render_pass: &mut dyn RenderPass,
        sky_color: &Vector3f,
    ) {
        let dynamic_transforms = SkyRenderer::write_transform(
            device,
            RenderSystem::get_model_view_matrix_copy(),
            Vector4f::from_xyzw(sky_color.x, sky_color.y, sky_color.z, 1.0),
        );
        render_pass.push_debug_group("Sky disc");
        render_pass.set_pipeline(&pipelines.sky);
        render_pass.set_uniform_buffer("DynamicTransforms", &dynamic_transforms);
        render_pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(&self.top_sky_buffer)));
        render_pass.draw(10, 1, 0, 0);
        render_pass.pop_debug_group();
    }

    fn render_dark_disc(
        &self,
        device: &dyn GpuDevice,
        pipelines: &SkyPipelines,
        render_pass: &mut dyn RenderPass,
    ) {
        let model_view = RenderSystem::with_model_view_stack(|stack| {
            stack.push_matrix();
            stack.translate(0.0, 12.0, 0.0);
            let model_view = Matrix4f::from_matrix4f(stack);
            stack.pop_matrix();
            model_view
        });
        let dynamic_transforms = SkyRenderer::write_transform(
            device,
            model_view,
            Vector4f::from_xyzw(0.0, 0.0, 0.0, 1.0),
        );
        render_pass.push_debug_group("Dark disc");
        render_pass.set_pipeline(&pipelines.sky);
        RenderSystem::bind_default_uniforms(render_pass);
        render_pass.set_uniform_buffer("DynamicTransforms", &dynamic_transforms);
        render_pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(&self.bottom_sky_buffer)));
        render_pass.draw(10, 1, 0, 0);
        render_pass.pop_debug_group();
    }

    /// The stars, at their angle; the sun and the moon aren't ported (see the header).
    fn render_sun_moon_and_stars(&self, star_brightness: f32) {
        assert!(
            star_brightness <= 0.0,
            "the stars (SkyRenderer.buildStars) aren't ported"
        );
    }

    fn render_sunrise_and_sunset(&self, sunrise_and_sunset_color: &Vector4f) {
        let alpha = sunrise_and_sunset_color.w;
        assert!(
            alpha <= 0.001,
            "the sunrise and sunset (SkyRenderer.buildSunriseFan) aren't ported"
        );
    }

    fn build_sky_disc(builder: &mut dyn VertexConsumer, yy: f32) {
        let x = yy.signum() * SkyRenderer::SKY_DISC_RADIUS;
        builder.add_vertex(0.0, yy, 0.0);
        let mut i = -180;
        while i <= 180 {
            let radians = i as f32 * (std::f64::consts::PI / 180.0) as f32;
            builder.add_vertex(
                x * Mth::cos(f64::from(radians)),
                yy,
                SkyRenderer::SKY_DISC_RADIUS * Mth::sin(f64::from(radians)),
            );
            i += 45;
        }
    }
}
