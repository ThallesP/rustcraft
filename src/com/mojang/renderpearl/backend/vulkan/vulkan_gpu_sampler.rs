//! com/mojang/renderpearl/backend/vulkan/VulkanGpuSampler.java
//!
//! A `VkSampler`, its mipmaps linear unless the most detailed level is all there is (a maximum
//! level of detail of a quarter or less), anisotropic above 1. Its destruction is queued on the
//! encoder when it's dropped (Java's `close()`).

use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_const::VulkanConst;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_device::VulkanDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use ash::vk;
use std::any::Any;
use std::rc::Rc;

pub struct VulkanGpuSampler {
    vk_sampler: vk::Sampler,
    device: Rc<VulkanDevice>,
    address_mode_u: AddressMode,
    address_mode_v: AddressMode,
    min_filter: FilterMode,
    mag_filter: FilterMode,
    max_anisotropy: i32,
    max_lod: Option<f64>,
}

impl VulkanGpuSampler {
    pub fn new(
        device: &Rc<VulkanDevice>,
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        min_filter: FilterMode,
        mag_filter: FilterMode,
        max_anisotropy: i32,
        max_lod: Option<f64>,
    ) -> VulkanGpuSampler {
        let lod = max_lod.unwrap_or(1000.0);
        let create_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::from_raw(VulkanConst::to_vk_filter_mode(
                mag_filter,
            )))
            .min_filter(vk::Filter::from_raw(VulkanConst::to_vk_filter_mode(
                min_filter,
            )))
            .mipmap_mode(if lod > 0.25 {
                vk::SamplerMipmapMode::LINEAR
            } else {
                vk::SamplerMipmapMode::NEAREST
            })
            .address_mode_u(vk::SamplerAddressMode::from_raw(
                VulkanConst::to_vk_address_mode(address_mode_u),
            ))
            .address_mode_v(vk::SamplerAddressMode::from_raw(
                VulkanConst::to_vk_address_mode(address_mode_v),
            ))
            .mip_lod_bias(0.0)
            .max_lod(0.25f32.max(lod as f32))
            .anisotropy_enable(max_anisotropy > 1)
            .max_anisotropy(max_anisotropy as f32);
        // SAFETY: the create info lives for the call.
        let vk_sampler = VulkanUtils::crash_if_err(
            unsafe { device.vk_device().create_sampler(&create_info, None) },
            "Can't create sampler",
        );
        VulkanGpuSampler {
            vk_sampler,
            device: device.clone(),
            address_mode_u,
            address_mode_v,
            min_filter,
            mag_filter,
            max_anisotropy,
            max_lod,
        }
    }

    /// `(VulkanGpuSampler) sampler`. Panics where Java throws `ClassCastException`.
    pub fn of(sampler: &dyn GpuSampler) -> &VulkanGpuSampler {
        sampler
            .as_any()
            .downcast_ref::<VulkanGpuSampler>()
            .expect("a Vulkan sampler")
    }

    pub fn vk_sampler(&self) -> vk::Sampler {
        self.vk_sampler
    }
}

impl GpuSampler for VulkanGpuSampler {
    fn get_address_mode_u(&self) -> AddressMode {
        self.address_mode_u
    }

    fn get_address_mode_v(&self) -> AddressMode {
        self.address_mode_v
    }

    fn get_min_filter(&self) -> FilterMode {
        self.min_filter
    }

    fn get_mag_filter(&self) -> FilterMode {
        self.mag_filter
    }

    fn get_max_anisotropy(&self) -> i32 {
        self.max_anisotropy
    }

    fn get_max_lod(&self) -> Option<f64> {
        self.max_lod
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for VulkanGpuSampler {
    fn drop(&mut self) {
        let device = self.device.clone();
        let vk_sampler = self.vk_sampler;
        self.device
            .create_command_encoder()
            .queue_for_destroy(Box::new(move || {
                // SAFETY: the sampler is the device's, and the GPU is done with it.
                unsafe { device.vk_device().destroy_sampler(vk_sampler, None) }
            }));
    }
}
