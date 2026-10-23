//! com/mojang/blaze3d/platform/NativeImage.java
//!
//! RGBA images: reading PNGs, pixels, transparency, and writing PNGs. Java keeps the pixels in
//! native memory, each an int whose bytes are red, green, blue and alpha (little-endian ABGR).
//! `read` decodes with stb_image from LWJGL's own build (`STBImage`), so the pixels are vanilla's;
//! the port copies them out of stb's buffer and frees it, where Java keeps stb's buffer. Java
//! writes PNGs with stb_image_write; the port writes the same bytes with the `png` crate, whose
//! compression differs but whose pixels are the same. Only RGBA images are made so far, and
//! fonts' (`copyFromFont`) and resizing (`resizeSubRectTo`) come with their callers.

use crate::com::mojang::blaze3d::platform::transparency::Transparency;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::png_info::PngInfo;
use crate::org::lwjgl::stb::stb_image::STBImage;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// `NativeImage.Format`: the channels a pixel has. Only their count so far.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    RGBA,
    RGB,
    LUMINANCE_ALPHA,
    LUMINANCE,
}

impl Format {
    pub fn components(self) -> i32 {
        match self {
            Format::RGBA => 4,
            Format::RGB => 3,
            Format::LUMINANCE_ALPHA => 2,
            Format::LUMINANCE => 1,
        }
    }
}

#[derive(Clone)]
pub struct NativeImage {
    width: i32,
    height: i32,
    pixels: Vec<u32>,
}

impl NativeImage {
    /// `new NativeImage(width, height, zero)`: RGBA, zeroed (Java's unzeroed memory is only ever
    /// written over before it's read). Panics where Java throws `IllegalArgumentException` for a
    /// size that isn't positive.
    pub fn new(width: i32, height: i32) -> NativeImage {
        assert!(
            width > 0 && height > 0,
            "Invalid texture size: {width}x{height}"
        );
        NativeImage {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
        }
    }

    /// `read(ByteBuffer)`: a PNG's header checked, then decoded by stb to RGBA. Java's
    /// `IOException` is the `Err`.
    pub fn read(bytes: &[u8]) -> Result<NativeImage, String> {
        PngInfo::validate_header(bytes)?;
        let Some(decoded) = STBImage::stbi_load_from_memory(bytes, Format::RGBA.components())
        else {
            return Err(format!(
                "Could not load image: {}",
                STBImage::stbi_failure_reason().unwrap_or_default()
            ));
        };
        assert!(
            decoded.width > 0 && decoded.height > 0,
            "Invalid texture size: {}x{}",
            decoded.width,
            decoded.height
        );
        let pixels = decoded
            .bytes()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| u32::from_le_bytes(*pixel))
            .collect();
        Ok(NativeImage {
            width: decoded.width,
            height: decoded.height,
            pixels,
        })
    }

    /// A new image of each pixel through `function`, as ARGB. Java throws
    /// `IllegalArgumentException` for an image that isn't RGBA, which the port's images all are.
    pub fn mapped_copy(&self, function: impl Fn(i32) -> i32) -> NativeImage {
        let pixels = self
            .pixels
            .iter()
            .map(|&pixel| ARGB::to_abgr(function(ARGB::from_abgr(pixel as i32))) as u32)
            .collect();
        NativeImage {
            width: self.width,
            height: self.height,
            pixels,
        }
    }

    fn is_outside_bounds(&self, x: i32, y: i32) -> bool {
        x < 0 || x >= self.width || y < 0 || y >= self.height
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn format(&self) -> Format {
        Format::RGBA
    }

    /// Panics where Java throws `IllegalArgumentException` for a pixel outside the image.
    pub fn set_pixel_abgr(&mut self, x: i32, y: i32, pixel: i32) {
        assert!(
            !self.is_outside_bounds(x, y),
            "({x}, {y}) outside of image bounds ({}, {})",
            self.width,
            self.height
        );
        self.pixels[(x + y * self.width) as usize] = pixel as u32;
    }

    /// Panics where Java throws `IllegalArgumentException` for a pixel outside the image.
    pub fn get_pixel_abgr(&self, x: i32, y: i32) -> i32 {
        assert!(
            !self.is_outside_bounds(x, y),
            "({x}, {y}) outside of image bounds ({}, {})",
            self.width,
            self.height
        );
        self.pixels[(x + y * self.width) as usize] as i32
    }

    /// The pixel as ARGB.
    pub fn get_pixel(&self, x: i32, y: i32) -> i32 {
        ARGB::from_abgr(self.get_pixel_abgr(x, y))
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, pixel: i32) {
        self.set_pixel_abgr(x, y, ARGB::to_abgr(pixel));
    }

    /// `getPixelsABGR()`.
    pub fn get_pixels_abgr(&self) -> &[u32] {
        &self.pixels
    }

    /// `computeTransparency(x0, y0, x1, y1)`: whether the rectangle has pixels with no alpha, and
    /// pixels with some but not all. Panics where Java throws `IllegalArgumentException` for a
    /// rectangle outside the image.
    pub fn compute_transparency_in(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> Transparency {
        assert!(
            x0 >= 0 && y0 >= 0 && x1 <= self.width && y1 <= self.height,
            "Cannot compute translucency out of bounds: [{x0}, {y0}, {x1}, {y1}] in {}x{} image",
            self.width,
            self.height
        );
        let mut has_transparent_pixel = false;
        let mut has_translucent_pixel = false;
        for y in y0..y1 {
            for x in x0..x1 {
                let alpha = ARGB::alpha(self.pixels[(x + y * self.width) as usize] as i32);
                if alpha == 0 {
                    has_transparent_pixel = true;
                } else if alpha != 255 {
                    has_translucent_pixel = true;
                }
            }
        }
        Transparency::of(has_transparent_pixel, has_translucent_pixel)
    }

    pub fn compute_transparency(&self) -> Transparency {
        self.compute_transparency_in(0, 0, self.width, self.height)
    }

    /// The pixels' bytes as they lie in memory: red, green, blue, alpha.
    pub fn get_pixel_bytes(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|pixel| pixel.to_le_bytes())
            .collect()
    }

    pub fn write_to_file(&self, path: &Path) -> std::io::Result<()> {
        let file = BufWriter::new(File::create(path)?);
        let mut encoder = png::Encoder::new(file, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(std::io::Error::other)?;
        writer
            .write_image_data(&self.get_pixel_bytes())
            .map_err(std::io::Error::other)?;
        writer.finish().map_err(std::io::Error::other)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::golden::Golden;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::tests::vanilla_resources;
    use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
    use md5::{Digest, Md5};
    use serde_json::{Value, json};

    /// An image as the goldens hold it: `[width, height, md5]` of its RGBA bytes.
    pub fn encode_image(image: &NativeImage) -> Value {
        let digest: [u8; 16] = Md5::digest(image.get_pixel_bytes()).into();
        let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        json!([image.get_width(), image.get_height(), hex])
    }

    /// Every PNG under `textures/` decodes to vanilla's pixels, through the same stb.
    #[test]
    fn reads_every_texture_like_vanilla() {
        let golden = Golden::load("com.mojang.blaze3d.platform.NativeImage");
        let manager = vanilla_resources();
        golden.check(
            "read(Ljava/io/InputStream;)Lcom/mojang/blaze3d/platform/NativeImage;",
            |id: String| {
                let resource = manager
                    .get_resource(&Identifier::parse(&id))
                    .expect("the texture is listed");
                let image = NativeImage::read(&resource.open().expect("readable"))
                    .unwrap_or_else(|error| panic!("{error}"));
                encode_image(&image)
            },
        );
        golden.finish();
    }
}
