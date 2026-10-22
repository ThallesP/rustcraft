//! com/mojang/blaze3d/platform/NativeImage.java
//!
//! Only an RGBA image's pixels and writing it as a PNG so far. Java keeps the pixels in native
//! memory, each an int whose bytes are red, green, blue and alpha (little-endian ABGR), and writes
//! them with stb_image_write; the port writes the same bytes with the `png` crate, whose
//! compression differs but whose pixels are the same.

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

pub struct NativeImage {
    width: i32,
    height: i32,
    pixels: Vec<u32>,
}

impl NativeImage {
    /// `new NativeImage(width, height, useStbFree)`: RGBA, zeroed.
    pub fn new(width: i32, height: i32) -> NativeImage {
        NativeImage {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
        }
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
            (0..self.width).contains(&x) && (0..self.height).contains(&y),
            "({x}, {y}) outside of image bounds ({}x{})",
            self.width,
            self.height
        );
        self.pixels[(x + y * self.width) as usize] = pixel as u32;
    }

    pub fn get_pixel_abgr(&self, x: i32, y: i32) -> i32 {
        self.pixels[(x + y * self.width) as usize] as i32
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
