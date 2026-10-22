//! com/mojang/renderpearl/backend/opengl/GlDebug.java
//!
//! Turns on the driver's debug messages up to the options' verbosity, which vanilla logs; the
//! port prints them to stderr. Only `enableDebugCallback`'s `KHR_debug` path is ported: the
//! `ARB_debug_output` fallback comes with a driver that needs it. Vanilla's message format and
//! its last-messages buffer come with the debug screen.

use crate::com::mojang::renderpearl::backend::opengl::gl_device::GlDevice;
use glow::HasContext;
use std::collections::HashSet;

pub struct GlDebug;

/// `DEBUG_LEVELS`: high, medium, low severity, then notifications.
const DEBUG_LEVELS: [u32; 4] = [
    glow::DEBUG_SEVERITY_HIGH,
    glow::DEBUG_SEVERITY_MEDIUM,
    glow::DEBUG_SEVERITY_LOW,
    glow::DEBUG_SEVERITY_NOTIFICATION,
];

impl GlDebug {
    /// The extension it turned on, if any.
    pub fn enable_debug_callback(
        gl: &mut glow::Context,
        capabilities: &HashSet<String>,
        verbosity: i32,
        debug_synchronous_gl_logs: bool,
    ) -> Option<&'static str> {
        if verbosity <= 0 {
            return None;
        }
        if !(capabilities.contains("GL_KHR_debug") && GlDevice::USE_GL_KHR_DEBUG) {
            return None;
        }
        // SAFETY: the device's context is current.
        unsafe {
            gl.enable(glow::DEBUG_OUTPUT);
            if debug_synchronous_gl_logs {
                gl.enable(glow::DEBUG_OUTPUT_SYNCHRONOUS);
            }
            for (index, level) in DEBUG_LEVELS.into_iter().enumerate() {
                let is_enabled = (index as i32) < verbosity;
                gl.debug_message_control(glow::DONT_CARE, glow::DONT_CARE, level, &[], is_enabled);
            }
            gl.debug_message_callback(|source, message_type, id, severity, message| {
                eprintln!(
                    "OpenGL debug message: id={id}, source={source:#x}, type={message_type:#x}, severity={severity:#x}, message='{message}'"
                );
            });
        }
        Some("GL_KHR_debug")
    }
}
