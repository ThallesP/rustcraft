//! net/minecraft/client/renderer/Lightmap.java
//!
//! The 16×16 texture terrain and entities look their light up in, by block light and sky light,
//! drawn each frame its state changed by `pipeline/lightmap`. Java writes its block into a ring of
//! mapped buffers (`MappableRingBuffer`); here each update makes a buffer of its own, which the
//! shader reads the same.

use std::rc::Rc;

use crate::com::mojang::blaze3d::buffers::std140_builder::Std140Builder;
use crate::com::mojang::blaze3d::buffers::std140_size_calculator::Std140SizeCalculator;
use crate::com::mojang::blaze3d::systems::render_system::RenderSystem;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::{self, GpuTexture};
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::state::lightmap_render_state::LightmapRenderState;
use crate::net::minecraft::util::argb::ARGB;

pub struct Lightmap {
    texture: Rc<dyn GpuTexture>,
    texture_view: GpuTextureView,
}

impl Lightmap {
    pub const TEXTURE_SIZE: i32 = 16;
    const LIGHTMAP_UBO_SIZE: i32 = Std140SizeCalculator::new()
        .put_float()
        .put_float()
        .put_float()
        .put_float()
        .put_float()
        .put_float()
        .put_vec3()
        .put_vec3()
        .put_vec3()
        .put_vec3()
        .get();

    pub fn new(device: &dyn GpuDevice) -> Lightmap {
        let texture = device.create_texture(
            Some("Lightmap"),
            gpu_texture::USAGE_COPY_DST
                | gpu_texture::USAGE_TEXTURE_BINDING
                | gpu_texture::USAGE_RENDER_ATTACHMENT,
            GpuFormat::RGBA8_UNORM,
            16,
            16,
            1,
            1,
        );
        let texture_view = device.create_texture_view(&texture);
        device
            .create_command_encoder()
            .clear_color_texture(&*texture, &ARGB::vector4f_from_argb32(-1));
        Lightmap {
            texture,
            texture_view,
        }
    }

    pub fn get_texture_view(&self) -> &GpuTextureView {
        &self.texture_view
    }

    pub fn texture(&self) -> &Rc<dyn GpuTexture> {
        &self.texture
    }

    pub fn render(
        &self,
        device: &dyn GpuDevice,
        pipeline: &Rc<dyn CompiledRenderPipeline>,
        render_state: &LightmapRenderState,
    ) {
        if !render_state.needs_update {
            return;
        }

        let mut builder = Std140Builder::on_stack(Lightmap::LIGHTMAP_UBO_SIZE);
        let vec3 = |builder: &mut Std140Builder, v: &crate::org::joml::vector3f::Vector3f| {
            builder.put_vec3(v.x, v.y, v.z);
        };
        builder
            .put_float(render_state.sky_factor)
            .put_float(render_state.block_factor)
            .put_float(render_state.night_vision_effect_intensity)
            .put_float(render_state.darkness_effect_scale)
            .put_float(render_state.boss_overlay_world_darkening)
            .put_float(render_state.brightness);
        vec3(&mut builder, &render_state.block_light_tint);
        vec3(&mut builder, &render_state.sky_light_color);
        vec3(&mut builder, &render_state.ambient_color);
        vec3(&mut builder, &render_state.night_vision_color);
        let ubo = device.create_buffer_with_data(
            Some("Lightmap UBO"),
            gpu_buffer::USAGE_UNIFORM,
            &builder.get(),
        );
        let encoder = device.create_command_encoder();
        let mut render_pass =
            encoder.create_render_pass_with("Update light", &self.texture_view, None, None, None);
        render_pass.set_pipeline(pipeline);
        RenderSystem::bind_default_uniforms(&mut *render_pass);
        render_pass.set_uniform_buffer("LightmapInfo", &ubo);
        render_pass.draw(3, 1, 0, 0);
    }
}
