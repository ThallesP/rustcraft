//! org/lwjgl/sdl/SDLVulkan.java (LWJGL 3.4.3)
//!
//! SDL's Vulkan support, which the Vulkan backend loads the Vulkan loader through: the loader's
//! `vkGetInstanceProcAddr`, the instance extensions a window's surface needs, whether a queue
//! family can present, and a window's surface. Vulkan's handles are `ash`'s here.

use crate::org::lwjgl::sdl::sdl_video::SdlWindow;
use crate::org::lwjgl::system::shared_library_loader;
use ash::vk::{self, Handle};
use std::ffi::{CStr, CString, c_char, c_void};

shared_library_loader::functions! {
    "sdl", "SDL3";
    SDL_Vulkan_LoadLibrary: fn(*const c_char) -> bool;
    SDL_Vulkan_UnloadLibrary: fn();
    SDL_Vulkan_GetVkGetInstanceProcAddr: fn() -> *const c_void;
    SDL_Vulkan_GetInstanceExtensions: fn(*mut u32) -> *const *const c_char;
    SDL_Vulkan_GetPresentationSupport: fn(*mut c_void, *mut c_void, u32) -> bool;
    SDL_Vulkan_CreateSurface: fn(SdlWindow, *mut c_void, *const c_void, *mut u64) -> bool;
}

// SAFETY (all): instances, devices and windows passed in are live ones the backend made, and
// strings are null-terminated.
pub struct SDLVulkan;

#[allow(non_snake_case)]
impl SDLVulkan {
    pub const SDL_WINDOW_VULKAN: u64 = 0x1000_0000;

    pub(crate) fn SDL_Vulkan_LoadLibrary(path: Option<&str>) -> bool {
        let path = path.map(|path| CString::new(path).expect("no NUL"));
        unsafe {
            (FUNCTIONS.SDL_Vulkan_LoadLibrary)(
                path.as_ref().map_or(std::ptr::null(), |path| path.as_ptr()),
            )
        }
    }

    pub(crate) fn SDL_Vulkan_UnloadLibrary() {
        unsafe { (FUNCTIONS.SDL_Vulkan_UnloadLibrary)() }
    }

    /// The loaded loader's `vkGetInstanceProcAddr`, or `None` before it's loaded.
    pub(crate) fn SDL_Vulkan_GetVkGetInstanceProcAddr() -> Option<vk::PFN_vkGetInstanceProcAddr> {
        let function = unsafe { (FUNCTIONS.SDL_Vulkan_GetVkGetInstanceProcAddr)() };
        // SAFETY: SDL hands out the loader's vkGetInstanceProcAddr, of that signature.
        (!function.is_null()).then(|| unsafe {
            std::mem::transmute::<*const c_void, vk::PFN_vkGetInstanceProcAddr>(function)
        })
    }

    /// The instance extensions SDL needs for a window's surface, or `None` where SDL can't say.
    pub(crate) fn SDL_Vulkan_GetInstanceExtensions() -> Option<Vec<String>> {
        let mut count = 0;
        let names = unsafe { (FUNCTIONS.SDL_Vulkan_GetInstanceExtensions)(&mut count) };
        if names.is_null() {
            return None;
        }
        Some(
            (0..count as usize)
                .map(|index| {
                    // SAFETY: SDL's array holds `count` null-terminated names.
                    unsafe { CStr::from_ptr(*names.add(index)) }
                        .to_string_lossy()
                        .into_owned()
                })
                .collect(),
        )
    }

    pub(crate) fn SDL_Vulkan_GetPresentationSupport(
        instance: vk::Instance,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> bool {
        unsafe {
            (FUNCTIONS.SDL_Vulkan_GetPresentationSupport)(
                instance.as_raw() as *mut c_void,
                physical_device.as_raw() as *mut c_void,
                queue_family_index,
            )
        }
    }

    /// The window's surface, or `None` where SDL couldn't make it (`SDL_GetError` says why).
    pub(crate) fn SDL_Vulkan_CreateSurface(
        window: SdlWindow,
        instance: vk::Instance,
    ) -> Option<vk::SurfaceKHR> {
        let mut surface = 0;
        unsafe {
            (FUNCTIONS.SDL_Vulkan_CreateSurface)(
                window,
                instance.as_raw() as *mut c_void,
                std::ptr::null(),
                &mut surface,
            )
        }
        .then(|| vk::SurfaceKHR::from_raw(surface))
    }
}
