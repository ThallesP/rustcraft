//! net/minecraft/client/renderer/texture/SpriteContents.java
//!
//! A sprite's images: its first frame's size, the whole image (every frame of an animation) and
//! its mip levels, and how it animates. Java's `originalImage` is `byMipLevel[0]`, the same
//! object, so here it's the first level. The additional metadata sections an atlas asks for
//! (Java's `MetadataSectionType.WithValue`s) are `AdditionalMetadata`'s kinds, so far the GUI
//! atlas's `gui` section. The animation's GPU half (`AnimationState`'s frame textures and draws) is
//! `TextureAtlas`'s, which owns the device's objects; this holds what it draws from.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::blaze3d::platform::transparency::Transparency;
use crate::net::minecraft::client::renderer::texture::mipmap_generator::MipmapGenerator;
use crate::net::minecraft::client::renderer::texture::mipmap_strategy::MipmapStrategy;
use crate::net::minecraft::client::renderer::texture::stitcher::Entry;
use crate::net::minecraft::client::resources::metadata::animation::animation_metadata_section::AnimationMetadataSection;
use crate::net::minecraft::client::resources::metadata::animation::frame_size::FrameSize;
use crate::net::minecraft::client::resources::metadata::gui::gui_metadata_section::GuiMetadataSection;
use crate::net::minecraft::client::resources::metadata::texture::texture_metadata_section::TextureMetadataSection;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;

#[derive(Clone)]
pub struct SpriteContents {
    name: Identifier,
    width: i32,
    height: i32,
    by_mip_level: Vec<NativeImage>,
    animated_texture: Option<AnimatedTexture>,
    mipmap_strategy: MipmapStrategy,
    alpha_cutoff_bias: f32,
    transparency: Transparency,
    additional_metadata: Vec<AdditionalMetadata>,
}

/// An additional metadata section's type, which an atlas asks its sprites for
/// (`MetadataSectionType<?>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdditionalMetadataType {
    Gui,
}

/// An additional metadata section's value (`MetadataSectionType.WithValue<?>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdditionalMetadata {
    Gui(GuiMetadataSection),
}

/// `SpriteContents.FrameInfo`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameInfo {
    pub index: i32,
    pub time: i32,
}

/// `SpriteContents.AnimatedTexture`: the frames in order, the frames used once each in the order
/// they're first used, and how many frames a row of the image holds.
#[derive(Clone, Debug)]
pub struct AnimatedTexture {
    pub frames: Vec<FrameInfo>,
    pub unique_frames: Vec<i32>,
    pub frame_row_size: i32,
    pub interpolate_frames: bool,
}

impl AnimatedTexture {
    fn new(
        frames: Vec<FrameInfo>,
        frame_row_size: i32,
        interpolate_frames: bool,
    ) -> AnimatedTexture {
        let mut unique_frames = Vec::new();
        for frame in &frames {
            if !unique_frames.contains(&frame.index) {
                unique_frames.push(frame.index);
            }
        }
        AnimatedTexture {
            frames,
            unique_frames,
            frame_row_size,
            interpolate_frames,
        }
    }

    pub fn get_frame_x(&self, index: i32) -> i32 {
        index % self.frame_row_size
    }

    pub fn get_frame_y(&self, index: i32) -> i32 {
        index / self.frame_row_size
    }
}

impl SpriteContents {
    pub const UBO_SIZE: i32 =
        crate::com::mojang::blaze3d::buffers::std140_size_calculator::Std140SizeCalculator::new()
            .put_mat4f()
            .put_mat4f()
            .put_float()
            .put_float()
            .put_int()
            .get();

    pub fn new(
        name: Identifier,
        frame_size: FrameSize,
        image: NativeImage,
        animation_info: Option<AnimationMetadataSection>,
        additional_metadata: Vec<AdditionalMetadata>,
        texture_info: Option<TextureMetadataSection>,
    ) -> SpriteContents {
        let animated_texture = animation_info.and_then(|animation| {
            SpriteContents::create_animated_texture(
                &name,
                frame_size,
                image.get_width(),
                image.get_height(),
                &animation,
            )
        });
        let transparency = image.compute_transparency();
        SpriteContents {
            name,
            width: frame_size.width,
            height: frame_size.height,
            by_mip_level: vec![image],
            animated_texture,
            mipmap_strategy: texture_info.map_or(MipmapStrategy::AUTO, |info| info.mipmap_strategy),
            alpha_cutoff_bias: texture_info.map_or(0.0, |info| info.alpha_cutoff_bias),
            transparency,
            additional_metadata,
        }
    }

    /// Java wraps a failure in a crash report, which is the panic.
    pub fn increase_mip_level(&mut self, mip_level: i32) {
        let current = std::mem::take(&mut self.by_mip_level);
        self.by_mip_level = MipmapGenerator::generate_mip_levels(
            &self.name,
            current,
            mip_level,
            self.mipmap_strategy,
            self.alpha_cutoff_bias,
            self.transparency,
        );
    }

    fn get_frame_count(&self) -> usize {
        self.animated_texture
            .as_ref()
            .map_or(1, |animated| animated.frames.len())
    }

    pub fn is_animated(&self) -> bool {
        self.get_frame_count() > 1
    }

    pub fn transparency(&self) -> Transparency {
        self.transparency
    }

    /// Java logs each invalid frame (a time that isn't positive, an index past the image) and
    /// drops it, and logs the frames no entry uses.
    fn create_animated_texture(
        name: &Identifier,
        frame_size: FrameSize,
        full_width: i32,
        full_height: i32,
        metadata: &AnimationMetadataSection,
    ) -> Option<AnimatedTexture> {
        let frame_row_size = full_width / frame_size.width;
        let frame_column_size = full_height / frame_size.height;
        let total_frame_count = frame_row_size * frame_column_size;
        let default_frame_time = metadata.default_frame_time;
        let frames: Vec<FrameInfo> = match &metadata.frames {
            None => (0..total_frame_count)
                .map(|i| FrameInfo {
                    index: i,
                    time: default_frame_time,
                })
                .collect(),
            Some(metadata_frames) => metadata_frames
                .iter()
                .map(|frame| FrameInfo {
                    index: frame.index,
                    time: frame.time_or(default_frame_time),
                })
                .filter(|frame| {
                    let valid =
                        frame.time > 0 && frame.index >= 0 && frame.index < total_frame_count;
                    if !valid {
                        eprintln!("Invalid frame on sprite {name}: {frame:?}");
                    }
                    valid
                })
                .collect(),
        };
        if frames.len() <= 1 {
            None
        } else {
            Some(AnimatedTexture::new(
                frames,
                frame_row_size,
                metadata.interpolated_frames,
            ))
        }
    }

    /// `getAdditionalMetadata(GuiMetadataSection.TYPE)`.
    pub fn get_gui_metadata(&self) -> Option<GuiMetadataSection> {
        // The GUI's section is the only kind so far, so the first is it.
        self.additional_metadata
            .iter()
            .map(|AdditionalMetadata::Gui(section)| *section)
            .next()
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }

    /// `getUniqueFrames()`: `[1]` for a still sprite, as Java's `IntList.of(1)`.
    pub fn get_unique_frames(&self) -> Vec<i32> {
        self.animated_texture
            .as_ref()
            .map_or_else(|| vec![1], |animated| animated.unique_frames.clone())
    }

    pub fn animated_texture(&self) -> Option<&AnimatedTexture> {
        self.animated_texture.as_ref()
    }

    /// The image at each mip level (the first is Java's `originalImage`).
    pub fn by_mip_level(&self) -> &[NativeImage] {
        &self.by_mip_level
    }

    pub fn original_image(&self) -> &NativeImage {
        &self.by_mip_level[0]
    }

    pub fn is_transparent(&self, frame: i32, x: i32, y: i32) -> bool {
        let mut actual_x = x;
        let mut actual_y = y;
        if let Some(animated) = &self.animated_texture {
            actual_x += animated.get_frame_x(frame) * self.width;
            actual_y += animated.get_frame_y(frame) * self.height;
        }
        ARGB::alpha(self.original_image().get_pixel(actual_x, actual_y)) == 0
    }

    pub fn compute_transparency(&self, u0: f32, v0: f32, u1: f32, v1: f32) -> Transparency {
        if self.transparency.is_opaque() {
            return self.transparency;
        }
        if u0 == 0.0 && v0 == 0.0 && u1 == 1.0 && v1 == 1.0 {
            return self.transparency;
        }
        let x0 = Mth::floor_f32(u0 * self.width as f32);
        let y0 = Mth::floor_f32(v0 * self.height as f32);
        let x1 = Mth::ceil_f32(u1 * self.width as f32);
        let y1 = Mth::ceil_f32(v1 * self.height as f32);
        let Some(animated) = &self.animated_texture else {
            return self
                .original_image()
                .compute_transparency_in(x0, y0, x1, y1);
        };
        let mut transparency = Transparency::NONE;
        for &frame in &animated.unique_frames {
            let frame_x = animated.get_frame_x(frame) * self.width;
            let frame_y = animated.get_frame_y(frame) * self.height;
            transparency = transparency.or(self.original_image().compute_transparency_in(
                frame_x + x0,
                frame_y + y0,
                frame_x + x1,
                frame_y + y1,
            ));
        }
        transparency
    }
}

impl Entry for SpriteContents {
    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn name(&self) -> &Identifier {
        &self.name
    }
}
