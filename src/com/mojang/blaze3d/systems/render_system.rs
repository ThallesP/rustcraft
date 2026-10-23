//! com/mojang/blaze3d/systems/RenderSystem.java
//!
//! `initBackendSystem` (SDL's hints and app metadata, then its video subsystem), the shared index
//! buffers (`AutoStorageIndexBuffer`), and the state a frame's passes bind by default: the
//! projection, the fog, the global settings and the lights (`bindDefaultUniforms`), and the model
//! view stack. Java's statics are the render thread's here (`thread_local!`), which is the only
//! thread that touches them. `SdlDebug.init()` (SDL's log callback) and the timer it returns come
//! with the window loop. The device (`DEVICE`) and the shared index buffers
//! (`getSequentialBuffer`) are passed around until the game's renderer holds them.

use crate::com::mojang::blaze3d::buffers::std140_size_calculator::Std140SizeCalculator;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::net::minecraft::util::mth::Mth;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::matrix4f_stack::Matrix4fStack;
use crate::org::lwjgl::sdl::sdl_error::SDLError;
use crate::org::lwjgl::sdl::sdl_hints::SDLHints;
use crate::org::lwjgl::sdl::sdl_init::SDLInit;
use std::cell::RefCell;
use std::rc::Rc;

/// The uniforms `bindDefaultUniforms` binds, and the model view stack.
struct State {
    shader_fog: Option<GpuBufferSlice>,
    shader_light_directions: Option<GpuBufferSlice>,
    projection_matrix_buffer: Option<GpuBufferSlice>,
    global_settings_uniform: Option<Rc<dyn GpuBuffer>>,
    model_view_stack: Matrix4fStack,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        shader_fog: None,
        shader_light_directions: None,
        projection_matrix_buffer: None,
        global_settings_uniform: None,
        model_view_stack: Matrix4fStack::new(16),
    });
}

pub struct RenderSystem;

impl RenderSystem {
    pub const PROJECTION_MATRIX_UBO_SIZE: i32 = Std140SizeCalculator::new().put_mat4f().get();

    /// The shared index buffer that draws `primitive_topology` from sequential vertices: quads
    /// and lines as two triangles each, the rest a vertex each.
    pub fn sequential_buffer(primitive_topology: PrimitiveTopology) -> AutoStorageIndexBuffer {
        match primitive_topology {
            PrimitiveTopology::QUADS => {
                AutoStorageIndexBuffer::new(4, 6, |i| vec![i, i + 1, i + 2, i + 2, i + 3, i])
            }
            PrimitiveTopology::LINES => {
                AutoStorageIndexBuffer::new(4, 6, |i| vec![i, i + 1, i + 2, i + 3, i + 2, i + 1])
            }
            _ => AutoStorageIndexBuffer::new(1, 1, |i| vec![i]),
        }
    }

    pub fn set_shader_fog(fog: GpuBufferSlice) {
        STATE.with_borrow_mut(|state| state.shader_fog = Some(fog));
    }

    pub fn get_shader_fog() -> Option<GpuBufferSlice> {
        STATE.with_borrow(|state| state.shader_fog.clone())
    }

    pub fn set_shader_lights(buffer: GpuBufferSlice) {
        STATE.with_borrow_mut(|state| state.shader_light_directions = Some(buffer));
    }

    /// `setProjectionMatrix`: the projection's buffer (its type only matters to the GUI's
    /// sorting, which isn't ported).
    pub fn set_projection_matrix(projection_matrix_buffer: GpuBufferSlice) {
        STATE.with_borrow_mut(|state| {
            state.projection_matrix_buffer = Some(projection_matrix_buffer);
        });
    }

    pub fn set_global_settings_uniform(buffer: Rc<dyn GpuBuffer>) {
        STATE.with_borrow_mut(|state| state.global_settings_uniform = Some(buffer));
    }

    /// `getModelViewStack()`, lent to `f`.
    pub fn with_model_view_stack<R>(f: impl FnOnce(&mut Matrix4fStack) -> R) -> R {
        STATE.with_borrow_mut(|state| f(&mut state.model_view_stack))
    }

    pub fn get_model_view_matrix_copy() -> Matrix4f {
        STATE.with_borrow(|state| Matrix4f::from_matrix4f(&state.model_view_stack))
    }

    pub fn bind_default_uniforms(render_pass: &mut dyn RenderPass) {
        let (projection_matrix, fog, global_uniform, shader_lights) = STATE.with_borrow(|state| {
            (
                state.projection_matrix_buffer.clone(),
                state.shader_fog.clone(),
                state.global_settings_uniform.clone(),
                state.shader_light_directions.clone(),
            )
        });
        if let Some(projection_matrix) = projection_matrix {
            render_pass.set_uniform_slice("Projection", &projection_matrix);
        }

        if let Some(fog) = fog {
            render_pass.set_uniform_slice("Fog", &fog);
        }

        if let Some(global_uniform) = global_uniform {
            render_pass.set_uniform_buffer("Globals", &global_uniform);
        }

        if let Some(shader_lights) = shader_lights {
            render_pass.set_uniform_slice("Lighting", &shader_lights);
        }
    }

    /// Panics where Java throws `IllegalStateException`.
    pub fn init_backend_system() {
        let version = include_str!("../../../../../MINECRAFT_VERSION").trim();
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.name", "Minecraft");
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.version", version);
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.identifier", "com.mojang.minecraft");
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.creator", "Mojang Studios");
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.copyright", "Copyright Mojang AB.");
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.url", "https://www.minecraft.net");
        SDLInit::SDL_SetAppMetadataProperty("SDL.app.metadata.type", "game");
        SDLHints::SDL_SetHint("SDL_NO_SIGNAL_HANDLERS", "1");
        SDLHints::SDL_SetHint("SDL_VIDEO_MINIMIZE_ON_FOCUS_LOSS", "0");
        SDLHints::SDL_SetHint("SDL_QUIT_ON_LAST_WINDOW_CLOSE", "0");
        SDLHints::SDL_SetHint("SDL_MOUSE_FOCUS_CLICKTHROUGH", "1");
        SDLHints::SDL_SetHint("SDL_ENABLE_SCREEN_KEYBOARD", "0");
        SDLHints::SDL_SetHint("SDL_IME_IMPLEMENTED_UI", "composition");
        assert!(
            SDLInit::SDL_Init(SDLInit::SDL_INIT_VIDEO),
            "Unable to initialize SDL: {}",
            SDLError::SDL_GetError().unwrap_or_default()
        );
    }
}

/// `RenderSystem.AutoStorageIndexBuffer`: indices for sequential vertices, regenerated at twice
/// the count asked for when it runs out. Its generator gives a primitive's indices from its
/// first vertex, where Java's hands them to a consumer.
pub struct AutoStorageIndexBuffer {
    vertex_stride: i32,
    index_stride: i32,
    generator: fn(i32) -> Vec<i32>,
    buffer: Option<Rc<dyn GpuBuffer>>,
    index_type: IndexType,
    index_count: i32,
    max_requested_index_count: i32,
}

impl AutoStorageIndexBuffer {
    fn new(vertex_stride: i32, index_stride: i32, generator: fn(i32) -> Vec<i32>) -> Self {
        AutoStorageIndexBuffer {
            vertex_stride,
            index_stride,
            generator,
            buffer: None,
            index_type: IndexType::SHORT,
            index_count: 0,
            max_requested_index_count: 0,
        }
    }

    pub fn has_storage(&self, index_count: i32) -> bool {
        index_count <= self.index_count
    }

    pub fn request_index_count(&mut self, index_count: i32) {
        self.max_requested_index_count = self.max_requested_index_count.max(index_count);
    }

    pub fn resize_to_requested_index_count(&mut self, device: &dyn GpuDevice) {
        self.ensure_storage(device, self.max_requested_index_count);
    }

    pub fn get_buffer(&mut self, device: &dyn GpuDevice, index_count: i32) -> Rc<dyn GpuBuffer> {
        self.request_index_count(index_count);
        self.ensure_storage(device, index_count);
        self.buffer.clone().expect("a buffer")
    }

    fn ensure_storage(&mut self, device: &dyn GpuDevice, index_count: i32) {
        if self.has_storage(index_count) {
            return;
        }
        let index_count = Mth::round_toward_i32(index_count * 2, self.index_stride);
        let primitive_count = index_count / self.index_stride;
        let vertex_count = primitive_count * self.vertex_stride;
        let index_type = IndexType::least(vertex_count);
        let buffer_size = Mth::round_toward_i32(index_count * index_type.bytes(), 4);
        let mut data = Vec::with_capacity(buffer_size as usize);
        self.index_type = index_type;
        let mut ii = 0;
        while ii < index_count {
            for value in (self.generator)(ii * self.vertex_stride / self.index_stride) {
                match index_type {
                    IndexType::SHORT => data.extend_from_slice(&(value as i16).to_le_bytes()),
                    IndexType::INT => data.extend_from_slice(&value.to_le_bytes()),
                }
            }
            ii += self.index_stride;
        }
        self.buffer = Some(device.create_buffer_with_data(
            Some("Auto Storage index buffer"),
            gpu_buffer::USAGE_INDEX,
            &data,
        ));
        self.index_count = index_count;
    }

    pub fn index_type(&self) -> IndexType {
        self.index_type
    }
}
