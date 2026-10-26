//! com/mojang/renderpearl/backend/vulkan/VulkanGpuTextureView.java
//!
//! A `VkImageView` of some of a texture's mip levels, a cube's six layers or one. The views are
//! the texture's here (`VulkanGpuTexture::image_view`), which makes and destroys them with these.

use crate::com::mojang::renderpearl::api::textures::gpu_texture::{self, GpuTexture};
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_const::VulkanConst;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_device::VulkanDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_texture::VulkanGpuTexture;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use ash::vk::{self, Handle};

pub struct VulkanGpuTextureView;

impl VulkanGpuTextureView {
    /// The constructor's image view.
    pub fn create(
        device: &VulkanDevice,
        texture: &VulkanGpuTexture,
        base_mip_level: i32,
        mip_levels: i32,
    ) -> vk::ImageView {
        let is_cubemap = texture.get_usage() & gpu_texture::USAGE_CUBEMAP_COMPATIBLE != 0;
        let format = texture.get_format();
        let image_view_create_info = vk::ImageViewCreateInfo::default()
            .image(texture.vk_image())
            .view_type(if is_cubemap {
                vk::ImageViewType::CUBE
            } else {
                vk::ImageViewType::TYPE_2D
            })
            .format(vk::Format::from_raw(VulkanConst::to_vk_format(format)))
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: if format.has_color_aspect() {
                    vk::ImageAspectFlags::COLOR
                } else {
                    vk::ImageAspectFlags::DEPTH
                },
                base_mip_level: base_mip_level as u32,
                level_count: mip_levels as u32,
                base_array_layer: 0,
                layer_count: if is_cubemap { 6 } else { 1 },
            });
        // SAFETY: the image is the device's, and the create info lives for the call.
        let vk_image_view = VulkanUtils::crash_if_err(
            unsafe {
                device
                    .vk_device()
                    .create_image_view(&image_view_create_info, None)
            },
            "Failed to create VkImageView",
        );
        device.instance().debug().set_object_name(
            vk::ObjectType::IMAGE_VIEW,
            vk_image_view.as_raw(),
            texture.get_label(),
        );
        vk_image_view
    }

    pub fn destroy(device: &VulkanDevice, vk_image_view: vk::ImageView) {
        // SAFETY: the view is the device's, and the GPU is done with it.
        unsafe { device.vk_device().destroy_image_view(vk_image_view, None) };
    }
}
