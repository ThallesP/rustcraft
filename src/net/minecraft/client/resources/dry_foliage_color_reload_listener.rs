//! net/minecraft/client/resources/DryFoliageColorReloadListener.java
//!
//! What the resource reload does for the dry leaves' colormap: read
//! `textures/colormap/dry_foliage.png`, then hand its pixels to `DryFoliageColor`. The reload's
//! threads and profiler aren't ported; the client calls these in turn.

use crate::net::minecraft::client::resources::legacy_stuff_wrapper::LegacyStuffWrapper;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use crate::net::minecraft::world::level::dry_foliage_color::DryFoliageColor;

/// The dry leaves' colormap's loader.
pub struct DryFoliageColorReloadListener;

impl DryFoliageColorReloadListener {
    /// Panics where Java throws `IllegalStateException`, for a texture it can't read.
    pub fn prepare(manager: &dyn ResourceManager) -> Vec<i32> {
        LegacyStuffWrapper::get_pixels(
            manager,
            &Identifier::with_default_namespace("textures/colormap/dry_foliage.png"),
        )
        .unwrap_or_else(|error| panic!("Failed to load dry foliage color texture: {error}"))
    }

    pub fn apply(pixels: Vec<i32>) {
        DryFoliageColor::init(pixels);
    }
}
