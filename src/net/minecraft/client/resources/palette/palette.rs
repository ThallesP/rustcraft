//! net/minecraft/client/resources/palette/Palette.java
//!
//! Not ported yet: `listAndLoad`, which the palettes' own reload listener calls.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
use crate::net::minecraft::server::packs::resources::resource::Resource;
use crate::net::minecraft::util::argb::ARGB;

/// A palette image's colors, ARGB, row by row.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Palette {
    colors: Vec<i32>,
}

impl Palette {
    /// `ID_CONVERTER`.
    pub fn id_converter() -> FileToIdConverter {
        FileToIdConverter::new("textures/palettes", ".png")
    }

    /// The `Err` is Java's `IOException`.
    pub fn load(resource: &Resource) -> Result<Palette, String> {
        let bytes = resource.open().map_err(|error| error.to_string())?;
        Ok(Palette::from(&NativeImage::read(&bytes)?))
    }

    /// `getPixels()`: each pixel as ARGB.
    pub fn from(image: &NativeImage) -> Palette {
        Palette {
            colors: image
                .get_pixels_abgr()
                .iter()
                .map(|&pixel| ARGB::from_abgr(pixel as i32))
                .collect(),
        }
    }

    pub fn of(colors: &[i32]) -> Palette {
        Palette {
            colors: colors.to_vec(),
        }
    }

    /// Java throws `ArrayIndexOutOfBoundsException` past the end.
    pub fn get(&self, index: usize) -> i32 {
        self.colors[index]
    }

    pub fn size(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }
}
