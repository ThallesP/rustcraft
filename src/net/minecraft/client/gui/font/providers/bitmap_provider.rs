//! net/minecraft/client/gui/font/providers/BitmapProvider.java
//!
//! Glyphs cut from a PNG in a grid (`ascii.png`, `accented.png`, `nonlatin_european.png`): each
//! codepoint's cell, as wide as its last column with a pixel in it, scaled to the definition's
//! height, its advance that width rounded and one more. A glyph is uploaded from a GPU buffer of
//! the whole image, made when the first one is (`ImageDataHolder`). The definition is decoded by
//! hand from serde_json, the codec's way.

use std::cell::RefCell;
use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_bitmap::GlyphBitmap;
use crate::com::mojang::blaze3d::font::glyph_info::GlyphInfo;
use crate::com::mojang::blaze3d::font::glyph_provider::GlyphProvider;
use crate::com::mojang::blaze3d::font::unbaked_glyph::{Stitcher, UnbakedGlyph};
use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::net::minecraft::client::gui::font::codepoint_map::CodepointMap;
use crate::net::minecraft::client::gui::font::glyphs::baked_glyph::BakedGlyph;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

pub struct BitmapProvider {
    glyphs: CodepointMap<Rc<Glyph>>,
}

impl GlyphProvider for BitmapProvider {
    fn get_glyph(&self, codepoint: i32) -> Option<Rc<dyn UnbakedGlyph>> {
        self.glyphs
            .get(codepoint)
            .map(|glyph| Rc::clone(glyph) as Rc<dyn UnbakedGlyph>)
    }

    fn get_supported_glyphs(&self) -> Vec<i32> {
        self.glyphs.key_set()
    }
}

/// `BitmapProvider.Definition`.
#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    pub file: Identifier,
    pub height: i32,
    pub ascent: i32,
    pub codepoint_grid: Vec<Vec<i32>>,
}

impl Definition {
    pub fn decode(
        object: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Definition, String> {
        let file = object
            .get("file")
            .and_then(|file| file.as_str())
            .ok_or("No key file in MapLike")?;
        let file = Identifier::try_parse(file)
            .ok_or_else(|| format!("Not a valid resource location: {file}"))?;
        let height = match object.get("height") {
            Some(height) => int(height)?,
            None => 8,
        };
        let ascent = int(object.get("ascent").ok_or("No key ascent in MapLike")?)?;
        let chars = object
            .get("chars")
            .and_then(|chars| chars.as_array())
            .ok_or("No key chars in MapLike")?;
        let mut codepoint_grid = Vec::new();
        for line in chars {
            let line = line
                .as_str()
                .ok_or_else(|| format!("Not a string: {line}"))?;
            codepoint_grid.push(line.chars().map(|c| c as i32).collect::<Vec<i32>>());
        }
        Definition::validate_dimensions(&codepoint_grid)?;
        if ascent > height {
            return Err(format!("Ascent {ascent} higher than height {height}"));
        }
        Ok(Definition {
            file,
            height,
            ascent,
            codepoint_grid,
        })
    }

    fn validate_dimensions(grid: &[Vec<i32>]) -> Result<(), String> {
        let Some(first_line) = grid.first() else {
            return Err("Expected to find data in codepoint grid".to_owned());
        };
        let line_width = first_line.len();
        if line_width == 0 {
            return Err("Expected to find data in codepoint grid".to_owned());
        }
        for line in &grid[1..] {
            if line.len() != line_width {
                return Err(format!(
                    "Lines in codepoint grid have to be the same length (found: {} codepoints, expected: {line_width}), pad with \\u0000",
                    line.len()
                ));
            }
        }
        Ok(())
    }

    /// `load`: the grid's image (`textures/<file>`), each cell made a glyph. Java's `IOException`
    /// is the `Err`.
    pub fn load(
        &self,
        resource_manager: &dyn ResourceManager,
    ) -> Result<Rc<dyn GlyphProvider>, String> {
        let texture = self.file.with_prefix("textures/");
        let resource = resource_manager
            .get_resource(&texture)
            .ok_or_else(|| format!("{texture}"))?;
        let bytes = resource.open().map_err(|error| error.to_string())?;
        let image = NativeImage::read(&bytes)?;
        let w = image.get_width();
        let h = image.get_height();
        let glyph_width = w / self.codepoint_grid[0].len() as i32;
        let glyph_height = h / self.codepoint_grid.len() as i32;
        let pixel_scale = self.height as f32 / glyph_height as f32;
        let mut char_map = CodepointMap::new();
        let image_data_holder = Rc::new(ImageDataHolder {
            identifier: texture.clone(),
            image,
            gpu_buffer: RefCell::new(None),
        });
        for (slot_y, line) in self.codepoint_grid.iter().enumerate() {
            let slot_y = slot_y as i32;
            for (slot_x, &c) in line.iter().enumerate() {
                let slot_x = slot_x as i32;
                if c != 0 {
                    let actual_glyph_width = Definition::get_actual_glyph_width(
                        &image_data_holder.image,
                        glyph_width,
                        glyph_height,
                        slot_x,
                        slot_y,
                    );
                    let previous = char_map.put(
                        c,
                        Rc::new(Glyph {
                            scale: pixel_scale,
                            image_data: Rc::clone(&image_data_holder),
                            offset_x: slot_x * glyph_width,
                            offset_y: slot_y * glyph_height,
                            width: glyph_width,
                            height: glyph_height,
                            // (int)(0.5 + actualGlyphWidth * pixelScale) + 1: a float product,
                            // widened to a double for the sum.
                            advance: (0.5 + f64::from(actual_glyph_width as f32 * pixel_scale))
                                as i32
                                + 1,
                            ascent: self.ascent,
                        }),
                    );
                    if previous.is_some() {
                        eprintln!("Codepoint '{:x}' declared multiple times in {texture}", c);
                    }
                }
            }
        }
        Ok(Rc::new(BitmapProvider { glyphs: char_map }))
    }

    /// The width of the cell up to its last column with a pixel whose alpha isn't 0.
    fn get_actual_glyph_width(
        image: &NativeImage,
        glyph_width: i32,
        glyph_height: i32,
        x_glyph: i32,
        y_glyph: i32,
    ) -> i32 {
        let mut width = glyph_width - 1;
        while width >= 0 {
            let x_pixel = x_glyph * glyph_width + width;
            for y in 0..glyph_height {
                let y_pixel = y_glyph * glyph_height + y;
                // getLuminanceOrAlpha of an RGBA image: its alpha.
                if (image.get_pixel_abgr(x_pixel, y_pixel) as u32 >> 24) != 0 {
                    return width + 1;
                }
            }
            width -= 1;
        }
        width + 1
    }
}

/// `Codec.INT`: a number's `intValue()`.
fn int(value: &serde_json::Value) -> Result<i32, String> {
    match value {
        serde_json::Value::Number(number) => Ok(match number.as_i64() {
            Some(int) => int as i32,
            None => number.as_f64().unwrap_or(0.0) as i32,
        }),
        other => Err(format!("Not a number: {other}")),
    }
}

/// `BitmapProvider.ImageDataHolder`: the image, and the GPU buffer of its pixels.
struct ImageDataHolder {
    identifier: Identifier,
    image: NativeImage,
    gpu_buffer: RefCell<Option<Rc<dyn GpuBuffer>>>,
}

impl ImageDataHolder {
    fn gpu_data(&self, device: &dyn GpuDevice) -> Rc<dyn GpuBuffer> {
        let mut gpu_buffer = self.gpu_buffer.borrow_mut();
        let buffer = gpu_buffer.get_or_insert_with(|| {
            device.create_buffer_with_data(
                Some(&format!("{} staging buffer", self.identifier)),
                gpu_buffer::USAGE_COPY_SRC
                    | gpu_buffer::USAGE_HINT_CLIENT_STORAGE
                    | gpu_buffer::USAGE_MAP_WRITE,
                &self.image.get_pixel_bytes(),
            )
        });
        Rc::clone(buffer)
    }
}

/// `BitmapProvider.Glyph`.
struct Glyph {
    scale: f32,
    image_data: Rc<ImageDataHolder>,
    offset_x: i32,
    offset_y: i32,
    width: i32,
    height: i32,
    advance: i32,
    ascent: i32,
}

impl UnbakedGlyph for Glyph {
    fn info(&self) -> GlyphInfo {
        GlyphInfo::simple(self.advance as f32)
    }

    fn bake(&self, stitcher: &mut dyn Stitcher) -> Rc<BakedGlyph> {
        stitcher.stitch(self.info(), self)
    }
}

impl GlyphBitmap for Glyph {
    fn get_oversample(&self) -> f32 {
        1.0 / self.scale
    }

    fn get_pixel_width(&self) -> i32 {
        self.width
    }

    fn get_pixel_height(&self) -> i32 {
        self.height
    }

    fn get_bearing_top(&self) -> f32 {
        self.ascent as f32
    }

    fn upload(&self, device: &dyn GpuDevice, x: i32, y: i32, texture: &dyn GpuTexture) {
        let buffer = self.image_data.gpu_data(device);
        device.create_command_encoder().copy_buffer_to_texture(
            &gpu_buffer::slice(&buffer),
            self.offset_x,
            self.offset_y,
            self.image_data.image.get_width(),
            self.image_data.image.get_height(),
            texture,
            x,
            y,
            self.width,
            self.height,
            0,
            0,
        );
    }

    /// The port reads every image as RGBA, like Java's `NativeImage.read`.
    fn is_colored(&self) -> bool {
        true
    }
}
