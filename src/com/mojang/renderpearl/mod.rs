//! Renderpearl, the game's renderer since 26.1: `api` is what the game calls, `frontend` turns
//! its pipelines into what a backend needs, and `backend` holds the OpenGL (and later Vulkan)
//! implementations. The game's code only sees `api`, so a backend can be swapped underneath.

pub mod api;
pub mod backend;
pub mod frontend;
pub mod util;
