//! com/mojang/renderpearl/backend/vulkan/VulkanDevice.java
//!
//! The device and what it's made of: the instance, the `VkDevice` with the extensions' functions
//! the backend calls, VMA's allocator, the queues, the `DeviceInfo` the game reads (Vulkan's depth
//! from 0 to 1, the device's limits, its extensions as a Java `HashSet` of the instance's, marked
//! `(I)`, and the device's, marked `(D)`, in that set's order) and the command encoder, made last
//! like Java's. Java's `GpuDeviceBackend` is the API's `GpuDevice` here until the frontend's
//! device is ported (as the GL backend's); it's implemented on `Rc<VulkanDevice>`, which every
//! object the device makes holds, to queue its destruction on the encoder when it's dropped
//! (Java's `close()`). Timestamps and their query pools (`createTimestampQueryPool`,
//! `getTimestampCalibrationOffset`) come with the profiler, which nothing ported reads.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::commands::command_encoder::CommandEncoder;
use crate::com::mojang::renderpearl::api::device::device_features::DeviceFeatures;
use crate::com::mojang::renderpearl::api::device::device_info::DeviceInfo;
use crate::com::mojang::renderpearl::api::device::device_limits::DeviceLimits;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::device::gpu_surface::GpuSurface;
use crate::com::mojang::renderpearl::api::device::hints_and_workarounds::HintsAndWorkarounds;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::{
    BackendRenderPipeline, CreateInfo,
};
use crate::com::mojang::renderpearl::backend::vulkan::checkpoints::checkpoint_extension::CheckpointExtension;
use crate::com::mojang::renderpearl::backend::vulkan::init::feature_set::FeatureSet;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_command_encoder::VulkanCommandEncoder;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_feature_sets::{
    MULTI_DRAW_FEATURESET, WIREFRAME_FEATURESET,
};
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_buffer::VulkanGpuBuffer;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_sampler::VulkanGpuSampler;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_surface::VulkanGpuSurface;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_texture::VulkanGpuTexture;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_instance::VulkanInstance;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_physical_device::VulkanPhysicalDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_queue::VulkanQueue;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_render_pipeline::VulkanRenderPipeline;
use crate::java::util::hash_set::HashSet;
use crate::net::minecraft::util::util::Util;
use crate::org::lwjgl::sdl::sdl_video::SdlWindow;
use crate::org::lwjgl::util::vma::vma::{Vma, VmaAllocator};
use ash::vk;
use std::cell::OnceCell;
use std::rc::Rc;

pub struct VulkanDevice {
    instance: VulkanInstance,
    physical_device: vk::PhysicalDevice,
    vk_device: ash::Device,
    vma: VmaAllocator,
    device_info: DeviceInfo,
    graphics_queue: VulkanQueue,
    compute_queue: VulkanQueue,
    transfer_queue: VulkanQueue,
    is_integrated_intel_molten_vk: bool,
    enabled_features: FeatureSet,
    command_encoder: OnceCell<VulkanCommandEncoder>,
    checkpoint_extension: Box<dyn CheckpointExtension>,
    /// The extensions' functions: Java's LWJGL classes (`KHRPushDescriptor`, …) read them from
    /// the device's capabilities.
    push_descriptor: ash::khr::push_descriptor::Device,
    dynamic_rendering: ash::khr::dynamic_rendering::Device,
    synchronization2: ash::khr::synchronization2::Device,
    swapchain: ash::khr::swapchain::Device,
    surface: ash::khr::surface::Instance,
}

impl VulkanDevice {
    pub fn new(
        instance: VulkanInstance,
        physical_device: VulkanPhysicalDevice,
        enabled_feature_set: FeatureSet,
        vk_device: ash::Device,
        vma: VmaAllocator,
        checkpoint_extension: Box<dyn CheckpointExtension>,
    ) -> Rc<VulkanDevice> {
        let mut extension_names = HashSet::new();
        for name in instance.get_enabled_extensions().iter() {
            extension_names.add(format!("{name} (I)"));
        }
        for name in &enabled_feature_set.extensions {
            extension_names.add(format!("{name} (D)"));
        }

        let limits = physical_device.vk_physical_device_properties().limits;
        let vk11_properties = physical_device.vk_physical_device_vulkan11_properties();
        let indirect_draw_count = limits.max_draw_indirect_count.min(i32::MAX as u32) as i32;
        let max_multi_draw_count = physical_device
            .vk_physical_device_multi_draw_properties_ext()
            .max_multi_draw_count as i32;
        let max_memory_allocation_size = vk11_properties.max_memory_allocation_size as i64;
        let device_info = DeviceInfo {
            name: physical_device.device_name(),
            vendor_name: physical_device.vendor_name(),
            driver_info: physical_device.driver_info(),
            is_z_zero_to_one: true,
            backend_name: "Vulkan".to_owned(),
            timestamp_period: limits.timestamp_period,
            limits: DeviceLimits {
                max_anisotropy: limits.max_sampler_anisotropy as i32,
                min_uniform_offset_alignment: limits.min_uniform_buffer_offset_alignment as i32,
                max_texture_size: limits.max_image_dimension2_d as i32,
                max_memory_allocation_size: if max_memory_allocation_size < 0 {
                    i64::MAX
                } else {
                    max_memory_allocation_size
                },
                max_multi_draw_direct_interleaved_draw_count: if max_multi_draw_count < 0 {
                    i32::MAX
                } else {
                    max_multi_draw_count
                },
                max_color_attachments: limits.max_color_attachments as i32,
                max_draw_indirect_draw_count: indirect_draw_count,
            },
            features: DeviceFeatures {
                wireframe_fill_mode: enabled_feature_set.contains(&WIREFRAME_FEATURESET),
                shader_draw_parameters: true,
                multi_draw_direct_interleaved: enabled_feature_set.contains(&MULTI_DRAW_FEATURESET),
                multi_draw_direct_separate: false,
                multi_draw_indirect: true,
                draw_indirect: true,
                non_zero_first_instance: true,
                persistent_mapping: true,
            },
            underlying_extensions: extension_names.iter().cloned().collect(),
            hints_and_workarounds: HintsAndWorkarounds {
                write_to_buffer_is_slow: false,
                anisotropy_has_known_issues: false,
                is_explicit_depth_required: Util::is_apple_silicon_mac(
                    &physical_device.device_name(),
                ),
                multi_draw_indirect_has_known_issues: false,
            },
            device_type: physical_device.device_type(),
        };

        let graphics_queue_family = physical_device
            .graphics_queue_family_and_index()
            .expect("a graphics queue");
        let graphics_queue =
            VulkanQueue::new(&vk_device, graphics_queue_family.0, graphics_queue_family.1);
        let compute_queue = physical_device
            .compute_queue_family_and_index()
            .map_or(graphics_queue, |(family, index)| {
                VulkanQueue::new(&vk_device, family, index)
            });
        let transfer_queue = physical_device
            .transfer_queue_family_and_index()
            .map_or(compute_queue, |(family, index)| {
                VulkanQueue::new(&vk_device, family, index)
            });
        let properties = physical_device.vk_physical_device_properties();
        let is_integrated_intel_molten_vk = properties.device_type
            == vk::PhysicalDeviceType::INTEGRATED_GPU
            && properties.vendor_id == 32902
            && physical_device
                .vk_physical_device_driver_properties()
                .driver_id
                == vk::DriverId::MOLTENVK;

        let vk_instance = instance.vk_instance();
        instance.debug().attach_device(vk_instance, &vk_device);
        let device = Rc::new(VulkanDevice {
            physical_device: physical_device.vk_physical_device(),
            push_descriptor: ash::khr::push_descriptor::Device::new(vk_instance, &vk_device),
            dynamic_rendering: ash::khr::dynamic_rendering::Device::new(vk_instance, &vk_device),
            synchronization2: ash::khr::synchronization2::Device::new(vk_instance, &vk_device),
            swapchain: ash::khr::swapchain::Device::new(vk_instance, &vk_device),
            surface: ash::khr::surface::Instance::new(instance.entry(), vk_instance),
            instance,
            vk_device,
            vma,
            device_info,
            graphics_queue,
            compute_queue,
            transfer_queue,
            is_integrated_intel_molten_vk,
            enabled_features: enabled_feature_set,
            command_encoder: OnceCell::new(),
            checkpoint_extension,
        });
        let encoder = VulkanCommandEncoder::new(&device);
        let _ = device.command_encoder.set(encoder);
        device
    }

    /// `close()`.
    pub fn close(&self) {
        self.create_command_encoder().destroy();
        Vma::vmaDestroyAllocator(self.vma);
        // SAFETY: everything made from the device is gone.
        unsafe { self.vk_device.destroy_device(None) };
        self.instance.close();
    }

    pub fn instance(&self) -> &VulkanInstance {
        &self.instance
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn vk_device(&self) -> &ash::Device {
        &self.vk_device
    }

    pub fn graphics_queue(&self) -> &VulkanQueue {
        &self.graphics_queue
    }

    pub fn compute_queue(&self) -> &VulkanQueue {
        &self.compute_queue
    }

    pub fn transfer_queue(&self) -> &VulkanQueue {
        &self.transfer_queue
    }

    pub fn vma(&self) -> VmaAllocator {
        self.vma
    }

    pub fn push_descriptor(&self) -> &ash::khr::push_descriptor::Device {
        &self.push_descriptor
    }

    pub fn dynamic_rendering(&self) -> &ash::khr::dynamic_rendering::Device {
        &self.dynamic_rendering
    }

    pub fn synchronization2(&self) -> &ash::khr::synchronization2::Device {
        &self.synchronization2
    }

    pub fn swapchain(&self) -> &ash::khr::swapchain::Device {
        &self.swapchain
    }

    pub fn surface(&self) -> &ash::khr::surface::Instance {
        &self.surface
    }

    /// `createCommandEncoder()`: the device's one encoder.
    pub fn create_command_encoder(&self) -> &VulkanCommandEncoder {
        self.command_encoder
            .get()
            .expect("the encoder is made with the device")
    }

    pub fn checkpoint_extension(&self) -> &dyn CheckpointExtension {
        &*self.checkpoint_extension
    }

    pub fn is_debugging_enabled(&self) -> bool {
        self.instance.debug().enabled()
    }

    pub fn is_integrated_intel_molten_vk(&self) -> bool {
        self.is_integrated_intel_molten_vk
    }

    pub fn enabled_features(&self) -> &FeatureSet {
        &self.enabled_features
    }
}

impl GpuDevice for Rc<VulkanDevice> {
    fn create_command_encoder(&self) -> &dyn CommandEncoder {
        VulkanDevice::create_command_encoder(self)
    }

    fn create_surface(&self, window: SdlWindow) -> Box<dyn GpuSurface + '_> {
        Box::new(VulkanGpuSurface::new(self, window))
    }

    fn create_texture(
        &self,
        label: Option<&str>,
        usage: i32,
        format: GpuFormat,
        width: i32,
        height: i32,
        depth_or_layers: i32,
        mip_levels: i32,
    ) -> Rc<dyn GpuTexture> {
        let label = match label {
            Some(label) if self.is_debugging_enabled() => label,
            _ => "",
        };
        Rc::new(VulkanGpuTexture::new(
            self,
            usage,
            label,
            format,
            width,
            height,
            depth_or_layers,
            mip_levels,
        ))
    }

    /// `createTextureView`: the view's image view is the texture's (`VulkanGpuTexture`).
    fn create_texture_view_mips(
        &self,
        texture: &Rc<dyn GpuTexture>,
        base_mip_level: i32,
        mip_levels: i32,
    ) -> GpuTextureView {
        GpuTextureView::new(texture.clone(), base_mip_level, mip_levels)
    }

    fn create_buffer(&self, label: Option<&str>, usage: i32, size: i64) -> Rc<dyn GpuBuffer> {
        Rc::new(VulkanGpuBuffer::direct(
            self,
            label,
            usage,
            size,
            self.is_integrated_intel_molten_vk,
        ))
    }

    /// `createBuffer(label, usage, data)`: a buffer the data is copied into from the transient
    /// memory, in the commands that set up new objects.
    fn create_buffer_with_data(
        &self,
        label: Option<&str>,
        usage: i32,
        data: &[u8],
    ) -> Rc<dyn GpuBuffer> {
        let buffer = VulkanGpuBuffer::direct(
            self,
            label,
            usage | gpu_buffer::USAGE_COPY_DST,
            data.len() as i64,
            self.is_integrated_intel_molten_vk,
        );
        let encoder = VulkanDevice::create_command_encoder(self);
        let staging_buffer = encoder.transient_memory().upload_staging(&[data], 1, 16);
        let regions = [vk::BufferCopy::default()
            .src_offset(staging_buffer.offset as u64)
            .dst_offset(0)
            .size(data.len() as u64)];
        // SAFETY: both buffers are the device's, and the init buffer is recording.
        unsafe {
            self.vk_device.cmd_copy_buffer(
                encoder.object_init_command_buffer(),
                VulkanGpuBuffer::of(&*staging_buffer.buffer).vk_buffer(),
                buffer.vk_buffer(),
                &regions,
            );
        }
        Rc::new(buffer)
    }

    fn create_sampler(
        &self,
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        min_filter: FilterMode,
        mag_filter: FilterMode,
        max_anisotropy: i32,
        max_lod: Option<f64>,
    ) -> Rc<dyn GpuSampler> {
        let max_supported_anisotropy = self.device_info.limits.max_anisotropy;
        assert!(
            max_anisotropy >= 1 && max_anisotropy <= max_supported_anisotropy,
            "maxAnisotropy out of range; must be >= 1 and <= {max_supported_anisotropy}, but was {max_anisotropy}"
        );
        Rc::new(VulkanGpuSampler::new(
            self,
            address_mode_u,
            address_mode_v,
            min_filter,
            mag_filter,
            max_anisotropy,
            max_lod,
        ))
    }

    fn compile_pipeline(&self, create_info: &CreateInfo) -> Option<Rc<dyn BackendRenderPipeline>> {
        Some(Rc::new(VulkanRenderPipeline::compile(self, create_info)))
    }

    fn get_device_info(&self) -> &DeviceInfo {
        &self.device_info
    }
}
