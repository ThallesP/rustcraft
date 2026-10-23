//! net/minecraft/world/level/GrassColor.java
//!
//! Java's `pixels` is a static the resource reload sets (`GrassColorReloadListener`, from
//! `textures/colormap/grass.png`); here it's a lock the loader sets, 256 by 256 zeros until then.

use std::sync::{Arc, LazyLock, RwLock};

use crate::net::minecraft::world::level::color_map_color_util;

static PIXELS: LazyLock<RwLock<Arc<[i32]>>> =
    LazyLock::new(|| RwLock::new(Arc::from(vec![0; 65536])));

/// The grass's colormap.
pub struct GrassColor;

impl GrassColor {
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
        color_map_color_util::get(temp, rain, &pixels, -65281)
    }

    pub fn get_default_color() -> i32 {
        GrassColor::get(0.5, 1.0)
    }
}
