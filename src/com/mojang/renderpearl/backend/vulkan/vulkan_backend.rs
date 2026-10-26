//! com/mojang/renderpearl/backend/vulkan/VulkanBackend.java
//!
//! Vulkan: the loader LWJGL loads (`libvulkan.so.1` on Linux, the MoltenVK in LWJGL's natives
//! jar on macOS, `vulkan-1.dll` on Windows), loaded through SDL, whose `vkGetInstanceProcAddr`
//! `ash` calls everything through (Java checks LWJGL's and SDL's are the same function; the port
//! has only SDL's). The device is the first one with everything the game requires, or a discrete
//! one after it, with the optional feature sets it has (`VulkanFeatureSets`), and VMA over it.
//! `checkBackendAvailable`, which only the game's default backend choice asks, isn't ported: the
//! port picks its backend by its option.

use crate::com::mojang::renderpearl::api::device::backend_creation_exception::{
    BackendCreationException, Reason,
};
use crate::com::mojang::renderpearl::api::device::gpu_backend::GpuBackend;
use crate::com::mojang::renderpearl::api::device::gpu_debug_options::GpuDebugOptions;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::backend::vulkan::checkpoints::checkpoint_extension::CheckpointExtension;
use crate::com::mojang::renderpearl::backend::vulkan::checkpoints::noop_checkpoint_extension::NoopCheckpointExtension;
use crate::com::mojang::renderpearl::backend::vulkan::init::feature_set::FeatureSet;
use crate::com::mojang::renderpearl::backend::vulkan::init::vulkan_p_next_struct::PNextArena;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_device::VulkanDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_feature_sets::VulkanFeatureSets;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_instance::VulkanInstance;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_physical_device::VulkanPhysicalDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::{DeviceUUID, VulkanUtils};
use crate::org::lwjgl::sdl::sdl_error::SDLError;
use crate::org::lwjgl::sdl::sdl_video::{SDLVideo, SdlWindow};
use crate::org::lwjgl::sdl::sdl_vulkan::SDLVulkan;
use crate::org::lwjgl::system::platform::Platform;
use crate::org::lwjgl::system::shared_library_loader;
use crate::org::lwjgl::util::vma::vma::{
    VULKAN_FUNCTION_NAMES, Vma, VmaAllocator, VmaAllocatorCreateInfo, VmaVulkanFunctions,
};
use ash::vk;
use std::ffi::{CString, c_void};

#[derive(Default)]
pub struct VulkanBackend {
    library_loaded: bool,
    library_load_failure: Option<BackendCreationException>,
    entry: Option<ash::Entry>,
}

impl VulkanBackend {
    pub fn new() -> VulkanBackend {
        VulkanBackend::default()
    }

    /// The loader `VK.create()` loads: its path, or `None` where it can't be found.
    fn vulkan_loader_path() -> Option<String> {
        match Platform::get() {
            Platform::LINUX | Platform::FREEBSD => Some("libvulkan.so.1".to_owned()),
            Platform::WINDOWS => Some("vulkan-1.dll".to_owned()),
            Platform::MACOSX => Some(
                shared_library_loader::extract("vulkan", "MoltenVK")
                    .map(|path| path.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| "libvulkan.1.dylib".to_owned()),
            ),
        }
    }

    /// `NativeLibrariesBootstrap.isVulkanLoaderAvailable()`: whether LWJGL could load the loader.
    fn is_vulkan_loader_available() -> bool {
        VulkanBackend::vulkan_loader_path().is_some_and(|path| {
            // SAFETY: the loader has no initialisers that need anything of the loading thread.
            unsafe { libloading::Library::new(path) }.is_ok()
        })
    }

    fn find_physical_device(
        instance: &VulkanInstance,
        required_feature_sets: &[&FeatureSet],
        required_if_extensions_available: &[&FeatureSet],
    ) -> Result<VulkanPhysicalDevice, BackendCreationException> {
        let vk_instance = instance.vk_instance();
        // SAFETY: the instance is live.
        let physical_devices =
            unsafe { vk_instance.enumerate_physical_devices() }.map_err(|error| {
                BackendCreationException::new(
                    format!(
                        "{}: Failed to get physical devices",
                        VulkanUtils::result_to_string(error.as_raw())
                    ),
                    Reason::VULKAN_NO_DEVICE,
                )
            })?;
        if physical_devices.is_empty() {
            return Err(BackendCreationException::new(
                "No Vulkan capable devices".to_owned(),
                Reason::VULKAN_NO_DEVICE,
            ));
        }

        let mut device_failure_reason = None;
        let mut selected_device: Option<vk::PhysicalDevice> = None;
        for current_device in physical_devices {
            if current_device == vk::PhysicalDevice::null() {
                continue;
            }
            match VulkanBackend::check_device_suitability(
                vk_instance,
                current_device,
                required_feature_sets,
                required_if_extensions_available,
            )? {
                Some(failure_reason) => {
                    device_failure_reason.get_or_insert(failure_reason);
                }
                None => match selected_device {
                    None => selected_device = Some(current_device),
                    Some(selected) => {
                        if VulkanBackend::is_device_discrete(vk_instance, current_device)
                            && !VulkanBackend::is_device_discrete(vk_instance, selected)
                        {
                            eprintln!(
                                "Preferring discrete GPU: {}",
                                VulkanBackend::get_device_name(vk_instance, current_device)
                            );
                            selected_device = Some(current_device);
                            break;
                        }
                    }
                },
            }
        }

        match selected_device {
            Some(selected_device) => VulkanPhysicalDevice::new(vk_instance, selected_device),
            None => Err(device_failure_reason.unwrap_or_else(|| {
                BackendCreationException::new(
                    "No Vulkan capable devices".to_owned(),
                    Reason::VULKAN_NO_DEVICE,
                )
            })),
        }
    }

    fn device_meets_feature_query_requirements(
        instance: &ash::Instance,
        vk_physical_device: vk::PhysicalDevice,
    ) -> bool {
        // SAFETY: the device is the instance's.
        let properties = unsafe { instance.get_physical_device_properties(vk_physical_device) };
        properties.api_version >= vk::API_VERSION_1_1
    }

    fn is_device_discrete(
        instance: &ash::Instance,
        vk_physical_device: vk::PhysicalDevice,
    ) -> bool {
        // SAFETY: the device is the instance's.
        let properties = unsafe { instance.get_physical_device_properties(vk_physical_device) };
        properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
    }

    fn get_device_name(instance: &ash::Instance, vk_physical_device: vk::PhysicalDevice) -> String {
        // SAFETY: the device is the instance's.
        let properties = unsafe { instance.get_physical_device_properties(vk_physical_device) };
        properties.device_name_as_c_str().map_or_else(
            |_| String::new(),
            |name| name.to_string_lossy().into_owned(),
        )
    }

    /// Why the device won't do, or `None` when it will.
    fn check_device_suitability(
        instance: &ash::Instance,
        vk_physical_device: vk::PhysicalDevice,
        required_feature_sets: &[&FeatureSet],
        required_if_extensions_available: &[&FeatureSet],
    ) -> Result<Option<BackendCreationException>, BackendCreationException> {
        let physical_device = VulkanPhysicalDevice::new(instance, vk_physical_device)?;
        let device_name = physical_device.device_name();
        if !VulkanBackend::device_meets_feature_query_requirements(instance, vk_physical_device) {
            eprintln!(
                "Device [{device_name}] does not support Vulkan 1.1, skipping further capability checks"
            );
            return Ok(Some(BackendCreationException::with_missing_capabilities(
                "Device missing capabilities".to_owned(),
                Reason::VULKAN_DEVICE_VERSION_TOO_LOW,
                vec!["VULKAN_CORE_1_1".to_owned()],
            )));
        }

        let device_uuid = DeviceUUID {
            driver_id: physical_device
                .vk_physical_device_driver_properties()
                .driver_id
                .as_raw(),
            vendor_id: physical_device.vk_physical_device_properties().vendor_id as i32,
            device_id: physical_device.vk_physical_device_properties().device_id as i32,
        };
        if VulkanUtils::is_known_problematic(device_uuid) {
            eprintln!("Device [{device_name}] is known to be problematic, skipping");
            return Ok(Some(BackendCreationException::with_missing_capabilities(
                "Device known problematic".to_owned(),
                Reason::VULKAN_KNOWN_PROBLEMATIC,
                Vec::new(),
            )));
        }

        let mut device_required_feature_sets: Vec<&FeatureSet> = required_feature_sets.to_vec();
        let device_extensions = VulkanUtils::enumerate_extensions(instance, vk_physical_device)?;
        for feature_set in required_if_extensions_available {
            if feature_set
                .extensions
                .iter()
                .all(|extension| device_extensions.contains(*extension))
            {
                eprintln!(
                    "Device [{device_name}] supports all extensions from FeatureSet [{}], making required",
                    feature_set.name
                );
                device_required_feature_sets.push(feature_set);
            }
        }

        let mut missing_capabilities = Vec::new();
        let mut most_prominent_reason = None;
        for feature_set in &device_required_feature_sets {
            let missing_features = feature_set.unsupported_features(instance, vk_physical_device);
            if !missing_features.is_empty() {
                eprintln!(
                    "Device [{device_name}] does not support required features from FeatureSet [{}], missing: {:?}",
                    feature_set.name,
                    missing_features
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                );
                most_prominent_reason = Some(Reason::VULKAN_MISSING_FEATURE);
                for missing_feature in missing_features {
                    missing_capabilities.push(missing_feature.name.to_owned());
                }
            }
        }

        for feature_set in &device_required_feature_sets {
            let missing_extensions = feature_set.unsupported_extensions(&device_extensions);
            if !missing_extensions.is_empty() {
                eprintln!(
                    "Device [{device_name}] does not support required extensions from FeatureSet [{}], missing: {missing_extensions:?}",
                    feature_set.name
                );
                most_prominent_reason = Some(Reason::VULKAN_MISSING_EXTENSION);
                missing_capabilities
                    .extend(missing_extensions.iter().map(|name| (*name).to_owned()));
            }
        }

        if physical_device.graphics_queue_family_and_index().is_none() {
            eprintln!("Device [{device_name}] does not have a graphics queue");
            most_prominent_reason = Some(Reason::VULKAN_NO_GRAPHICS_QUEUE);
            missing_capabilities.push("COMBINED_GRAPHICS_COMPUTE_PRESENT_QUEUE".to_owned());
        }

        if physical_device.vk_physical_device_properties().api_version < vk::API_VERSION_1_2 {
            eprintln!("Device [{device_name}] does not support Vulkan 1.2");
            most_prominent_reason = Some(Reason::VULKAN_DEVICE_VERSION_TOO_LOW);
            missing_capabilities.push("VULKAN_CORE_1_2".to_owned());
        }

        Ok(most_prominent_reason.map(|reason| {
            BackendCreationException::with_missing_capabilities(
                "Device missing capabilities".to_owned(),
                reason,
                missing_capabilities,
            )
        }))
    }

    /// `createDevice(FeatureSet, VulkanPhysicalDevice)`: the device with the set's extensions
    /// and features, and each family's queues.
    fn create_vk_device(
        instance: &ash::Instance,
        feature_set: &FeatureSet,
        physical_device: &VulkanPhysicalDevice,
    ) -> Result<ash::Device, BackendCreationException> {
        let mut arena = PNextArena::default();
        let device_features = arena.features2();
        for required_device_feature in &feature_set.features {
            // SAFETY: the chain lives in the arena.
            unsafe { required_device_feature.set(device_features, true, &mut arena) };
        }

        let priorities: Vec<Vec<f32>> = physical_device
            .queue_family_create_info_map()
            .iter()
            .map(|(_, count)| vec![0.0; *count as usize])
            .collect();
        let queue_creation_info: Vec<vk::DeviceQueueCreateInfo<'_>> = physical_device
            .queue_family_create_info_map()
            .iter()
            .zip(&priorities)
            .map(|((family, _), priorities)| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*family as u32)
                    .queue_priorities(priorities)
            })
            .collect();
        let extension_names: Vec<CString> = feature_set
            .extensions
            .iter()
            .map(|name| CString::new(*name).expect("no NUL"))
            .collect();
        let extension_pointers: Vec<_> = extension_names.iter().map(|name| name.as_ptr()).collect();
        // SAFETY: the head is a VkPhysicalDeviceFeatures2, whose chain and features live in the
        // arena for the call.
        let features2 = unsafe { &*device_features.cast::<vk::PhysicalDeviceFeatures2<'_>>() };
        let mut device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_creation_info)
            .enabled_extension_names(&extension_pointers)
            .enabled_features(&features2.features);
        device_create_info.p_next = features2.p_next.cast_const();
        // SAFETY: as above.
        unsafe {
            instance.create_device(
                physical_device.vk_physical_device(),
                &device_create_info,
                None,
            )
        }
        .map_err(|error| {
            BackendCreationException::new(
                format!(
                    "{}: Failed to create device",
                    VulkanUtils::result_to_string(error.as_raw())
                ),
                Reason::VULKAN_NO_DEVICE,
            )
        })
    }

    /// `createVma`: VMA over the device, with the functions LWJGL's `VmaVulkanFunctions.set`
    /// hands it, the instance's from `vkGetInstanceProcAddr` and the device's from
    /// `vkGetDeviceProcAddr`.
    fn create_vma(
        entry: &ash::Entry,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
    ) -> Result<VmaAllocator, BackendCreationException> {
        let get_instance_proc_addr = entry.static_fn().get_instance_proc_addr;
        let get_device_proc_addr = instance.fp_v1_0().get_device_proc_addr;
        let mut functions = VmaVulkanFunctions {
            functions: [std::ptr::null(); 28],
        };
        for (slot, name) in functions.functions.iter_mut().zip(VULKAN_FUNCTION_NAMES) {
            let name = CString::new(name).expect("no NUL");
            // SAFETY: the instance and device are live, and the names null-terminated.
            let function = unsafe {
                match name.to_bytes() {
                    b"vkGetInstanceProcAddr" => Some(std::mem::transmute::<
                        vk::PFN_vkGetInstanceProcAddr,
                        unsafe extern "system" fn(),
                    >(get_instance_proc_addr)),
                    b"vkGetDeviceProcAddr" => Some(std::mem::transmute::<
                        vk::PFN_vkGetDeviceProcAddr,
                        unsafe extern "system" fn(),
                    >(get_device_proc_addr)),
                    b"vkGetPhysicalDeviceProperties"
                    | b"vkGetPhysicalDeviceMemoryProperties"
                    | b"vkGetPhysicalDeviceMemoryProperties2"
                    | b"vkGetPhysicalDeviceProperties2" => {
                        get_instance_proc_addr(instance.handle(), name.as_ptr())
                    }
                    _ => get_device_proc_addr(device.handle(), name.as_ptr()),
                }
            };
            *slot = function.map_or(std::ptr::null(), |function| function as *const c_void);
        }
        let create_info = VmaAllocatorCreateInfo {
            flags: 0,
            physical_device,
            device: device.handle(),
            preferred_large_heap_block_size: 0,
            p_allocation_callbacks: std::ptr::null(),
            p_device_memory_callbacks: std::ptr::null(),
            p_heap_size_limit: std::ptr::null(),
            p_vulkan_functions: &functions,
            instance: instance.handle(),
            vulkan_api_version: vk::API_VERSION_1_2,
            p_type_external_memory_handle_types: std::ptr::null(),
        };
        let (result, allocator) = Vma::vmaCreateAllocator(&create_info);
        VulkanUtils::throw_if_failure(result, "Failed to create VMA allocator", Reason::OTHER)?;
        Ok(allocator)
    }
}

impl GpuBackend for VulkanBackend {
    fn get_name(&self) -> &'static str {
        "Vulkan"
    }

    fn load_library(&mut self) -> Result<(), BackendCreationException> {
        if self.library_loaded {
            return Ok(());
        }
        if let Some(failure) = &self.library_load_failure {
            return Err(failure.clone());
        }
        if !VulkanBackend::is_vulkan_loader_available() {
            let failure = BackendCreationException::new(
                "Vulkan loader library is missing".to_owned(),
                Reason::VULKAN_LOADER_MISSING,
            );
            self.library_load_failure = Some(failure.clone());
            return Err(failure);
        }
        let loaded =
            SDLVulkan::SDL_Vulkan_LoadLibrary(VulkanBackend::vulkan_loader_path().as_deref());
        let get_instance_proc_addr = SDLVulkan::SDL_Vulkan_GetVkGetInstanceProcAddr();
        let Some(get_instance_proc_addr) = get_instance_proc_addr.filter(|_| loaded) else {
            let failure = BackendCreationException::new(
                format!(
                    "Vulkan is not supported: {}",
                    SDLError::SDL_GetError().unwrap_or_else(|| "<no error>".to_owned())
                ),
                Reason::PLATFORM_ERROR,
            );
            self.library_load_failure = Some(failure.clone());
            return Err(failure);
        };
        // SAFETY: SDL's vkGetInstanceProcAddr is the loader's.
        self.entry = Some(unsafe {
            ash::Entry::from_static_fn(ash::StaticFn {
                get_instance_proc_addr,
            })
        });
        self.library_loaded = true;
        Ok(())
    }

    fn unload_library(&mut self) {
        if self.library_loaded {
            self.entry = None;
            SDLVulkan::SDL_Vulkan_UnloadLibrary();
            self.library_loaded = false;
        }
    }

    /// `createWindow`: a window SDL can make a Vulkan surface for.
    fn create_window(&self, title: Option<&str>, width: i32, height: i32, flags: u64) -> SdlWindow {
        SDLVideo::SDL_CreateWindow(title, width, height, SDLVulkan::SDL_WINDOW_VULKAN | flags)
    }

    fn create_device(
        &self,
        debug_options: &GpuDebugOptions,
    ) -> Result<Box<dyn GpuDevice>, BackendCreationException> {
        if !VulkanBackend::is_vulkan_loader_available() {
            return Err(BackendCreationException::new(
                "Vulkan loader library is missing".to_owned(),
                Reason::VULKAN_LOADER_MISSING,
            ));
        }
        let entry = self.entry.clone().expect("the library is loaded");
        let required_feature_sets = VulkanFeatureSets::required_feature_sets();
        let required_if_extensions_available_feature_sets =
            VulkanFeatureSets::required_if_extensions_available_feature_sets();
        let optional_feature_sets = VulkanFeatureSets::optional_feature_sets();

        let renderdoc_attached =
            std::env::var("ENABLE_VULKAN_RENDERDOC_CAPTURE").is_ok_and(|value| value == "1");
        let instance = VulkanInstance::new(
            entry.clone(),
            debug_options.log_level,
            debug_options.use_labels || renderdoc_attached,
            debug_options.use_validation_layers,
        )?;
        let result = (|| {
            let physical_device = VulkanBackend::find_physical_device(
                &instance,
                &required_feature_sets,
                &required_if_extensions_available_feature_sets,
            )?;
            let vk_instance = instance.vk_instance();
            let vk_physical_device = physical_device.vk_physical_device();
            let device_extensions =
                VulkanUtils::enumerate_extensions(vk_instance, vk_physical_device)?;
            let mut enabled_feature_sets: Vec<&FeatureSet> = required_feature_sets.clone();
            for feature_set in &required_if_extensions_available_feature_sets {
                if feature_set.is_supported(vk_instance, vk_physical_device, &device_extensions) {
                    enabled_feature_sets.push(feature_set);
                    eprintln!(
                        "Enabling required for device FeatureSet [{}]",
                        feature_set.name
                    );
                }
            }
            for feature_set in &optional_feature_sets {
                if feature_set.is_supported(vk_instance, vk_physical_device, &device_extensions) {
                    if feature_set.check_condition(vk_instance, vk_physical_device)? {
                        enabled_feature_sets.push(feature_set);
                        eprintln!("Enabling optional FeatureSet [{}]", feature_set.name);
                    } else {
                        eprintln!(
                            "Optional FeatureSet [{}] supported but device condition failed and will not be enabled",
                            feature_set.name
                        );
                    }
                } else {
                    eprintln!("Optional FeatureSet [{}] not supported", feature_set.name);
                }
            }

            let checkpoint_extension: Box<dyn CheckpointExtension> =
                Box::new(NoopCheckpointExtension::INSTANCE);
            let enabled_features = FeatureSet::composite_of("Enabled", &enabled_feature_sets);
            let device =
                VulkanBackend::create_vk_device(vk_instance, &enabled_features, &physical_device)?;
            match VulkanBackend::create_vma(&entry, vk_instance, vk_physical_device, &device) {
                Ok(vma) => Ok((
                    physical_device,
                    enabled_features,
                    device,
                    vma,
                    checkpoint_extension,
                )),
                Err(error) => {
                    // SAFETY: nothing was made from the device.
                    unsafe { device.destroy_device(None) };
                    Err(error)
                }
            }
        })();
        match result {
            Ok((physical_device, enabled_features, device, vma, checkpoint_extension)) => {
                Ok(Box::new(VulkanDevice::new(
                    instance,
                    physical_device,
                    enabled_features,
                    device,
                    vma,
                    checkpoint_extension,
                )))
            }
            Err(error) => {
                instance.close();
                Err(error)
            }
        }
    }
}
