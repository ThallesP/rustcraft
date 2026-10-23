//! net/minecraft/client/renderer/GlobalSettingsUniform.java

use std::rc::Rc;

use crate::com::mojang::blaze3d::buffers::std140_builder::Std140Builder;
use crate::com::mojang::blaze3d::buffers::std140_size_calculator::Std140SizeCalculator;
use crate::com::mojang::blaze3d::systems::render_system::RenderSystem;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// The `Globals` block every pass binds: the camera's block and its offset in it, the screen's
/// size, the time of day, and the texture filtering.
pub struct GlobalSettingsUniform {
    buffer: Rc<dyn GpuBuffer>,
}

impl GlobalSettingsUniform {
    pub const UBO_SIZE: i32 = Std140SizeCalculator::new()
        .put_ivec3()
        .put_float()
        .put_vec3()
        .put_float()
        .put_vec2()
        .put_int()
        .put_int()
        .get();

    pub fn new(device: &dyn GpuDevice) -> GlobalSettingsUniform {
        GlobalSettingsUniform {
            buffer: device.create_buffer(
                Some("Global Settings UBO"),
                gpu_buffer::USAGE_UNIFORM | gpu_buffer::USAGE_COPY_DST,
                i64::from(GlobalSettingsUniform::UBO_SIZE),
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &self,
        device: &dyn GpuDevice,
        width: i32,
        height: i32,
        glint_alpha: f64,
        game_time: i64,
        world_partial_ticks: f32,
        menu_blur_radius: i32,
        camera_pos: &Vec3,
        use_rgss: bool,
    ) {
        let camera_x = Mth::floor_f64(camera_pos.x);
        let camera_y = Mth::floor_f64(camera_pos.y);
        let camera_z = Mth::floor_f64(camera_pos.z);
        let mut builder = Std140Builder::on_stack(GlobalSettingsUniform::UBO_SIZE);
        builder
            .put_ivec3(camera_x, camera_y, camera_z)
            .put_float(glint_alpha as f32)
            .put_vec3(
                (f64::from(camera_x) - camera_pos.x) as f32,
                (f64::from(camera_y) - camera_pos.y) as f32,
                (f64::from(camera_z) - camera_pos.z) as f32,
            )
            .put_float(((game_time % 24000) as f32 + world_partial_ticks) / 24000.0)
            .put_vec2(width as f32, height as f32)
            .put_int(menu_blur_radius)
            .put_int(i32::from(use_rgss));
        device
            .create_command_encoder()
            .write_to_buffer(&gpu_buffer::slice(&self.buffer), &builder.get());
        RenderSystem::set_global_settings_uniform(Rc::clone(&self.buffer));
    }
}
