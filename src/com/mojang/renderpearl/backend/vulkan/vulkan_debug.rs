//! com/mojang/renderpearl/backend/vulkan/VulkanDebug.java
//!
//! `VK_EXT_debug_utils`, when the game logs GPU messages (its debug verbosity, 1 by default, which
//! reports errors) or labels objects: a messenger logging what the driver reports, and names and
//! groups for a debugger. The interface's two classes are an enum. An error's log has Java's call
//! stack after it (a `StackWalker`'s five frames); here it's the message alone.

use ash::vk;
use std::cell::{Cell, RefCell};
use std::ffi::{CStr, CString, c_void};

pub enum VulkanDebug {
    Disabled,
    Enabled(Enabled),
}

/// `VulkanDebug.Enabled`.
pub struct Enabled {
    wants_debug_labels: bool,
    messenger: Cell<vk::DebugUtilsMessengerEXT>,
    severity_bitmask: vk::DebugUtilsMessageSeverityFlagsEXT,
    instance_functions: RefCell<Option<ash::ext::debug_utils::Instance>>,
    device_functions: RefCell<Option<ash::ext::debug_utils::Device>>,
}

/// `DEBUG_LEVELS`: errors, warnings, infos, then verbose messages.
const DEBUG_LEVELS: [u32; 4] = [4096, 256, 16, 1];
/// `MESSAGE_TYPE_BITMASK`: general, validation and performance messages.
const MESSAGE_TYPE_BITMASK: u32 = 7;

impl VulkanDebug {
    /// `create`: enabled, with `VK_EXT_debug_utils` among the instance's extensions, when the
    /// game wants messages or labels and the loader has the extension.
    pub fn create(
        verbosity: i32,
        wants_debug_labels: bool,
        available_extensions: &std::collections::HashSet<String>,
        enabled_extensions: &mut crate::java::util::hash_set::HashSet<String>,
    ) -> VulkanDebug {
        if (verbosity > 0 || wants_debug_labels)
            && available_extensions.contains("VK_EXT_debug_utils")
        {
            enabled_extensions.add("VK_EXT_debug_utils".to_owned());
            VulkanDebug::Enabled(Enabled::new(verbosity, wants_debug_labels))
        } else {
            VulkanDebug::Disabled
        }
    }

    /// `chainCreateInfo`: the messenger's create info, which the instance's creation reports to
    /// as well, when there are messages to report.
    pub fn messenger_create_info(&self) -> Option<vk::DebugUtilsMessengerCreateInfoEXT<'static>> {
        match self {
            VulkanDebug::Enabled(enabled) if !enabled.severity_bitmask.is_empty() => {
                Some(enabled.create_info())
            }
            _ => None,
        }
    }

    /// `setup`: the messenger, logging an error where it can't be made.
    pub fn setup(&self, entry: &ash::Entry, instance: &ash::Instance) {
        let VulkanDebug::Enabled(enabled) = self else {
            return;
        };
        let functions = ash::ext::debug_utils::Instance::new(entry, instance);
        // SAFETY: the create info's callback is a function of the right signature.
        match unsafe { functions.create_debug_utils_messenger(&enabled.create_info(), None) } {
            Ok(messenger) => enabled.messenger.set(messenger),
            Err(error) => eprintln!(
                "Error creating debug utils messenger: {}",
                crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils::result_to_string(
                    error.as_raw()
                )
            ),
        }
        *enabled.instance_functions.borrow_mut() = Some(functions);
    }

    /// The device's object naming and label functions, once there's a device.
    pub fn attach_device(&self, instance: &ash::Instance, device: &ash::Device) {
        if let VulkanDebug::Enabled(enabled) = self {
            *enabled.device_functions.borrow_mut() =
                Some(ash::ext::debug_utils::Device::new(instance, device));
        }
    }

    pub fn set_object_name(&self, object_type: vk::ObjectType, object_handle: u64, label: &str) {
        let VulkanDebug::Enabled(enabled) = self else {
            return;
        };
        if !enabled.wants_debug_labels {
            return;
        }
        if let Some(functions) = enabled.device_functions.borrow().as_ref() {
            let name = CString::new(label.replace('\0', "")).expect("no NUL");
            let mut name_info = vk::DebugUtilsObjectNameInfoEXT::default().object_name(&name);
            name_info.object_type = object_type;
            name_info.object_handle = object_handle;
            // SAFETY: the object is the device's, and the name lives for the call.
            unsafe {
                let _ = functions.set_debug_utils_object_name(&name_info);
            }
        }
    }

    pub fn begin_debug_group(&self, buffer: vk::CommandBuffer, label: &str) {
        let VulkanDebug::Enabled(enabled) = self else {
            return;
        };
        if !enabled.wants_debug_labels {
            return;
        }
        if let Some(functions) = enabled.device_functions.borrow().as_ref() {
            let name = CString::new(label.replace('\0', "")).expect("no NUL");
            let name_info = vk::DebugUtilsLabelEXT::default().label_name(&name);
            // SAFETY: the buffer is recording, and the name lives for the call.
            unsafe { functions.cmd_begin_debug_utils_label(buffer, &name_info) };
        }
    }

    pub fn end_debug_group(&self, buffer: vk::CommandBuffer) {
        let VulkanDebug::Enabled(enabled) = self else {
            return;
        };
        if !enabled.wants_debug_labels {
            return;
        }
        if let Some(functions) = enabled.device_functions.borrow().as_ref() {
            // SAFETY: the buffer is recording, inside a group.
            unsafe { functions.cmd_end_debug_utils_label(buffer) };
        }
    }

    pub fn destroy(&self) {
        if let VulkanDebug::Enabled(enabled) = self
            && let Some(functions) = enabled.instance_functions.borrow().as_ref()
            && enabled.messenger.get() != vk::DebugUtilsMessengerEXT::null()
        {
            // SAFETY: the messenger is the instance's, made in `setup`.
            unsafe { functions.destroy_debug_utils_messenger(enabled.messenger.get(), None) };
        }
    }

    pub fn enabled(&self) -> bool {
        matches!(self, VulkanDebug::Enabled(_))
    }
}

impl Enabled {
    fn new(verbosity: i32, wants_debug_labels: bool) -> Enabled {
        let mut severity_bitmask = 0;
        if verbosity > 0 {
            for level in DEBUG_LEVELS
                .iter()
                .take(verbosity.min(DEBUG_LEVELS.len() as i32) as usize)
            {
                severity_bitmask |= level;
            }
        }
        Enabled {
            wants_debug_labels,
            messenger: Cell::new(vk::DebugUtilsMessengerEXT::null()),
            severity_bitmask: vk::DebugUtilsMessageSeverityFlagsEXT::from_raw(severity_bitmask),
            instance_functions: RefCell::new(None),
            device_functions: RefCell::new(None),
        }
    }

    fn create_info(&self) -> vk::DebugUtilsMessengerCreateInfoEXT<'static> {
        vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(self.severity_bitmask)
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::from_raw(
                MESSAGE_TYPE_BITMASK,
            ))
            .pfn_user_callback(Some(on_debug_message))
    }
}

/// `onDebugMessage`: the message logged at its severity; an error makes the call that caused it
/// fail (`VK_TRUE`), as in Java.
unsafe extern "system" fn on_debug_message(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    // SAFETY: the driver hands a live callback data struct, whose message is null or a string.
    let message = unsafe {
        p_callback_data
            .as_ref()
            .filter(|data| !data.p_message.is_null())
            .map(|data| {
                CStr::from_ptr(data.p_message)
                    .to_string_lossy()
                    .into_owned()
            })
    };
    let message = message.unwrap_or_else(|| "null".to_owned());
    let severity = message_severity.as_raw();
    if severity & 16 != 0 {
        eprintln!("{message}");
    } else if severity & 256 != 0 {
        eprintln!("WARN: {message}");
    } else if severity & 4096 != 0 {
        eprintln!("ERROR: {message}");
        return vk::TRUE;
    }
    vk::FALSE
}
