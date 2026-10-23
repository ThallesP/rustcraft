//! net/minecraft/client/renderer/SkyRenderer.java
//!
//! The overworld's sky: the sky disc in the sky's colour, faded into the fog; the sunrise and
//! sunset fan; the sun, the moon in its phase and the stars at their angles, from the celestials
//! atlas; and the dark disc below the horizon when the camera is under it. A dimension without a
//! sky (the nether's `Skybox.NONE`) draws no sky pass at all (`LevelRenderer.addSkyPass`). Not
//! ported yet: the end's sky (`renderEndSky`, the `end_sky.png` texture through the texture
//! manager's loading, which the port doesn't do yet) and its flashes (`EndFlashState`), so in the
//! end the sky is the clear colour. Java's buffers for the transforms are a ring the frame writes
//! into (`DynamicGpuData`); here each draw makes a buffer of its own with the same bytes.

use std::rc::Rc;

use crate::com::mojang::blaze3d::systems::render_system::{AutoStorageIndexBuffer, RenderSystem};
use crate::com::mojang::blaze3d::vertex::buffer_builder::BufferBuilder;
use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
use crate::com::mojang::blaze3d::vertex::default_vertex_format::DefaultVertexFormat;
use crate::com::mojang::blaze3d::vertex::pose_stack::PoseStack;
use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::math::axis::Axis;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::dynamic_gpu_data::Transform;
use crate::net::minecraft::client::renderer::texture::texture_atlas::TextureAtlas;
use crate::net::minecraft::client::renderer::texture::texture_atlas_sprite::TextureAtlasSprite;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::dimension::dimension_type::Skybox;
use crate::net::minecraft::world::level::levelgen::single_threaded_random_source::SingleThreadedRandomSource;
use crate::net::minecraft::world::level::moon_phase::MoonPhase;
use crate::org::joml::matrix3f::Matrix3f;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector3f::Vector3f;
use crate::org::joml::vector4f::Vector4f;

/// `SkyRenderState`: the sky's part of the level's render state.
#[derive(Clone, Debug)]
pub struct SkyRenderState {
    pub skybox: Skybox,
    pub should_render_dark_disc: bool,
    pub sun_angle: f32,
    pub moon_angle: f32,
    pub star_angle: f32,
    pub rain_brightness: f32,
    pub star_brightness: f32,
    pub sunrise_and_sunset_color: Vector4f,
    pub moon_phase: MoonPhase,
    pub sky_color: Vector3f,
}

impl Default for SkyRenderState {
    fn default() -> SkyRenderState {
        SkyRenderState {
            skybox: Skybox::NONE,
            should_render_dark_disc: false,
            sun_angle: 0.0,
            moon_angle: 0.0,
            star_angle: 0.0,
            rain_brightness: 0.0,
            star_brightness: 0.0,
            sunrise_and_sunset_color: Vector4f::new(),
            moon_phase: MoonPhase::FULL_MOON,
            sky_color: Vector3f::new(),
        }
    }
}

/// The sky's pipelines, compiled.
pub struct SkyPipelines {
    pub sky: Rc<dyn CompiledRenderPipeline>,
    pub sunrise_sunset: Rc<dyn CompiledRenderPipeline>,
    pub celestial: Rc<dyn CompiledRenderPipeline>,
    pub stars: Rc<dyn CompiledRenderPipeline>,
}

/// What the sky draws with besides its own buffers: the celestials atlas and the shared quad
/// indices (`RenderSystem.getSequentialBuffer(QUADS)`).
pub struct SkyResources<'a> {
    pub pipelines: &'a SkyPipelines,
    pub celestials: &'a GpuTextureView,
    pub celestials_sampler: &'a Rc<dyn GpuSampler>,
    pub quad_indices: &'a mut AutoStorageIndexBuffer,
}

pub struct SkyRenderer {
    star_buffer: Rc<dyn GpuBuffer>,
    top_sky_buffer: Rc<dyn GpuBuffer>,
    bottom_sky_buffer: Rc<dyn GpuBuffer>,
    sun_buffer: Rc<dyn GpuBuffer>,
    moon_buffer: Rc<dyn GpuBuffer>,
    sunrise_buffer: Rc<dyn GpuBuffer>,
    star_index_count: i32,
}

impl SkyRenderer {
    const SKY_DISC_RADIUS: f32 = 512.0;

    const STAR_COUNT: i32 = 1500;
    const SUN_SIZE: f32 = 30.0;
    const SUN_HEIGHT: f32 = 100.0;
    const MOON_SIZE: f32 = 20.0;
    const MOON_HEIGHT: f32 = 100.0;
    const SUNRISE_STEPS: i32 = 16;

    /// The sky's buffers, its sun and moon from the celestials atlas.
    pub fn new(device: &dyn GpuDevice, celestials_atlas: &TextureAtlas) -> SkyRenderer {
        let (star_buffer, star_index_count) = SkyRenderer::build_stars(device);
        let sun_buffer = SkyRenderer::build_sun_quad(device, celestials_atlas);
        let moon_buffer = SkyRenderer::build_moon_phases(device, celestials_atlas);
        let sunrise_buffer = SkyRenderer::build_sunrise_fan(device);

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
            star_buffer,
            top_sky_buffer,
            bottom_sky_buffer,
            sun_buffer,
            moon_buffer,
            sunrise_buffer,
            star_index_count,
        }
    }

    /// The overworld's sky into the main target, the sky's fog bound. The caller draws no sky for
    /// `Skybox.NONE`; the end's (`Skybox.END`) isn't ported, so it draws nothing either.
    pub fn render(
        &self,
        device: &dyn GpuDevice,
        resources: &mut SkyResources<'_>,
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
        if state.skybox == Skybox::END {
            return;
        }

        let mut pose_stack = PoseStack::new();
        self.render_sky_disc(device, resources, &mut *render_pass, &state.sky_color);
        self.render_sunrise_and_sunset(
            device,
            resources,
            &mut *render_pass,
            &mut pose_stack,
            state.sun_angle,
            &state.sunrise_and_sunset_color,
        );
        self.render_sun_moon_and_stars(
            device,
            resources,
            &mut *render_pass,
            &mut pose_stack,
            state,
        );
        if state.should_render_dark_disc {
            self.render_dark_disc(device, resources, &mut *render_pass);
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
        resources: &SkyResources<'_>,
        render_pass: &mut dyn RenderPass,
        sky_color: &Vector3f,
    ) {
        let dynamic_transforms = SkyRenderer::write_transform(
            device,
            RenderSystem::get_model_view_matrix_copy(),
            Vector4f::from_xyzw(sky_color.x, sky_color.y, sky_color.z, 1.0),
        );
        render_pass.push_debug_group("Sky disc");
        render_pass.set_pipeline(&resources.pipelines.sky);
        render_pass.set_uniform_buffer("DynamicTransforms", &dynamic_transforms);
        render_pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(&self.top_sky_buffer)));
        render_pass.draw(10, 1, 0, 0);
        render_pass.pop_debug_group();
    }

    fn render_dark_disc(
        &self,
        device: &dyn GpuDevice,
        resources: &SkyResources<'_>,
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
        render_pass.set_pipeline(&resources.pipelines.sky);
        RenderSystem::bind_default_uniforms(render_pass);
        render_pass.set_uniform_buffer("DynamicTransforms", &dynamic_transforms);
        render_pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(&self.bottom_sky_buffer)));
        render_pass.draw(10, 1, 0, 0);
        render_pass.pop_debug_group();
    }

    fn render_sun_moon_and_stars(
        &self,
        device: &dyn GpuDevice,
        resources: &mut SkyResources<'_>,
        render_pass: &mut dyn RenderPass,
        pose_stack: &mut PoseStack,
        state: &SkyRenderState,
    ) {
        pose_stack.push_pose();
        pose_stack.rotate_degrees(Axis::YP, -90.0);
        pose_stack.push_pose();
        pose_stack.rotate_axis(Axis::XP, state.sun_angle);
        self.render_sun(
            device,
            resources,
            render_pass,
            state.rain_brightness,
            pose_stack,
        );
        pose_stack.pop_pose();
        pose_stack.push_pose();
        pose_stack.rotate_axis(Axis::XP, state.moon_angle);
        self.render_moon(
            device,
            resources,
            render_pass,
            state.moon_phase,
            state.rain_brightness,
            pose_stack,
        );
        pose_stack.pop_pose();
        if state.star_brightness > 0.0 {
            pose_stack.push_pose();
            pose_stack.rotate_axis(Axis::XP, state.star_angle);
            self.render_stars(
                device,
                resources,
                render_pass,
                state.star_brightness,
                pose_stack,
            );
            pose_stack.pop_pose();
        }

        pose_stack.pop_pose();
    }

    fn render_sun(
        &self,
        device: &dyn GpuDevice,
        resources: &mut SkyResources<'_>,
        render_pass: &mut dyn RenderPass,
        rain_brightness: f32,
        pose_stack: &PoseStack,
    ) {
        let model_view_matrix = SkyRenderer::apply_celestial_body_transform(
            pose_stack,
            SkyRenderer::SUN_HEIGHT,
            SkyRenderer::SUN_SIZE,
        );
        let dynamic_transforms = SkyRenderer::write_transform(
            device,
            model_view_matrix,
            Vector4f::from_xyzw(1.0, 1.0, 1.0, rain_brightness),
        );
        self.draw_celestial_body(
            "Sun",
            device,
            resources,
            render_pass,
            &dynamic_transforms,
            &self.sun_buffer,
            0,
        );
    }

    fn render_moon(
        &self,
        device: &dyn GpuDevice,
        resources: &mut SkyResources<'_>,
        render_pass: &mut dyn RenderPass,
        moon_phase: MoonPhase,
        rain_brightness: f32,
        pose_stack: &PoseStack,
    ) {
        let base_vertex = moon_phase.index() * 4;
        let model_view_matrix = SkyRenderer::apply_celestial_body_transform(
            pose_stack,
            SkyRenderer::MOON_HEIGHT,
            SkyRenderer::MOON_SIZE,
        );
        let dynamic_transforms = SkyRenderer::write_transform(
            device,
            model_view_matrix,
            Vector4f::from_xyzw(1.0, 1.0, 1.0, rain_brightness),
        );
        self.draw_celestial_body(
            "Moon",
            device,
            resources,
            render_pass,
            &dynamic_transforms,
            &self.moon_buffer,
            base_vertex,
        );
    }

    fn apply_celestial_body_transform(pose_stack: &PoseStack, height: f32, scale: f32) -> Matrix4f {
        RenderSystem::with_model_view_stack(|stack| {
            stack.push_matrix();
            stack.mul(pose_stack.last().pose());
            stack.translate(0.0, height, 0.0);
            stack.scale(scale, 1.0, scale);
            let model_view_matrix = Matrix4f::from_matrix4f(stack);
            stack.pop_matrix();
            model_view_matrix
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_celestial_body(
        &self,
        label: &str,
        device: &dyn GpuDevice,
        resources: &mut SkyResources<'_>,
        render_pass: &mut dyn RenderPass,
        dynamic_transforms: &Rc<dyn GpuBuffer>,
        vertex_buffer: &Rc<dyn GpuBuffer>,
        base_vertex: i32,
    ) {
        let index_buffer = resources.quad_indices.get_buffer(device, 6);
        render_pass.push_debug_group(label);
        render_pass.set_pipeline(&resources.pipelines.celestial);
        RenderSystem::bind_default_uniforms(render_pass);
        render_pass.set_uniform_buffer("DynamicTransforms", dynamic_transforms);
        render_pass.set_uniform_sampler(
            "Sampler0",
            Some(resources.celestials),
            Some(resources.celestials_sampler),
        );
        render_pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(vertex_buffer)));
        render_pass.set_index_buffer(&index_buffer, resources.quad_indices.index_type());
        render_pass.draw_indexed(6, 1, 0, base_vertex, 0);
        render_pass.pop_debug_group();
    }

    fn render_stars(
        &self,
        device: &dyn GpuDevice,
        resources: &mut SkyResources<'_>,
        render_pass: &mut dyn RenderPass,
        star_brightness: f32,
        pose_stack: &PoseStack,
    ) {
        let model_view = RenderSystem::with_model_view_stack(|stack| {
            stack.push_matrix();
            stack.mul(pose_stack.last().pose());
            let model_view = Matrix4f::from_matrix4f(stack);
            stack.pop_matrix();
            model_view
        });
        let index_buffer = resources
            .quad_indices
            .get_buffer(device, self.star_index_count);
        let dynamic_transforms = SkyRenderer::write_transform(
            device,
            model_view,
            Vector4f::from_xyzw(
                star_brightness,
                star_brightness,
                star_brightness,
                star_brightness,
            ),
        );
        render_pass.push_debug_group("Stars");
        render_pass.set_pipeline(&resources.pipelines.stars);
        RenderSystem::bind_default_uniforms(render_pass);
        render_pass.set_uniform_buffer("DynamicTransforms", &dynamic_transforms);
        render_pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(&self.star_buffer)));
        render_pass.set_index_buffer(&index_buffer, resources.quad_indices.index_type());
        render_pass.draw_indexed(self.star_index_count, 1, 0, 0, 0);
        render_pass.pop_debug_group();
    }

    fn render_sunrise_and_sunset(
        &self,
        device: &dyn GpuDevice,
        resources: &SkyResources<'_>,
        render_pass: &mut dyn RenderPass,
        pose_stack: &mut PoseStack,
        sun_angle: f32,
        sunrise_and_sunset_color: &Vector4f,
    ) {
        let alpha = sunrise_and_sunset_color.w;
        if alpha <= 0.001 {
            return;
        }
        pose_stack.push_pose();
        pose_stack.rotate_degrees(Axis::XP, 90.0);
        let angle = if Mth::sin(f64::from(sun_angle)) < 0.0 {
            180.0
        } else {
            0.0
        };
        pose_stack.rotate_degrees(Axis::ZP, angle + 90.0);
        let model_view = RenderSystem::with_model_view_stack(|stack| {
            stack.push_matrix();
            stack.mul(pose_stack.last().pose());
            stack.scale(1.0, 1.0, alpha);
            let model_view = Matrix4f::from_matrix4f(stack);
            stack.pop_matrix();
            model_view
        });
        let dynamic_transforms =
            SkyRenderer::write_transform(device, model_view, *sunrise_and_sunset_color);
        render_pass.push_debug_group("Sunrise sunset");
        render_pass.set_pipeline(&resources.pipelines.sunrise_sunset);
        RenderSystem::bind_default_uniforms(render_pass);
        render_pass.set_uniform_buffer("DynamicTransforms", &dynamic_transforms);
        render_pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(&self.sunrise_buffer)));
        render_pass.draw(18, 1, 0, 0);
        render_pass.pop_debug_group();
        pose_stack.pop_pose();
    }

    fn build_sunrise_fan(device: &dyn GpuDevice) -> Rc<dyn GpuBuffer> {
        let vertices = 18;
        let vtx_size = DefaultVertexFormat::POSITION_COLOR.get_vertex_size();
        let mut byte_buffer_builder = ByteBufferBuilder::exactly_sized(vertices * vtx_size);
        let mut buffer_builder = BufferBuilder::new(
            &mut byte_buffer_builder,
            PrimitiveTopology::TRIANGLE_FAN,
            DefaultVertexFormat::POSITION_COLOR,
        );
        let center_color = ARGB::white_f32(1.0);
        let ring_color = ARGB::white_f32(0.0);
        buffer_builder
            .add_vertex(0.0, 100.0, 0.0)
            .set_color(center_color);

        for i in 0..=SkyRenderer::SUNRISE_STEPS {
            let angle = i as f32 * (std::f64::consts::PI * 2.0) as f32 / 16.0;
            let sin_angle = Mth::sin(f64::from(angle));
            let cos_angle = Mth::cos(f64::from(angle));
            buffer_builder
                .add_vertex(sin_angle * 120.0, cos_angle * 120.0, -cos_angle * 40.0)
                .set_color(ring_color);
        }

        let mesh = buffer_builder.build_or_throw();
        device.create_buffer_with_data(
            Some("Sunrise/Sunset fan"),
            gpu_buffer::USAGE_VERTEX,
            mesh.vertex_buffer(),
        )
    }

    fn build_sun_quad(device: &dyn GpuDevice, atlas: &TextureAtlas) -> Rc<dyn GpuBuffer> {
        SkyRenderer::build_celestial_quad(
            device,
            "Sun quad",
            atlas.get_sprite(&Identifier::with_default_namespace("sun")),
        )
    }

    fn build_celestial_quad(
        device: &dyn GpuDevice,
        name: &str,
        sprite: &TextureAtlasSprite,
    ) -> Rc<dyn GpuBuffer> {
        let format = DefaultVertexFormat::POSITION_TEX;
        let mut byte_buffer_builder =
            ByteBufferBuilder::exactly_sized(4 * format.get_vertex_size());
        let mut buffer_builder =
            BufferBuilder::new(&mut byte_buffer_builder, PrimitiveTopology::QUADS, format);
        buffer_builder
            .add_vertex(-1.0, 0.0, -1.0)
            .set_uv(sprite.get_u0(), sprite.get_v0());
        buffer_builder
            .add_vertex(1.0, 0.0, -1.0)
            .set_uv(sprite.get_u1(), sprite.get_v0());
        buffer_builder
            .add_vertex(1.0, 0.0, 1.0)
            .set_uv(sprite.get_u1(), sprite.get_v1());
        buffer_builder
            .add_vertex(-1.0, 0.0, 1.0)
            .set_uv(sprite.get_u0(), sprite.get_v1());
        let mesh = buffer_builder.build_or_throw();
        device.create_buffer_with_data(Some(name), gpu_buffer::USAGE_VERTEX, mesh.vertex_buffer())
    }

    fn build_moon_phases(device: &dyn GpuDevice, atlas: &TextureAtlas) -> Rc<dyn GpuBuffer> {
        let phases = MoonPhase::VALUES;
        let format = DefaultVertexFormat::POSITION_TEX;
        let mut byte_buffer_builder =
            ByteBufferBuilder::exactly_sized(phases.len() as i32 * 4 * format.get_vertex_size());
        let mut buffer_builder =
            BufferBuilder::new(&mut byte_buffer_builder, PrimitiveTopology::QUADS, format);

        for phase in phases {
            let sprite = atlas.get_sprite(&Identifier::with_default_namespace(&format!(
                "moon/{}",
                phase.get_serialized_name()
            )));
            buffer_builder
                .add_vertex(-1.0, 0.0, -1.0)
                .set_uv(sprite.get_u1(), sprite.get_v1());
            buffer_builder
                .add_vertex(1.0, 0.0, -1.0)
                .set_uv(sprite.get_u0(), sprite.get_v1());
            buffer_builder
                .add_vertex(1.0, 0.0, 1.0)
                .set_uv(sprite.get_u0(), sprite.get_v0());
            buffer_builder
                .add_vertex(-1.0, 0.0, 1.0)
                .set_uv(sprite.get_u1(), sprite.get_v0());
        }

        let mesh = buffer_builder.build_or_throw();
        device.create_buffer_with_data(
            Some("Moon phases"),
            gpu_buffer::USAGE_VERTEX,
            mesh.vertex_buffer(),
        )
    }

    /// The stars' quads, from the same seed every time, and how many indices draw them. The
    /// negated comparisons are Java's, which a NaN would pass.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn build_stars(device: &dyn GpuDevice) -> (Rc<dyn GpuBuffer>, i32) {
        let mut random = SingleThreadedRandomSource::new(10842);
        let star_distance = 100.0;
        let mut byte_buffer_builder = ByteBufferBuilder::exactly_sized(
            DefaultVertexFormat::POSITION.get_vertex_size() * SkyRenderer::STAR_COUNT * 4,
        );
        let mut buffer_builder = BufferBuilder::new(
            &mut byte_buffer_builder,
            PrimitiveTopology::QUADS,
            DefaultVertexFormat::POSITION,
        );

        for _ in 0..SkyRenderer::STAR_COUNT {
            let x = random.next_float() * 2.0 - 1.0;
            let y = random.next_float() * 2.0 - 1.0;
            let z = random.next_float() * 2.0 - 1.0;
            let star_size = 0.15 + random.next_float() * 0.1;
            let length_sq = Mth::length_squared3_f32(x, y, z);
            if !(length_sq <= 0.010000001) && !(length_sq >= 1.0) {
                let star_center = *Vector3f::from_xyz(x, y, z).normalize_length(star_distance);
                let z_rot =
                    (random.next_double() * f64::from(std::f64::consts::PI as f32) * 2.0) as f32;
                let rotation = *Matrix3f::new()
                    .rotate_towards(
                        Vector3f::from_vector3f(&star_center).negate(),
                        &Vector3f::from_xyz(0.0, 1.0, 0.0),
                    )
                    .rotate_z(-z_rot);
                for (corner_x, corner_y) in [
                    (star_size, -star_size),
                    (star_size, star_size),
                    (-star_size, star_size),
                    (-star_size, -star_size),
                ] {
                    let corner = *Vector3f::from_xyz(corner_x, corner_y, 0.0)
                        .mul_matrix3f(&rotation)
                        .add(&star_center);
                    buffer_builder.add_vertex(corner.x, corner.y, corner.z);
                }
            }
        }

        let mesh = buffer_builder.build_or_throw();
        let star_index_count = mesh.draw_state().index_count;
        let buffer = device.create_buffer_with_data(
            Some("Stars vertex buffer"),
            gpu_buffer::USAGE_VERTEX | gpu_buffer::USAGE_COPY_DST,
            mesh.vertex_buffer(),
        );
        (buffer, star_index_count)
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
