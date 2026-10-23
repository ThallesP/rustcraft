//! net/minecraft/client/renderer/fog/FogRenderer.java
//!
//! The fog's uniform block: none (`FogMode.NONE`, every distance at `Float.MAX_VALUE`) or the
//! world's, which `setupFog` works out each frame from the environment the camera is in. Java
//! writes the world's into a ring of mapped buffers (`MappableRingBuffer`); here each frame's is a
//! buffer of its own with the same bytes.

use std::rc::Rc;

use crate::com::mojang::blaze3d::buffers::std140_builder::Std140Builder;
use crate::com::mojang::blaze3d::buffers::std140_size_calculator::Std140SizeCalculator;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::java::lang::math::Math;
use crate::net::minecraft::client::renderer::fog::fog_data::FogData;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::material::fog_type::FogType;
use crate::org::joml::vector3f::Vector3f;
use crate::org::joml::vector4f::Vector4f;

/// `FogRenderer.FogMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FogMode {
    NONE,
    WORLD,
}

pub struct FogRenderer {
    empty_buffer: Rc<dyn GpuBuffer>,
    regular_buffer: Option<Rc<dyn GpuBuffer>>,
}

impl FogRenderer {
    pub const FOG_UBO_SIZE: i32 = Std140SizeCalculator::new()
        .put_vec4()
        .put_float()
        .put_float()
        .put_float()
        .put_float()
        .put_float()
        .put_float()
        .get();

    pub fn new(device: &dyn GpuDevice) -> FogRenderer {
        let data = FogRenderer::write(
            &Vector4f::from_scalar(0.0),
            f32::MAX,
            f32::MAX,
            f32::MAX,
            f32::MAX,
            f32::MAX,
            f32::MAX,
        );
        FogRenderer {
            empty_buffer: device.create_buffer_with_data(
                Some("Empty fog"),
                gpu_buffer::USAGE_UNIFORM,
                &data,
            ),
            regular_buffer: None,
        }
    }

    pub fn get_buffer(&self, mode: FogMode) -> GpuBufferSlice {
        let buffer = match mode {
            FogMode::NONE => &self.empty_buffer,
            FogMode::WORLD => self
                .regular_buffer
                .as_ref()
                .expect("the world's fog is written before it's read"),
        };
        gpu_buffer::slice_range(buffer, 0, i64::from(FogRenderer::FOG_UBO_SIZE))
    }

    /// `computeFogColor`: the camera's environment's colour (`color`), darkened in the void below
    /// the level (`voidDarknessOnsetRange`: within a block over a flat level's bottom, 32
    /// otherwise) but in lava and powder snow, and brightened toward its brightest channel by the
    /// player's water vision in water. The darkness and night vision effects and the boss
    /// overlay's darkening aren't ported, so none of them changes it.
    pub fn compute_fog_color(
        color: &Vector3f,
        camera_y: f64,
        min_y: i32,
        is_flat: bool,
        fog_type: FogType,
        water_vision: Option<f32>,
    ) -> Vector4f {
        let void_darkness_onset_range: f32 = if is_flat { 1.0 } else { 32.0 };
        let darkness = Mth::clamp_f32(
            (void_darkness_onset_range + min_y as f32 - camera_y as f32)
                / void_darkness_onset_range,
            0.0,
            1.0,
        );
        let (mut fog_red, mut fog_green, mut fog_blue) = (color.x, color.y, color.z);
        if darkness > 0.0 && fog_type != FogType::LAVA && fog_type != FogType::POWDER_SNOW {
            let brightness = Mth::square_f32(1.0 - darkness);
            fog_red *= brightness;
            fog_green *= brightness;
            fog_blue *= brightness;
        }

        let brighten_factor = if fog_type == FogType::WATER {
            water_vision.unwrap_or(1.0)
        } else {
            0.0
        };
        if fog_red != 0.0 && fog_green != 0.0 && fog_blue != 0.0 {
            let target_scale = 1.0 / Math::max_f32(fog_red, Math::max_f32(fog_green, fog_blue));
            fog_red = Mth::lerp_f32(brighten_factor, fog_red, fog_red * target_scale);
            fog_green = Mth::lerp_f32(brighten_factor, fog_green, fog_green * target_scale);
            fog_blue = Mth::lerp_f32(brighten_factor, fog_blue, fog_blue * target_scale);
        }

        Vector4f::from_xyzw(fog_red, fog_green, fog_blue, 1.0)
    }

    /// The world's fog for this frame.
    pub fn update_buffer(&mut self, device: &dyn GpuDevice, fog: &FogData) {
        let data = FogRenderer::write(
            &fog.color,
            fog.environmental_start,
            fog.environmental_end,
            fog.render_distance_start,
            fog.render_distance_end,
            fog.sky_end,
            fog.cloud_end,
        );
        self.regular_buffer =
            Some(device.create_buffer_with_data(Some("Fog UBO"), gpu_buffer::USAGE_UNIFORM, &data));
    }

    #[allow(clippy::too_many_arguments)]
    fn write(
        fog_color: &Vector4f,
        environmental_start: f32,
        environmental_end: f32,
        render_distance_start: f32,
        render_distance_end: f32,
        sky_end: f32,
        end_clouds: f32,
    ) -> Vec<u8> {
        let mut builder = Std140Builder::on_stack(FogRenderer::FOG_UBO_SIZE);
        builder
            .put_vec4(fog_color.x, fog_color.y, fog_color.z, fog_color.w)
            .put_float(environmental_start)
            .put_float(environmental_end)
            .put_float(render_distance_start)
            .put_float(render_distance_end)
            .put_float(sky_end)
            .put_float(end_clouds);
        builder.get()
    }
}
