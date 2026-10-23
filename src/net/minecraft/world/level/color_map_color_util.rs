//! net/minecraft/world/level/ColorMapColorUtil.java

/// A colormap's colour for a temperature and downfall, Java's interface's static method.
pub fn get(temp: f64, rain: f64, pixels: &[i32], default_map_color: i32) -> i32 {
    let rain = rain * temp;
    // Java's `(int)` saturates and takes NaN to 0, like `as`.
    let x = ((1.0 - temp) * 255.0) as i32;
    let y = ((1.0 - rain) * 255.0) as i32;
    let index = y << 8 | x;
    if index >= pixels.len() as i32 {
        default_map_color
    } else {
        pixels[index as usize]
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::golden::Golden;
    use crate::net::minecraft::client::resources::dry_foliage_color_reload_listener::DryFoliageColorReloadListener;
    use crate::net::minecraft::client::resources::foliage_color_reload_listener::FoliageColorReloadListener;
    use crate::net::minecraft::client::resources::grass_color_reload_listener::GrassColorReloadListener;
    use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::tests::vanilla_resources;
    use crate::net::minecraft::world::level::dry_foliage_color::DryFoliageColor;
    use crate::net::minecraft::world::level::foliage_color::FoliageColor;
    use crate::net::minecraft::world::level::grass_color::GrassColor;

    /// The three colormaps loaded from the vanilla pack, like the resource reload.
    pub(crate) fn load_colormaps() {
        let resources = vanilla_resources();
        GrassColorReloadListener::apply(GrassColorReloadListener::prepare(&resources));
        FoliageColorReloadListener::apply(FoliageColorReloadListener::prepare(&resources));
        DryFoliageColorReloadListener::apply(DryFoliageColorReloadListener::prepare(&resources));
    }

    /// Each colormap's pixels as the reload reads them, and its colour for temperatures and
    /// downfalls all over the map and past it.
    #[test]
    fn colormaps_match_vanilla() {
        let resources = vanilla_resources();
        type Colormap = (&'static str, Vec<i32>, fn(f64, f64) -> i32);
        let maps: [Colormap; 3] = [
            (
                "net.minecraft.world.level.GrassColor",
                GrassColorReloadListener::prepare(&resources),
                GrassColor::get,
            ),
            (
                "net.minecraft.world.level.FoliageColor",
                FoliageColorReloadListener::prepare(&resources),
                FoliageColor::get,
            ),
            (
                "net.minecraft.world.level.DryFoliageColor",
                DryFoliageColorReloadListener::prepare(&resources),
                DryFoliageColor::get,
            ),
        ];
        load_colormaps();
        for (class, pixels, get) in maps {
            let golden = Golden::load(class);
            golden.field("pixels", pixels);
            golden.check("get(DD)I", get);
            golden.finish();
        }
    }
}
