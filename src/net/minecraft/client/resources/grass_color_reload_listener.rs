//! net/minecraft/client/resources/GrassColorReloadListener.java
//!
//! What the resource reload does for the grass's colormap: read `textures/colormap/grass.png`, then
//! hand its pixels to `GrassColor`. The reload's threads and profiler aren't ported; the client
//! calls these in turn.

use crate::net::minecraft::client::resources::legacy_stuff_wrapper::LegacyStuffWrapper;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use crate::net::minecraft::world::level::grass_color::GrassColor;

/// The grass's colormap's loader.
pub struct GrassColorReloadListener;

impl GrassColorReloadListener {
    /// Panics where Java throws `IllegalStateException`, for a texture it can't read.
    pub fn prepare(manager: &dyn ResourceManager) -> Vec<i32> {
        LegacyStuffWrapper::get_pixels(
            manager,
            &Identifier::with_default_namespace("textures/colormap/grass.png"),
        )
        .unwrap_or_else(|error| panic!("Failed to load grass color texture: {error}"))
    }

    pub fn apply(pixels: Vec<i32>) {
        GrassColor::init(pixels);
    }
}
