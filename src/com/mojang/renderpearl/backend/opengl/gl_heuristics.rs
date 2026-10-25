//! com/mojang/renderpearl/backend/opengl/GlHeuristics.java
//!
//! `Util.getPlatform()` and `Util.isAarch64()` (GL on DirectX 12 for Windows on ARM) are what the
//! port is built for; `Util.isAppleSiliconMac` reads the renderer, like Java's.

use crate::com::mojang::renderpearl::api::device::device_features::DeviceFeatures;
use crate::com::mojang::renderpearl::api::device::device_info::DeviceInfo;
use crate::com::mojang::renderpearl::api::device::device_limits::DeviceLimits;
use crate::com::mojang::renderpearl::api::device::device_type::DeviceType;
use crate::com::mojang::renderpearl::api::device::hints_and_workarounds::HintsAndWorkarounds;
use crate::net::minecraft::util::util::Util;
use glow::HasContext;

const DEVICE_NAMES_THAT_IMPLY_CPU: [&str; 2] = ["mesa offscreen", "llvmpipe"];
const DEVICE_NAMES_THAT_IMPLY_VIRTUAL: [&str; 1] = ["virtgl"];

/// Guesses about the driver from its renderer and vendor strings.
#[derive(Clone, Copy, Debug)]
pub struct GlHeuristics {
    is_gl_on_dx12: bool,
    is_amd: bool,
    is_nvidia: bool,
    could_be_intel_gen7: bool,
}

impl GlHeuristics {
    pub fn new(device_name: &str, vendor: &str) -> GlHeuristics {
        GlHeuristics {
            is_gl_on_dx12: GlHeuristics::is_gl_on_dx12_name(device_name),
            is_amd: device_name.contains("AMD"),
            is_nvidia: device_name.to_lowercase().contains("nvidia"),
            could_be_intel_gen7: GlHeuristics::could_be_intel_gen7_name(
                &device_name.to_lowercase(),
                &vendor.to_lowercase(),
            ),
        }
    }

    pub fn is_gl_on_dx12(&self) -> bool {
        self.is_gl_on_dx12
    }

    pub fn is_amd(&self) -> bool {
        self.is_amd
    }

    pub fn is_nvidia(&self) -> bool {
        self.is_nvidia
    }

    pub fn could_be_intel_gen7(&self) -> bool {
        self.could_be_intel_gen7
    }

    fn is_gl_on_dx12_name(device_name: &str) -> bool {
        let is_windows_arm64 = cfg!(all(windows, target_arch = "aarch64"));
        is_windows_arm64 || device_name.starts_with("D3D12")
    }

    /// The largest square RGBA8 texture the driver takes, probed with the proxy target from 32768
    /// (or the reported maximum, if larger) down to 1024.
    fn get_max_supported_texture_size(gl: &glow::Context) -> i32 {
        // SAFETY: the device's context is current.
        unsafe {
            let max_reported = gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE);
            let mut tex_size = max_reported.max(32768);
            while tex_size >= 1024 {
                gl.tex_image_2d(
                    glow::PROXY_TEXTURE_2D,
                    0,
                    glow::RGBA as i32,
                    tex_size,
                    tex_size,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(None),
                );
                if gl.get_tex_level_parameter_i32(glow::PROXY_TEXTURE_2D, 0, glow::TEXTURE_WIDTH)
                    != 0
                {
                    return tex_size;
                }
                tex_size >>= 1;
            }
            max_reported.max(1024)
        }
    }

    pub fn create_device_info(
        &self,
        gl: &glow::Context,
        has_clip_control: bool,
        max_supported_anisotropy: i32,
        enabled_extensions: &[String],
    ) -> DeviceInfo {
        // SAFETY: the device's context is current.
        let (renderer, vendor, version, uniform_offset_alignment, max_draw_buffers) = unsafe {
            (
                gl.get_parameter_string(glow::RENDERER),
                gl.get_parameter_string(glow::VENDOR),
                gl.get_parameter_string(glow::VERSION),
                gl.get_parameter_i32(glow::UNIFORM_BUFFER_OFFSET_ALIGNMENT),
                gl.get_parameter_i32(glow::MAX_DRAW_BUFFERS),
            )
        };
        let renderer_lower_case = renderer.to_lowercase();
        let vendor_lower_case = vendor.to_lowercase();
        let enabled = |extension: &str| {
            enabled_extensions
                .iter()
                .any(|enabled| enabled == extension)
        };
        let draw_indirect_count = if enabled("GL_ARB_multi_draw_indirect") {
            i32::MAX
        } else if enabled("GL_ARB_draw_indirect") {
            1
        } else {
            0
        };
        DeviceInfo {
            name: renderer.clone(),
            vendor_name: vendor,
            driver_info: version,
            is_z_zero_to_one: has_clip_control,
            backend_name: "OpenGL".to_owned(),
            timestamp_period: 1.0,
            limits: DeviceLimits {
                max_anisotropy: max_supported_anisotropy,
                min_uniform_offset_alignment: uniform_offset_alignment,
                max_texture_size: GlHeuristics::get_max_supported_texture_size(gl),
                max_memory_allocation_size: i64::MAX,
                max_multi_draw_direct_interleaved_draw_count: 0,
                max_color_attachments: max_draw_buffers,
                max_draw_indirect_draw_count: draw_indirect_count,
            },
            features: DeviceFeatures {
                wireframe_fill_mode: true,
                shader_draw_parameters: enabled("GL_ARB_shader_draw_parameters"),
                multi_draw_direct_interleaved: false,
                multi_draw_direct_separate: true,
                multi_draw_indirect: enabled("GL_ARB_multi_draw_indirect"),
                draw_indirect: enabled("GL_ARB_draw_indirect"),
                non_zero_first_instance: enabled("GL_ARB_base_instance"),
                persistent_mapping: enabled("GL_ARB_buffer_storage"),
            },
            underlying_extensions: enabled_extensions.to_vec(),
            hints_and_workarounds: HintsAndWorkarounds {
                write_to_buffer_is_slow: self.is_gl_on_dx12,
                anisotropy_has_known_issues: self.is_amd,
                is_explicit_depth_required: Util::is_apple_silicon_mac(&renderer),
                multi_draw_indirect_has_known_issues: vendor_lower_case.contains("intel")
                    && !renderer_lower_case.contains("arc"),
            },
            device_type: GlHeuristics::guess_device_type(&renderer_lower_case, &vendor_lower_case),
        }
    }

    fn could_be_intel_gen7_name(renderer: &str, vendor: &str) -> bool {
        if !vendor.contains("intel") {
            false
        } else if renderer.contains("2500") || renderer.contains("4000") {
            true
        } else {
            renderer.contains("hd graphics (byt)") || renderer.ends_with("hd graphics")
        }
    }

    fn guess_device_type(renderer: &str, vendor: &str) -> DeviceType {
        if vendor.contains("intel") {
            return if renderer.contains("arc") {
                DeviceType::DISCRETE
            } else {
                DeviceType::INTEGRATED
            };
        }
        if DEVICE_NAMES_THAT_IMPLY_CPU
            .iter()
            .any(|name| renderer.contains(name))
        {
            return DeviceType::CPU;
        }
        if DEVICE_NAMES_THAT_IMPLY_VIRTUAL
            .iter()
            .any(|name| renderer.contains(name))
        {
            return DeviceType::VIRTUAL;
        }
        DeviceType::OTHER
    }
}
