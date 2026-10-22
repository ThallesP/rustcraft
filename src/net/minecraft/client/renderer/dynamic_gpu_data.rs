//! net/minecraft/client/renderer/DynamicGpuData.java
//!
//! Only the uniform blocks' layouts so far: what `Transform` and `TerrainTransform` write, and the
//! blocks' sizes. The ring buffers each frame's blocks go in (`DynamicGpuDataStorageMapped`, and
//! `writeTransform`'s slices of them) come with the game's frame loop; until then a caller writes
//! a block into a buffer of its own.

use crate::com::mojang::blaze3d::buffers::std140_builder::Std140Builder;
use crate::com::mojang::blaze3d::buffers::std140_size_calculator::Std140SizeCalculator;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::vector3f::Vector3f;
use crate::org::joml::vector4f::Vector4f;

pub struct DynamicGpuData;

impl DynamicGpuData {
    pub const TRANSFORM_UBO_SIZE: i32 = Std140SizeCalculator::new()
        .put_mat4f()
        .put_mat4f()
        .put_vec4()
        .put_vec3()
        .get();
    pub const TERRAIN_TRANSFORM_UBO_SIZE: i32 =
        Std140SizeCalculator::new().put_mat4f().put_ivec2().get();
    pub const CHUNK_SECTION_UBO_SIZE: i32 =
        Std140SizeCalculator::new().put_ivec3().put_float().get();
}

/// `DynamicGpuData.Transform`: the `DynamicTransforms` block.
#[derive(Clone, Debug)]
pub struct Transform {
    pub model_view: Matrix4f,
    pub color_modulator: Vector4f,
    pub model_offset: Vector3f,
    pub texture_matrix: Matrix4f,
}

impl Transform {
    /// `writeTransform(Matrix4f)`'s: white, no offset, and the identity texture matrix.
    pub fn of_model_view(model_view: Matrix4f) -> Transform {
        Transform {
            model_view,
            color_modulator: Vector4f::from_xyzw(1.0, 1.0, 1.0, 1.0),
            model_offset: Vector3f::new(),
            texture_matrix: Matrix4f::new(),
        }
    }

    pub fn write(&self) -> Vec<u8> {
        let mut builder = Std140Builder::on_stack(DynamicGpuData::TRANSFORM_UBO_SIZE);
        let modulator = &self.color_modulator;
        let offset = &self.model_offset;
        builder
            .put_mat4f(&self.model_view)
            .put_mat4f(&self.texture_matrix)
            .put_vec4(modulator.x, modulator.y, modulator.z, modulator.w)
            .put_vec3(offset.x, offset.y, offset.z);
        builder.get()
    }
}

/// `DynamicGpuData.TerrainTransform`: the terrain's block.
#[derive(Clone, Debug)]
pub struct TerrainTransform {
    pub model_view: Matrix4f,
    pub texture_atlas_width: i32,
    pub texture_atlas_height: i32,
}

impl TerrainTransform {
    pub fn write(&self) -> Vec<u8> {
        let mut builder = Std140Builder::on_stack(DynamicGpuData::TERRAIN_TRANSFORM_UBO_SIZE);
        builder
            .put_mat4f(&self.model_view)
            .put_ivec2(self.texture_atlas_width, self.texture_atlas_height);
        builder.get()
    }
}
