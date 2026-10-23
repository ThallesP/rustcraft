//! net/minecraft/client/resources/LegacyStuffWrapper.java

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

/// How the colormaps' pixels are read.
pub struct LegacyStuffWrapper;

impl LegacyStuffWrapper {
    /// The image's pixels as ARGB, row by row. An error where Java throws `IOException`: a
    /// texture that's missing or isn't an image.
    pub fn get_pixels(
        resource_manager: &dyn ResourceManager,
        location: &Identifier,
    ) -> Result<Vec<i32>, String> {
        let resource = resource_manager
            .get_resource(location)
            .ok_or_else(|| location.to_string())?;
        let bytes = resource.open().map_err(|error| error.to_string())?;
        let image = NativeImage::read(&bytes)?;
        Ok(image.make_pixel_array())
    }
}
