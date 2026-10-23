//! net/minecraft/client/resources/FoliageColorReloadListener.java
//!
//! What the resource reload does for the leaves' colormap: read `textures/colormap/foliage.png`,
//! then hand its pixels to `FoliageColor`. The reload's threads and profiler aren't ported; the
//! client calls these in turn.

use crate::net::minecraft::client::resources::legacy_stuff_wrapper::LegacyStuffWrapper;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use crate::net::minecraft::world::level::foliage_color::FoliageColor;

/// The leaves' colormap's loader.
pub struct FoliageColorReloadListener;

impl FoliageColorReloadListener {
    /// Panics where Java throws `IllegalStateException`, for a texture it can't read.
    pub fn prepare(manager: &dyn ResourceManager) -> Vec<i32> {
        LegacyStuffWrapper::get_pixels(
            manager,
            &Identifier::with_default_namespace("textures/colormap/foliage.png"),
        )
        .unwrap_or_else(|error| panic!("Failed to load foliage color texture: {error}"))
    }

    pub fn apply(pixels: Vec<i32>) {
        FoliageColor::init(pixels);
    }
}
