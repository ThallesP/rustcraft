//! com/mojang/renderpearl/backend/vulkan/VulkanInstance.java
//!
//! The Vulkan 1.2 instance, with the extensions SDL needs for a window's surface, the debug
//! utils when the game wants them (`VulkanDebug`), and the portability enumeration on macOS
//! (MoltenVK). `enabledExtensions` is a Java `HashSet`, whose order the device's extension list
//! takes (`VulkanDevice`). The validation layers are only enabled where the debug options ask.

use crate::com::mojang::renderpearl::api::device::backend_creation_exception::{
    BackendCreationException, Reason,
};
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_debug::VulkanDebug;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use crate::java::util::hash_set::HashSet;
use crate::org::lwjgl::sdl::sdl_vulkan::SDLVulkan;
use crate::org::lwjgl::system::platform::Platform;
use ash::vk;
use std::ffi::{CStr, CString};

const APPLICATION_NAME: &CStr = c"Minecraft Java Edition";
/// `SharedConstants.getCurrentVersion().dataVersion().version()`: 26.3's world version.
const APPLICATION_VERSION: u32 = 5023;
const ENGINE_NAME: &CStr = c"MinecraftJE";
const ENGINE_VERSION: u32 = 0;

pub struct VulkanInstance {
    entry: ash::Entry,
    enabled_extensions: HashSet<String>,
    vk_instance: ash::Instance,
    debug: VulkanDebug,
}

impl VulkanInstance {
    pub fn new(
        entry: ash::Entry,
        debug_verbosity: i32,
        mut wants_debug_labels: bool,
        validation: bool,
    ) -> Result<VulkanInstance, BackendCreationException> {
        let app_info = vk::ApplicationInfo::default()
            .application_name(APPLICATION_NAME)
            .application_version(APPLICATION_VERSION)
            .engine_name(ENGINE_NAME)
            .engine_version(ENGINE_VERSION)
            .api_version(vk::API_VERSION_1_2);
        let validation_layers = VulkanInstance::get_supported_validation_layers(&entry)?;
        let validation_layer = c"VK_LAYER_KHRONOS_validation";
        let mut required_layers = Vec::new();
        if validation {
            if validation_layers
                .iter()
                .any(|layer| layer == "VK_LAYER_KHRONOS_validation")
            {
                required_layers.push(validation_layer.as_ptr());
                eprintln!("Enabling Vulkan validation layers");
                wants_debug_labels = true;
            } else {
                eprintln!("Vulkan validation layers requested but not found");
            }
        }

        let available_extensions = VulkanInstance::get_supported_instance_extensions(&entry)?;
        let sdl_extensions = SDLVulkan::SDL_Vulkan_GetInstanceExtensions().ok_or_else(|| {
            BackendCreationException::new(
                "Failed to find the SDL platform surface extensions".to_owned(),
                Reason::PLATFORM_ERROR,
            )
        })?;
        let mut enabled_extensions = HashSet::new();
        for extension_name in sdl_extensions {
            if extension_name != "VK_KHR_portability_enumeration" {
                enabled_extensions.add(extension_name);
            }
        }

        let debug = VulkanDebug::create(
            debug_verbosity,
            wants_debug_labels,
            &available_extensions,
            &mut enabled_extensions,
        );
        let use_portability = available_extensions.contains("VK_KHR_portability_enumeration")
            && Platform::get() == Platform::MACOSX;
        if use_portability {
            enabled_extensions.add("VK_KHR_portability_enumeration".to_owned());
        }

        let extension_names: Vec<CString> = enabled_extensions
            .iter()
            .map(|name| CString::new(name.as_str()).expect("no NUL"))
            .collect();
        let extension_pointers: Vec<_> = extension_names.iter().map(|name| name.as_ptr()).collect();
        let mut instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&required_layers)
            .enabled_extension_names(&extension_pointers);
        if use_portability {
            instance_info = instance_info.flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR);
        }
        let mut messenger_info = debug.messenger_create_info();
        if let Some(messenger_info) = &mut messenger_info {
            instance_info = instance_info.push_next(messenger_info);
        }

        // SAFETY: the create info and everything it points at live for the call.
        let vk_instance =
            unsafe { entry.create_instance(&instance_info, None) }.map_err(|error| {
                BackendCreationException::new(
                    format!(
                        "{}: Error creating instance",
                        VulkanUtils::result_to_string(error.as_raw())
                    ),
                    Reason::VULKAN_INSTANCE_CREATION_FAILED,
                )
            })?;
        debug.setup(&entry, &vk_instance);
        Ok(VulkanInstance {
            entry,
            enabled_extensions,
            vk_instance,
            debug,
        })
    }

    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn vk_instance(&self) -> &ash::Instance {
        &self.vk_instance
    }

    fn get_supported_instance_extensions(
        entry: &ash::Entry,
    ) -> Result<std::collections::HashSet<String>, BackendCreationException> {
        // SAFETY: the loader is loaded.
        let properties =
            unsafe { entry.enumerate_instance_extension_properties(None) }.map_err(|error| {
                BackendCreationException::new(
                    format!(
                        "{}: Error enumerating instance extensions",
                        VulkanUtils::result_to_string(error.as_raw())
                    ),
                    Reason::VULKAN_INSTANCE_CREATION_FAILED,
                )
            })?;
        Ok(properties
            .iter()
            .filter_map(|extension| extension.extension_name_as_c_str().ok())
            .map(|name| name.to_string_lossy().into_owned())
            .collect())
    }

    fn get_supported_validation_layers(
        entry: &ash::Entry,
    ) -> Result<Vec<String>, BackendCreationException> {
        // SAFETY: the loader is loaded.
        let properties =
            unsafe { entry.enumerate_instance_layer_properties() }.map_err(|error| {
                BackendCreationException::new(
                    format!(
                        "{}: Error enumerating validation layers",
                        VulkanUtils::result_to_string(error.as_raw())
                    ),
                    Reason::VULKAN_INSTANCE_CREATION_FAILED,
                )
            })?;
        Ok(properties
            .iter()
            .filter_map(|layer| layer.layer_name_as_c_str().ok())
            .map(|name| name.to_string_lossy().into_owned())
            .collect())
    }

    /// `close()`.
    pub fn close(&self) {
        self.debug.destroy();
        // SAFETY: everything made from the instance is gone.
        unsafe { self.vk_instance.destroy_instance(None) };
    }

    pub fn get_enabled_extensions(&self) -> &HashSet<String> {
        &self.enabled_extensions
    }

    pub fn debug(&self) -> &VulkanDebug {
        &self.debug
    }
}
