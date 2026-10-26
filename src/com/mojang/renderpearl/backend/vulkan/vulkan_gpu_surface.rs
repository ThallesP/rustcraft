//! com/mojang/renderpearl/backend/vulkan/VulkanGpuSurface.java
//!
//! A window's surface and its swapchain of at least three 8-bit sRGB-space images: an image
//! acquired, the frame blitted into it upside down (Vulkan's first row is the frame's top, the
//! main target's first row its bottom), and presented once the submit the blit is in has run.
//!
//! The API's surface has no `acquireNextTexture` or configuration of a size yet (the GL backend
//! needs neither), so the swapchain takes the window's size in pixels when it's configured, and
//! `blit_from_texture` acquires the next image first, configuring again when the window's size
//! changed or the swapchain went out of date. A surface error (Java's `SurfaceException`, which
//! the game logs and recovers from by configuring again) panics.

use crate::com::mojang::renderpearl::api::device::gpu_surface::{GpuSurface, PresentMode};
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_const::VulkanConst;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_device::VulkanDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_gpu_texture::VulkanGpuTexture;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use crate::org::lwjgl::sdl::sdl_error::SDLError;
use crate::org::lwjgl::sdl::sdl_video::{SDLVideo, SdlWindow};
use crate::org::lwjgl::sdl::sdl_vulkan::SDLVulkan;
use ash::vk;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct VulkanGpuSurface {
    device: Rc<VulkanDevice>,
    window: SdlWindow,
    present_queue: vk::Queue,
    surface: vk::SurfaceKHR,
    swapchain_image_format: vk::Format,
    swapchain: Cell<vk::SwapchainKHR>,
    swapchain_width: Cell<i32>,
    swapchain_height: Cell<i32>,
    swapchain_images: RefCell<Vec<vk::Image>>,
    acquire_semaphores: [vk::Semaphore; 2],
    current_acquire_semaphore: Cell<usize>,
    present_semaphores: RefCell<Vec<vk::Semaphore>>,
    current_image_index: Cell<Option<u32>>,
    swapchain_out_of_date: Cell<bool>,
    present_mode: Cell<PresentMode>,
    supported_present_modes: Vec<PresentMode>,
}

impl VulkanGpuSurface {
    /// Panics where Java throws `IllegalStateException`, when SDL can't make the surface.
    pub fn new(device: &Rc<VulkanDevice>, window: SdlWindow) -> VulkanGpuSurface {
        let instance = device.instance().vk_instance();
        let surface = SDLVulkan::SDL_Vulkan_CreateSurface(window, instance.handle())
            .unwrap_or_else(|| {
                panic!(
                    "Failed to create window surface: {}",
                    SDLError::SDL_GetError().unwrap_or_default()
                )
            });
        let surface_functions = device.surface();
        let physical_device = device.physical_device();
        // SAFETY: the surface is the instance's, made for the device's window.
        let present_modes = VulkanUtils::crash_if_err(
            unsafe {
                surface_functions
                    .get_physical_device_surface_present_modes(physical_device, surface)
            },
            "Failed to enumerate surface present modes",
        );
        // `EnumSet`: in the enum's order.
        let supported_present_modes = [
            PresentMode::IMMEDIATE,
            PresentMode::MAILBOX,
            PresentMode::FIFO,
            PresentMode::FIFO_RELAXED,
        ]
        .into_iter()
        .filter(|mode| {
            present_modes
                .iter()
                .any(|supported| supported.as_raw() == VulkanConst::to_vk_present_mode(*mode))
        })
        .collect();
        // SAFETY: as above.
        let formats = unsafe {
            surface_functions.get_physical_device_surface_formats(physical_device, surface)
        }
        .unwrap_or_default();
        let swapchain_image_format = VulkanGpuSurface::pick_swapchain_surface_format(&formats);
        let semaphore = || {
            // SAFETY: the create info lives for the call.
            VulkanUtils::crash_if_err(
                unsafe {
                    device
                        .vk_device()
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                },
                "Failed to create VkSemaphore(binary)",
            )
        };
        VulkanGpuSurface {
            device: device.clone(),
            window,
            present_queue: device.graphics_queue().vk_queue(),
            surface,
            swapchain_image_format,
            swapchain: Cell::new(vk::SwapchainKHR::null()),
            swapchain_width: Cell::new(0),
            swapchain_height: Cell::new(0),
            swapchain_images: RefCell::new(Vec::new()),
            acquire_semaphores: [semaphore(), semaphore()],
            current_acquire_semaphore: Cell::new(0),
            present_semaphores: RefCell::new(Vec::new()),
            current_image_index: Cell::new(None),
            swapchain_out_of_date: Cell::new(true),
            present_mode: Cell::new(PresentMode::FIFO),
            supported_present_modes,
        }
    }

    /// The first 8-bit RGBA or BGRA format in sRGB's color space. Panics where Java throws.
    fn pick_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::Format {
        formats
            .iter()
            .find(|format| {
                format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                    && (format.format == vk::Format::R8G8B8A8_UNORM
                        || format.format == vk::Format::B8G8R8A8_UNORM)
            })
            .map(|format| format.format)
            .expect("Could not find compatible swapchain format")
    }

    fn destroy_swapchain(&self) {
        let swapchain = self.swapchain.get();
        if swapchain == vk::SwapchainKHR::null() {
            return;
        }
        let vk_device = self.device.vk_device();
        self.device.graphics_queue().wait_idle(vk_device);
        // SAFETY: the GPU is done with the swapchain and its semaphores.
        unsafe {
            self.device.swapchain().destroy_swapchain(swapchain, None);
            for semaphore in self.present_semaphores.take() {
                vk_device.destroy_semaphore(semaphore, None);
            }
        }
        self.swapchain.set(vk::SwapchainKHR::null());
    }

    /// `configure`: a swapchain of the window's size in pixels. Panics where Java throws
    /// `SurfaceException`.
    fn configure_size(&self, width: i32, height: i32) {
        self.destroy_swapchain();
        let physical_device = self.device.physical_device();
        // SAFETY: the surface is the device's instance's.
        let surface_capabilities = unsafe {
            self.device
                .surface()
                .get_physical_device_surface_capabilities(physical_device, self.surface)
        }
        .unwrap_or_else(|error| {
            panic!(
                "{}: Failed to get surface capabilities",
                VulkanUtils::result_to_string(error.as_raw())
            )
        });
        let min_extent = surface_capabilities.min_image_extent;
        let max_extent = surface_capabilities.max_image_extent;
        assert!(
            width as u32 >= min_extent.width
                && max_extent.width >= width as u32
                && height as u32 >= min_extent.height
                && max_extent.height >= height as u32,
            "Requested swapchain extent ({width} x {height}) not within allowed extent: min({} x {}) max({} x {})",
            min_extent.width,
            min_extent.height,
            max_extent.width,
            max_extent.height
        );
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(3.max(surface_capabilities.min_image_count))
            .image_format(self.swapchain_image_format)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(vk::Extent2D {
                width: width as u32,
                height: height as u32,
            })
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::from_raw(
                VulkanConst::to_vk_present_mode(self.present_mode.get()),
            ))
            .clipped(true);
        let swapchain_functions = self.device.swapchain();
        // SAFETY: the create info lives for the call.
        let swapchain =
            unsafe { swapchain_functions.create_swapchain(&swapchain_create_info, None) }
                .unwrap_or_else(|error| {
                    panic!(
                        "{}: Failed to create Swapchain",
                        VulkanUtils::result_to_string(error.as_raw())
                    )
                });
        self.swapchain.set(swapchain);
        // SAFETY: the swapchain is the device's.
        let images = unsafe { swapchain_functions.get_swapchain_images(swapchain) }.unwrap_or_else(
            |error| {
                panic!(
                    "{}: Failed to get swapchain images",
                    VulkanUtils::result_to_string(error.as_raw())
                )
            },
        );
        let present_semaphores = images
            .iter()
            .map(|_| {
                // SAFETY: the create info lives for the call.
                unsafe {
                    self.device
                        .vk_device()
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                }
                .expect("Failed to create VkSemaphore(binary)")
            })
            .collect();
        *self.swapchain_images.borrow_mut() = images;
        *self.present_semaphores.borrow_mut() = present_semaphores;
        self.swapchain_out_of_date.set(false);
        self.swapchain_width.set(width);
        self.swapchain_height.set(height);
    }

    /// `acquireNextTexture`: the next image, after configuring again where the window's size
    /// changed or the swapchain went out of date.
    fn acquire_next_texture(&self) {
        let (width, height) = SDLVideo::SDL_GetWindowSizeInPixels(self.window).unwrap_or((0, 0));
        if self.swapchain_out_of_date.get()
            || width != self.swapchain_width.get()
            || height != self.swapchain_height.get()
        {
            self.configure_size(width, height);
        }
        let current = (self.current_acquire_semaphore.get() + 1) % self.acquire_semaphores.len();
        self.current_acquire_semaphore.set(current);
        // SAFETY: the swapchain and semaphore are the device's.
        let result = unsafe {
            self.device.swapchain().acquire_next_image(
                self.swapchain.get(),
                5_000_000_000,
                self.acquire_semaphores[current],
                vk::Fence::null(),
            )
        };
        match result {
            Ok((index, _suboptimal)) => self.current_image_index.set(Some(index)),
            Err(vk::Result::TIMEOUT) => {
                let checkpoints = self
                    .device
                    .checkpoint_extension()
                    .retrieve_checkpoints(false);
                panic!(
                    "GPU timeout attempting to acquire next frame: {}",
                    VulkanUtils::format_checkpoints(&checkpoints)
                );
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.swapchain_out_of_date.set(true);
                self.acquire_next_texture();
            }
            Err(error) => VulkanUtils::crash_if_failure(error, "Failed to acquire image"),
        }
    }

    fn image_barrier(
        image: vk::Image,
        src: (vk::PipelineStageFlags2, vk::AccessFlags2),
        dst: (vk::PipelineStageFlags2, vk::AccessFlags2),
        layouts: (vk::ImageLayout, vk::ImageLayout),
    ) -> vk::ImageMemoryBarrier2<'static> {
        vk::ImageMemoryBarrier2::default()
            .src_stage_mask(src.0)
            .src_access_mask(src.1)
            .dst_stage_mask(dst.0)
            .dst_access_mask(dst.1)
            .old_layout(layouts.0)
            .new_layout(layouts.1)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
    }
}

impl GpuSurface for VulkanGpuSurface {
    /// The present mode the next swapchain is made with. Panics where Java's frontend throws
    /// `SurfaceException`, for one the surface doesn't support.
    fn configure(&self, present_mode: PresentMode) {
        assert!(
            self.supported_present_modes.contains(&present_mode),
            "Surface does not support present mode {present_mode:?} (supported: {:?})",
            self.supported_present_modes
        );
        self.present_mode.set(present_mode);
        self.swapchain_out_of_date.set(true);
    }

    fn supported_present_modes(&self) -> &[PresentMode] {
        &self.supported_present_modes
    }

    /// `blitFromTexture`: in a command buffer of its own after the frame's, waiting for the
    /// image's acquisition and signalling its presentation.
    fn blit_from_texture(&self, texture_view: &GpuTextureView) {
        if self.current_image_index.get().is_none() {
            self.acquire_next_texture();
        }
        let image_index = self.current_image_index.get().expect("an acquired image");
        let device = &self.device;
        let encoder = device.create_command_encoder();
        let blit_command_buffer = encoder.allocate_and_begin_transient_command_buffer();
        let swapchain_image = self.swapchain_images.borrow()[image_index as usize];
        let synchronization2 = device.synchronization2();
        let to_transfer = [VulkanGpuSurface::image_barrier(
            swapchain_image,
            (vk::PipelineStageFlags2::NONE, vk::AccessFlags2::NONE),
            (
                vk::PipelineStageFlags2::from_raw(4096),
                vk::AccessFlags2::from_raw(4096),
            ),
            (
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ),
        )];
        // SAFETY: the buffer is recording, and the images are the device's.
        unsafe {
            synchronization2.cmd_pipeline_barrier2(
                blit_command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_transfer),
            );
        }

        let copy_width = self.swapchain_width.get().min(texture_view.get_width(0));
        let copy_height = self.swapchain_height.get().min(texture_view.get_height(0));
        let subresource = |mip_level: i32| vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: mip_level as u32,
            base_array_layer: 0,
            layer_count: 1,
        };
        let blit_region = [vk::ImageBlit {
            src_subresource: subresource(texture_view.base_mip_level()),
            src_offsets: [
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: copy_width,
                    y: copy_height,
                    z: 1,
                },
            ],
            dst_subresource: subresource(0),
            dst_offsets: [
                vk::Offset3D {
                    x: 0,
                    y: copy_height,
                    z: 0,
                },
                vk::Offset3D {
                    x: copy_width,
                    y: 0,
                    z: 1,
                },
            ],
        }];
        // SAFETY: as above.
        unsafe {
            device.vk_device().cmd_blit_image(
                blit_command_buffer,
                VulkanGpuTexture::of(&**texture_view.texture()).vk_image(),
                vk::ImageLayout::GENERAL,
                swapchain_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &blit_region,
                vk::Filter::NEAREST,
            );
        }

        let to_present = [VulkanGpuSurface::image_barrier(
            swapchain_image,
            (
                vk::PipelineStageFlags2::from_raw(4096),
                vk::AccessFlags2::from_raw(4096),
            ),
            (
                vk::PipelineStageFlags2::ALL_COMMANDS,
                vk::AccessFlags2::NONE,
            ),
            (
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::PRESENT_SRC_KHR,
            ),
        )];
        let memory_barrier = [vk::MemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::from_raw(4096))
            .src_access_mask(vk::AccessFlags2::from_raw(2048))
            .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .dst_access_mask(vk::AccessFlags2::from_raw(98304))];
        // SAFETY: as above.
        unsafe {
            synchronization2.cmd_pipeline_barrier2(
                blit_command_buffer,
                &vk::DependencyInfo::default()
                    .memory_barriers(&memory_barrier)
                    .image_memory_barriers(&to_present),
            );
            let result = device.vk_device().end_command_buffer(blit_command_buffer);
            VulkanUtils::crash_if_err(result, "Failed to end VkCommandBuffer");
        }
        encoder.wait_semaphore(
            self.acquire_semaphores[self.current_acquire_semaphore.get()],
            0,
            vk::PipelineStageFlags2::ALL_COMMANDS,
        );
        encoder.execute(blit_command_buffer);
        encoder.signal_semaphore(
            self.present_semaphores.borrow()[image_index as usize],
            0,
            vk::PipelineStageFlags2::from_raw(4096),
        );
    }

    /// `present()`, after the submit the blit is in (Java's frame submits between the two).
    fn present(&self) {
        let Some(image_index) = self.current_image_index.take() else {
            return;
        };
        let wait_semaphores = [self.present_semaphores.borrow()[image_index as usize]];
        let swapchains = [self.swapchain.get()];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        // SAFETY: the semaphore's signal is submitted, and the info lives for the call.
        match unsafe {
            self.device
                .swapchain()
                .queue_present(self.present_queue, &present_info)
        } {
            Ok(_suboptimal) => {}
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.swapchain_out_of_date.set(true),
            Err(error) => VulkanUtils::crash_if_failure(error, "Failed to present image"),
        }
    }
}

impl Drop for VulkanGpuSurface {
    /// `close()`.
    fn drop(&mut self) {
        self.destroy_swapchain();
        let vk_device = self.device.vk_device();
        // SAFETY: the GPU is done with the semaphores and the surface.
        unsafe {
            for semaphore in self.acquire_semaphores {
                vk_device.destroy_semaphore(semaphore, None);
            }
            self.device.surface().destroy_surface(self.surface, None);
        }
    }
}
