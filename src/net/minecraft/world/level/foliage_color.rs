//! net/minecraft/world/level/FoliageColor.java
//!
//! Java's `pixels` is a static the resource reload sets (`FoliageColorReloadListener`, from
//! `textures/colormap/foliage.png`); here it's a lock the loader sets, 256 by 256 zeros until then.

use std::sync::{Arc, LazyLock, RwLock};

use crate::net::minecraft::world::level::color_map_color_util;

static PIXELS: LazyLock<RwLock<Arc<[i32]>>> =
    LazyLock::new(|| RwLock::new(Arc::from(vec![0; 65536])));

/// The leaves' colormap.
pub struct FoliageColor;

impl FoliageColor {
    pub const FOLIAGE_EVERGREEN: i32 = -10380959;
    pub const FOLIAGE_BIRCH: i32 = -8345771;
    pub const FOLIAGE_DEFAULT: i32 = -12012264;
    pub const FOLIAGE_MANGROVE: i32 = -7158200;

    pub fn init(pixels: Vec<i32>) {
        *PIXELS
            .write()
            .expect("no thread panics holding the colormap") = Arc::from(pixels);
    }

    pub fn get(temp: f64, rain: f64) -> i32 {
        let pixels = Arc::clone(
            &PIXELS
                .read()
                .expect("no thread panics holding the colormap"),
        );
        color_map_color_util::get(temp, rain, &pixels, FoliageColor::FOLIAGE_DEFAULT)
    }
}
