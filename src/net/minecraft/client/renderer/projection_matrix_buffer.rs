//! net/minecraft/client/renderer/ProjectionMatrixBuffer.java
//!
//! The `Projection` block's buffer: a projection's matrix, written again only when the projection
//! or its matrix changed. Java compares projections by reference; the port keeps the version the
//! buffer holds and whether it came from a projection (`getBuffer(Matrix4f)` forgets it), so a
//! caller passes the same projection each time, as each of Java's owners does.

use crate::com::mojang::blaze3d::buffers::std140_builder::Std140Builder;
use crate::com::mojang::blaze3d::systems::render_system::RenderSystem;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::net::minecraft::client::renderer::projection::Projection;
use crate::org::joml::matrix4f::Matrix4f;
use std::rc::Rc;

pub struct ProjectionMatrixBuffer {
    buffer: Rc<dyn GpuBuffer>,
    buffer_slice: GpuBufferSlice,
    has_uploaded_projection: bool,
    projection_matrix_version: i64,
    temp_matrix: Matrix4f,
}

impl ProjectionMatrixBuffer {
    pub fn new(device: &dyn GpuDevice, name: &str) -> ProjectionMatrixBuffer {
        let buffer = device.create_buffer(
            Some(&format!("Camera projection matrix UBO {name}")),
            gpu_buffer::USAGE_UNIFORM | gpu_buffer::USAGE_COPY_DST,
            i64::from(RenderSystem::PROJECTION_MATRIX_UBO_SIZE),
        );
        let buffer_slice = gpu_buffer::slice_range(
            &buffer,
            0,
            i64::from(RenderSystem::PROJECTION_MATRIX_UBO_SIZE),
        );
        ProjectionMatrixBuffer {
            buffer,
            buffer_slice,
            has_uploaded_projection: false,
            projection_matrix_version: -1,
            temp_matrix: Matrix4f::new(),
        }
    }

    /// `getBuffer(Projection)`. The matrix's depth range is the device's (`isZZeroToOne`).
    pub fn get_buffer(
        &mut self,
        device: &dyn GpuDevice,
        projection: &mut Projection,
    ) -> GpuBufferSlice {
        let z_zero_to_one = device.get_device_info().is_z_zero_to_one;
        if self.has_uploaded_projection
            && projection.get_matrix_version() == self.projection_matrix_version
        {
            return self.buffer_slice.clone();
        }
        self.has_uploaded_projection = true;
        self.projection_matrix_version = projection.get_matrix_version();
        let mut matrix = std::mem::take(&mut self.temp_matrix);
        projection.get_matrix(&mut matrix, z_zero_to_one);
        let slice = self.write_buffer(device, &matrix);
        self.temp_matrix = matrix;
        slice
    }

    /// `getBuffer(Matrix4f)`.
    pub fn get_buffer_matrix(
        &mut self,
        device: &dyn GpuDevice,
        projection_matrix: &Matrix4f,
    ) -> GpuBufferSlice {
        self.has_uploaded_projection = false;
        self.projection_matrix_version = -1;
        self.write_buffer(device, projection_matrix)
    }

    fn write_buffer(
        &mut self,
        device: &dyn GpuDevice,
        projection_matrix: &Matrix4f,
    ) -> GpuBufferSlice {
        let mut builder = Std140Builder::on_stack(RenderSystem::PROJECTION_MATRIX_UBO_SIZE);
        builder.put_mat4f(projection_matrix);
        device
            .create_command_encoder()
            .write_to_buffer(&gpu_buffer::slice(&self.buffer), &builder.get());
        self.buffer_slice.clone()
    }
}
