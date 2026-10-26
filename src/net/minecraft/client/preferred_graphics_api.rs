//! net/minecraft/client/PreferredGraphicsApi.java
//!
//! The graphics API the game asks for: by default OpenGL then Vulkan, or Vulkan first when it's
//! asked for (`--graphicsBackend vulkan`, or the option). Its codec and caption come with the
//! options screen.

use crate::com::mojang::renderpearl::api::device::gpu_backend::GpuBackend;
use crate::com::mojang::renderpearl::backend::opengl::gl_backend::GlBackend;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_backend::VulkanBackend;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferredGraphicsApi {
    DEFAULT,
    OPENGL,
    VULKAN,
}

impl PreferredGraphicsApi {
    pub const VALUES: [PreferredGraphicsApi; 3] = [
        PreferredGraphicsApi::DEFAULT,
        PreferredGraphicsApi::OPENGL,
        PreferredGraphicsApi::VULKAN,
    ];

    pub fn get_serialized_name(self) -> &'static str {
        match self {
            PreferredGraphicsApi::DEFAULT => "default",
            PreferredGraphicsApi::OPENGL => "opengl",
            PreferredGraphicsApi::VULKAN => "vulkan",
        }
    }

    /// joptsimple's `EnumConverter`: the constant whose name the value is, in any case.
    pub fn value_of(value: &str) -> Option<PreferredGraphicsApi> {
        PreferredGraphicsApi::VALUES
            .into_iter()
            .find(|api| api.get_serialized_name().eq_ignore_ascii_case(value))
    }

    pub fn get_backends_to_try(self) -> Vec<Box<dyn GpuBackend>> {
        let gl: Box<dyn GpuBackend> = Box::new(GlBackend::new());
        let vulkan: Box<dyn GpuBackend> = Box::new(VulkanBackend::new());
        if self == PreferredGraphicsApi::VULKAN {
            vec![vulkan, gl]
        } else {
            vec![gl, vulkan]
        }
    }
}
