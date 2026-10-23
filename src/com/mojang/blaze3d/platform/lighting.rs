//! com/mojang/blaze3d/platform/Lighting.java
//!
//! The directions of the two lights entity and item shaders shade by (the `Lighting` block), one
//! slot of a buffer for each use. Only the level's slot is written so far (`updateLevel`, the
//! dimension's cardinal lighting): the items', the UI's and the player skin preview's directions
//! come with the GUI that sets them up (they need JOML's `rotateYXZ`), and are zeros until then.

use std::rc::Rc;

use crate::com::mojang::blaze3d::buffers::std140_builder::Std140Builder;
use crate::com::mojang::blaze3d::buffers::std140_size_calculator::Std140SizeCalculator;
use crate::com::mojang::blaze3d::systems::render_system::RenderSystem;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::cardinal_lighting;
use crate::org::joml::vector3f::Vector3f;

/// `Lighting.Entry`: what a slot of the buffer lights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    LEVEL,
    ITEMS_FLAT,
    ITEMS_3D,
    ENTITY_IN_UI,
    PLAYER_SKIN,
}

impl Entry {
    pub const VALUES: [Entry; 5] = [
        Entry::LEVEL,
        Entry::ITEMS_FLAT,
        Entry::ITEMS_3D,
        Entry::ENTITY_IN_UI,
        Entry::PLAYER_SKIN,
    ];
}

pub struct Lighting {
    buffer: Rc<dyn GpuBuffer>,
    padded_size: i64,
}

fn normalized(x: f32, y: f32, z: f32) -> Vector3f {
    let mut vector = Vector3f::from_xyz(x, y, z);
    vector.normalize();
    vector
}

impl Lighting {
    pub const UBO_SIZE: i32 = Std140SizeCalculator::new().put_vec3().put_vec3().get();

    fn diffuse_light_0() -> Vector3f {
        normalized(0.2, 1.0, -0.7)
    }

    fn diffuse_light_1() -> Vector3f {
        normalized(-0.2, 1.0, 0.7)
    }

    fn nether_diffuse_light_0() -> Vector3f {
        normalized(0.2, 1.0, -0.7)
    }

    fn nether_diffuse_light_1() -> Vector3f {
        normalized(-0.2, -1.0, 0.7)
    }

    pub fn new(device: &dyn GpuDevice) -> Lighting {
        let padded_size = Mth::round_toward_i64(
            i64::from(Lighting::UBO_SIZE),
            i64::from(device.get_device_info().limits.min_uniform_offset_alignment),
        );
        let buffer = device.create_buffer(
            Some("Lighting UBO"),
            gpu_buffer::USAGE_UNIFORM | gpu_buffer::USAGE_COPY_DST,
            padded_size * Entry::VALUES.len() as i64,
        );
        Lighting {
            buffer,
            padded_size,
        }
    }

    /// The level's slot, for its dimension's cardinal lighting.
    pub fn update_level(&self, device: &dyn GpuDevice, r#type: cardinal_lighting::Type) {
        match r#type {
            cardinal_lighting::Type::DEFAULT => self.update_buffer(
                device,
                Entry::LEVEL,
                &Lighting::diffuse_light_0(),
                &Lighting::diffuse_light_1(),
            ),
            cardinal_lighting::Type::NETHER => self.update_buffer(
                device,
                Entry::LEVEL,
                &Lighting::nether_diffuse_light_0(),
                &Lighting::nether_diffuse_light_1(),
            ),
        }
    }

    fn update_buffer(
        &self,
        device: &dyn GpuDevice,
        entry: Entry,
        light0: &Vector3f,
        light1: &Vector3f,
    ) {
        let mut builder = Std140Builder::on_stack(Lighting::UBO_SIZE);
        builder
            .put_vec3(light0.x, light0.y, light0.z)
            .put_vec3(light1.x, light1.y, light1.z);
        device.create_command_encoder().write_to_buffer(
            &gpu_buffer::slice_range(
                &self.buffer,
                entry as i64 * self.padded_size,
                self.padded_size,
            ),
            &builder.get(),
        );
    }

    pub fn setup_for(&self, entry: Entry) {
        RenderSystem::set_shader_lights(gpu_buffer::slice_range(
            &self.buffer,
            entry as i64 * self.padded_size,
            i64::from(Lighting::UBO_SIZE),
        ));
    }
}
