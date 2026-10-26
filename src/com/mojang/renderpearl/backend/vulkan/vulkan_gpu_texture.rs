//! com/mojang/renderpearl/backend/vulkan/VulkanGpuTexture.java
//!
//! An image from VMA, optimally tiled, moved to the `GENERAL` layout it stays in by a barrier in
//! the commands that set up new objects. Its destruction is queued on the encoder when it's
//! dropped (Java's `close()` once no view is left), with its views'.
//!
//! The API's texture views are values here (`GpuTextureView`), so a view's `VkImageView`
//! (`VulkanGpuTextureView`) is the texture's: made the first time a view of those mip levels is
//! bound, and kept with the texture. Java makes one each time the game makes a view, and destroys
//! it when the view's closed; which image view a draw reads through changes nothing it reads.

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::{self, GpuTexture};
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_const::VulkanConst;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_device::VulkanDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_texture_view::VulkanGpuTextureView;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use crate::org::lwjgl::util::vma::vma::{Vma, VmaAllocation, VmaAllocationCreateInfo};
use ash::vk::{self, Handle};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub struct VulkanGpuTexture {
    device: Rc<VulkanDevice>,
    vk_image: vk::Image,
    vma_allocation: VmaAllocation,
    usage: i32,
    label: String,
    format: GpuFormat,
    width: i32,
    height: i32,
    depth_or_layers: i32,
    mip_levels: i32,
    /// The image views made so far, by their base mip level and count.
    views: RefCell<Vec<((i32, i32), vk::ImageView)>>,
}

impl VulkanGpuTexture {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &Rc<VulkanDevice>,
        usage: i32,
        label: &str,
        format: GpuFormat,
        width: i32,
        height: i32,
        depth_or_layers: i32,
        mip_levels: i32,
    ) -> VulkanGpuTexture {
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width: width as u32,
                height: height as u32,
                depth: 1,
            })
            .mip_levels(mip_levels as u32)
            .array_layers(depth_or_layers as u32)
            .format(vk::Format::from_raw(VulkanConst::to_vk_format(format)))
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::from_raw(
                VulkanConst::texture_usage_to_vk(usage, format) as u32,
            ))
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1)
            .flags(
                if VulkanUtils::has_any_bit(
                    i64::from(usage),
                    i64::from(gpu_texture::USAGE_CUBEMAP_COMPATIBLE),
                ) {
                    vk::ImageCreateFlags::CUBE_COMPATIBLE
                } else {
                    vk::ImageCreateFlags::empty()
                },
            );
        let allocation_create_info = VmaAllocationCreateInfo {
            usage: Vma::VMA_MEMORY_USAGE_AUTO_PREFER_DEVICE,
            ..VmaAllocationCreateInfo::default()
        };
        let (result, vk_image, vma_allocation) =
            Vma::vmaCreateImage(device.vma(), &image_create_info, &allocation_create_info);
        VulkanUtils::crash_if_failure(result, "Failed to create image");

        let barrier = [vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::from_raw(98304))
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(vk_image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: if format.has_color_aspect() {
                    vk::ImageAspectFlags::COLOR
                } else {
                    vk::ImageAspectFlags::DEPTH
                },
                base_mip_level: 0,
                level_count: mip_levels as u32,
                base_array_layer: 0,
                layer_count: depth_or_layers as u32,
            })];
        // SAFETY: the init buffer is recording, and the image is the device's.
        unsafe {
            device.vk_device().cmd_pipeline_barrier(
                device.create_command_encoder().object_init_command_buffer(),
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barrier,
            );
        }
        device
            .instance()
            .debug()
            .set_object_name(vk::ObjectType::IMAGE, vk_image.as_raw(), label);
        VulkanGpuTexture {
            device: device.clone(),
            vk_image,
            vma_allocation,
            usage,
            label: label.to_owned(),
            format,
            width,
            height,
            depth_or_layers,
            mip_levels,
            views: RefCell::new(Vec::new()),
        }
    }

    /// `(VulkanGpuTexture) texture`. Panics where Java throws `ClassCastException`.
    pub fn of(texture: &dyn GpuTexture) -> &VulkanGpuTexture {
        texture
            .as_any()
            .downcast_ref::<VulkanGpuTexture>()
            .expect("a Vulkan texture")
    }

    pub fn vk_image(&self) -> vk::Image {
        self.vk_image
    }

    /// The `VkImageView` of the view's mip levels (`VulkanGpuTextureView.vkImageView()`).
    pub fn image_view(view: &GpuTextureView) -> vk::ImageView {
        let texture = VulkanGpuTexture::of(&**view.texture());
        let key = (view.base_mip_level(), view.mip_levels());
        if let Some((_, image_view)) = texture
            .views
            .borrow()
            .iter()
            .find(|(known, _)| *known == key)
        {
            return *image_view;
        }
        let image_view = VulkanGpuTextureView::create(&texture.device, texture, key.0, key.1);
        texture.views.borrow_mut().push((key, image_view));
        image_view
    }
}

impl GpuTexture for VulkanGpuTexture {
    fn get_usage(&self) -> i32 {
        self.usage
    }

    fn get_label(&self) -> &str {
        &self.label
    }

    fn get_format(&self) -> GpuFormat {
        self.format
    }

    fn get_width(&self, mip_level: i32) -> i32 {
        self.width >> mip_level
    }

    fn get_height(&self, mip_level: i32) -> i32 {
        self.height >> mip_level
    }

    fn get_depth_or_layers(&self) -> i32 {
        self.depth_or_layers
    }

    fn get_mip_levels(&self) -> i32 {
        self.mip_levels
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for VulkanGpuTexture {
    /// `close()`, and every view's: their destruction queued for once the GPU is done with them.
    fn drop(&mut self) {
        let device = self.device.clone();
        let views: Vec<vk::ImageView> = self
            .views
            .take()
            .into_iter()
            .map(|(_, view)| view)
            .collect();
        let vk_image = self.vk_image;
        let vma_allocation = self.vma_allocation;
        self.device
            .create_command_encoder()
            .queue_for_destroy(Box::new(move || {
                for view in views {
                    VulkanGpuTextureView::destroy(&device, view);
                }
                Vma::vmaDestroyImage(device.vma(), vk_image, vma_allocation);
            }));
    }
}
