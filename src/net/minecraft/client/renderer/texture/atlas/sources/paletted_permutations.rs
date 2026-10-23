//! net/minecraft/client/renderer/texture/atlas/sources/PalettedPermutations.java
//!
//! `MAP_CODEC` is decoded by hand in `SpriteSourceList`. Java keeps the permutations' suppliers in
//! a `HashMap`, whose order only decides the order the sprites load in, which the stitcher sorts
//! away; the port keeps the definition's order. Java's suppliers look the palettes up in the
//! resource manager `run` was given when they're first asked for; here `run` looks them up and
//! the suppliers read them when they're first asked for, which finds the same resources.

use std::sync::{Arc, Mutex};

use crate::net::minecraft::client::renderer::texture::atlas::sources::lazy_loaded_image::LazyLoadedImage;
use crate::net::minecraft::client::renderer::texture::atlas::sprite_source::{
    Output, texture_id_converter,
};
use crate::net::minecraft::client::renderer::texture::sprite_contents::SpriteContents;
use crate::net::minecraft::client::resources::metadata::animation::frame_size::FrameSize;
use crate::net::minecraft::client::resources::palette::palette::Palette;
use crate::net::minecraft::client::resources::palette::palette_mapping::PaletteMapping;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource::Resource;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

pub const DEFAULT_SEPARATOR: &str = "_";

/// Each texture recolored with each palette: its colors in the key palette mapped to the
/// permutation's, named `<texture><separator><permutation>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PalettedPermutations {
    pub textures: Vec<Identifier>,
    pub palette_key: Identifier,
    pub permutations: Vec<(String, Identifier)>,
    pub separator: String,
}

/// `PalettedPermutations.PalettedSpriteSupplier`.
pub struct PalettedSpriteSupplier {
    base_image: Arc<LazyLoadedImage>,
    palette: Arc<MemoizedMapping>,
    permutation_location: Identifier,
}

/// A palette mapping made the first time it's asked for (`Suppliers.memoize`). Java's key palette
/// is memoized too, and read once per permutation's first ask here, which reads the same image.
pub struct MemoizedMapping {
    palette_key: (Identifier, Option<Resource>),
    palette: (Identifier, Option<Resource>),
    mapping: Mutex<Option<Result<PaletteMapping, String>>>,
}

impl PalettedPermutations {
    pub fn run(&self, resource_manager: &dyn ResourceManager, output: &mut Output) {
        let find = |palette_id: &Identifier| {
            let resource =
                resource_manager.get_resource(&Palette::id_converter().id_to_file(palette_id));
            (palette_id.clone(), resource)
        };
        let palettes: Vec<(String, Arc<MemoizedMapping>)> = self
            .permutations
            .iter()
            .map(|(suffix, palette)| {
                (
                    suffix.clone(),
                    Arc::new(MemoizedMapping {
                        palette_key: find(&self.palette_key),
                        palette: find(palette),
                        mapping: Mutex::new(None),
                    }),
                )
            })
            .collect();

        for texture_location in &self.textures {
            let texture_id = texture_id_converter().id_to_file(texture_location);
            let Some(resource) = resource_manager.get_resource(&texture_id) else {
                eprintln!("Unable to find texture {texture_id}");
                continue;
            };
            let base_image = Arc::new(LazyLoadedImage::new(
                texture_id,
                resource,
                palettes.len() as i32,
            ));
            for (suffix, palette) in &palettes {
                let permutation_location =
                    texture_location.with_suffix(&format!("{}{suffix}", self.separator));
                output.add_loader(
                    permutation_location.clone(),
                    PalettedSpriteSupplier {
                        base_image: base_image.clone(),
                        palette: palette.clone(),
                        permutation_location,
                    },
                );
            }
        }
    }

    // Java logs and throws `IllegalArgumentException`, which the `Err` is.
    fn load_palette_entry_from_image(
        palette: &(Identifier, Option<Resource>),
    ) -> Result<Palette, String> {
        let (palette_id, resource) = palette;
        let Some(resource) = resource else {
            eprintln!("Failed to load palette image {palette_id}");
            return Err("IllegalArgumentException".to_owned());
        };
        Palette::load(resource).map_err(|error| {
            eprintln!("Couldn't load texture {palette_id}: {error}");
            "IllegalArgumentException".to_owned()
        })
    }
}

impl MemoizedMapping {
    fn get(&self) -> Result<PaletteMapping, String> {
        let mut mapping = self.mapping.lock().expect("not poisoned");
        mapping
            .get_or_insert_with(|| {
                let key = PalettedPermutations::load_palette_entry_from_image(&self.palette_key)?;
                let palette = PalettedPermutations::load_palette_entry_from_image(&self.palette)?;
                PaletteMapping::create(&key, &palette)
            })
            .clone()
    }
}

impl PalettedSpriteSupplier {
    /// The recolored sprite, or `None` where Java logs that it couldn't make it. Releases the
    /// base image either way.
    pub fn get(&self) -> Option<SpriteContents> {
        let result = self.palette.get().and_then(|palette| {
            self.base_image
                .with_image(|image| image.mapped_copy(|color| palette.apply(color)))
        });
        self.base_image.release();
        match result {
            Ok(image) => {
                let frame_size = FrameSize {
                    width: image.get_width(),
                    height: image.get_height(),
                };
                Some(SpriteContents::new(
                    self.permutation_location.clone(),
                    frame_size,
                    image,
                    None,
                    Vec::new(),
                    None,
                ))
            }
            Err(error) => {
                eprintln!(
                    "unable to apply palette to {}: {error}",
                    self.permutation_location
                );
                None
            }
        }
    }

    pub fn discard(&self) {
        self.base_image.release();
    }
}
