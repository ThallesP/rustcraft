//! net/minecraft/client/renderer/texture/TextureAtlas.java
//!
//! The atlas texture and its sprites: `upload` makes a texture of the stitched size with a view
//! per mip level, then draws every sprite into each level with `pipeline/animate_sprite_blit`, a
//! quad over the sprite's padded place sampling its image with clamped edges, so the padding
//! repeats the sprite's border. A still sprite is drawn from a scratch texture of its mip levels;
//! an animated one keeps a texture per frame and draws its current frame (blending into the next
//! one's with `pipeline/animate_sprite_interpolate`), now and whenever its frame changes
//! (`cycleAnimationFrames`).
//!
//! Java's `AbstractTexture` (the texture, its view and sampler) and `SpriteContents.AnimationState`
//! are this struct's fields: the atlas owns every device object. `RenderSystem.bindDefaultUniforms`
//! binds nothing the animation pipelines read, so the passes leave it out until `RenderSystem`
//! holds the uniforms. Java draws its sprites in its `HashMap`'s order, into places that don't
//! overlap, so the order doesn't show; here it's the stitcher's.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::{self, GpuTexture};
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::texture::missing_texture_atlas_sprite::MissingTextureAtlasSprite;
use crate::net::minecraft::client::renderer::texture::sprite_contents::{
    AnimatedTexture, SpriteContents,
};
use crate::net::minecraft::client::renderer::texture::sprite_loader::Preparations;
use crate::net::minecraft::client::renderer::texture::texture_atlas_sprite::TextureAtlasSprite;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::mth::Mth;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

/// The two pipelines the atlas draws with, compiled (`RenderSystem.getCompiledPipeline`).
pub struct AnimationPipelines {
    pub blit: Rc<dyn CompiledRenderPipeline>,
    pub interpolate: Rc<dyn CompiledRenderPipeline>,
}

pub struct TextureAtlas {
    location: Identifier,
    max_supported_texture_size: i32,
    sprites: Vec<Arc<TextureAtlasSprite>>,
    animated_textures_states: Vec<AnimationState>,
    texture: Option<Rc<dyn GpuTexture>>,
    texture_view: Option<GpuTextureView>,
    sampler: Option<Rc<dyn GpuSampler>>,
    width: i32,
    height: i32,
    max_mip_level: i32,
    mip_level_count: i32,
    mip_views: Vec<GpuTextureView>,
    sprite_ubos: Option<Rc<dyn GpuBuffer>>,
}

/// `SpriteContents.AnimationState`: where an animated sprite is in its frames, its frames'
/// textures, and its blocks for each mip level.
struct AnimationState {
    sprite: usize,
    frame: usize,
    sub_frame: i32,
    animation_info: AnimatedTexture,
    frame_textures_by_index: BTreeMap<i32, GpuTextureView>,
    sprite_ubos_by_mip: Vec<GpuBufferSlice>,
    is_dirty: bool,
}

impl AnimationState {
    fn tick(&mut self) {
        self.sub_frame += 1;
        self.is_dirty = false;
        let current_frame = self.animation_info.frames[self.frame];
        if self.sub_frame >= current_frame.time {
            let old_frame = current_frame.index;
            self.frame = (self.frame + 1) % self.animation_info.frames.len();
            self.sub_frame = 0;
            let new_frame = self.animation_info.frames[self.frame].index;
            if old_frame != new_frame {
                self.is_dirty = true;
            }
        }
    }

    fn needs_to_draw(&self) -> bool {
        self.animation_info.interpolate_frames || self.is_dirty
    }

    fn draw_to_atlas(
        &self,
        render_pass: &mut dyn RenderPass,
        ubo: &GpuBufferSlice,
        pipelines: &AnimationPipelines,
        sampler: &Rc<dyn GpuSampler>,
    ) {
        let frames = &self.animation_info.frames;
        let old_frame = frames[self.frame].index;
        let frame_progress = self.sub_frame as f32 / frames[self.frame].time as f32;
        let frame_progress_as_int = (frame_progress * 1000.0) as i32;
        if self.animation_info.interpolate_frames {
            let new_frame = frames[(self.frame + 1) % frames.len()].index;
            render_pass.set_pipeline(&pipelines.interpolate);
            render_pass.set_uniform_sampler(
                "CurrentSprite",
                self.frame_textures_by_index.get(&old_frame),
                Some(sampler),
            );
            render_pass.set_uniform_sampler(
                "NextSprite",
                self.frame_textures_by_index.get(&new_frame),
                Some(sampler),
            );
        } else if self.is_dirty {
            render_pass.set_pipeline(&pipelines.blit);
            render_pass.set_uniform_sampler(
                "Sprite",
                self.frame_textures_by_index.get(&old_frame),
                Some(sampler),
            );
        }
        render_pass.set_uniform_slice("SpriteAnimationInfo", ubo);
        render_pass.draw(6, 1, frame_progress_as_int << 3, 0);
    }
}

impl TextureAtlas {
    pub const LOCATION_BLOCKS: &str = "textures/atlas/blocks.png";
    pub const LOCATION_ITEMS: &str = "textures/atlas/items.png";

    pub fn new(device: &dyn GpuDevice, location: Identifier) -> TextureAtlas {
        TextureAtlas {
            location,
            max_supported_texture_size: device
                .get_device_info()
                .limits
                .max_texture_size_for_format(GpuFormat::RGBA8_UNORM),
            sprites: Vec::new(),
            animated_textures_states: Vec::new(),
            texture: None,
            texture_view: None,
            sampler: None,
            width: 0,
            height: 0,
            max_mip_level: 0,
            mip_level_count: 0,
            mip_views: Vec::new(),
            sprite_ubos: None,
        }
    }

    fn create_texture(
        &mut self,
        device: &dyn GpuDevice,
        new_width: i32,
        new_height: i32,
        new_mip_level: i32,
    ) {
        eprintln!(
            "Created: {new_width}x{new_height}x{new_mip_level} {}-atlas",
            self.location
        );
        let texture = device.create_texture(
            Some(&self.location.to_string()),
            gpu_texture::USAGE_COPY_DST
                | gpu_texture::USAGE_COPY_SRC
                | gpu_texture::USAGE_TEXTURE_BINDING
                | gpu_texture::USAGE_RENDER_ATTACHMENT,
            GpuFormat::RGBA8_UNORM,
            new_width,
            new_height,
            1,
            new_mip_level + 1,
        );
        self.texture_view = Some(device.create_texture_view(&texture));
        self.width = new_width;
        self.height = new_height;
        self.max_mip_level = new_mip_level;
        self.mip_level_count = new_mip_level + 1;
        self.mip_views = (0..=new_mip_level)
            .map(|level| device.create_texture_view_mips(&texture, level, 1))
            .collect();
        self.texture = Some(texture);
    }

    /// Panics where Java throws `IllegalStateException` for an atlas without the missing sprite.
    pub fn upload(
        &mut self,
        device: &dyn GpuDevice,
        sampler_cache: &SamplerCache,
        pipelines: &AnimationPipelines,
        preparations: Preparations,
    ) {
        self.create_texture(
            device,
            preparations.width,
            preparations.height,
            preparations.mip_level,
        );
        self.clear_texture_data();
        self.sampler = Some(sampler_cache.get_clamp_to_edge(FilterMode::NEAREST).clone());
        assert!(
            preparations.missing().is_some(),
            "Atlas '{}' ({} sprites) has no missing texture sprite",
            self.location,
            preparations.regions.len()
        );
        self.sprites = preparations.regions;

        let sprite_ubo_size = Mth::round_toward_i32(
            SpriteContents::UBO_SIZE,
            device.get_device_info().limits.min_uniform_offset_alignment,
        );
        let ubo_block_size = sprite_ubo_size * self.mip_level_count;
        let animated: Vec<usize> = (0..self.sprites.len())
            .filter(|&i| self.sprites[i].is_animated())
            .collect();
        if !animated.is_empty() {
            let mut sprite_ubo_buffer = vec![0; animated.len() * ubo_block_size as usize];
            for (animation_index, &sprite) in animated.iter().enumerate() {
                self.sprites[sprite].upload_sprite_ubo(
                    &mut sprite_ubo_buffer,
                    animation_index as i32 * ubo_block_size,
                    self.max_mip_level,
                    self.width,
                    self.height,
                    sprite_ubo_size,
                );
            }
            let sprite_ubos = device.create_buffer_with_data(
                Some(&format!("{} sprite UBOs", self.location)),
                gpu_buffer::USAGE_UNIFORM,
                &sprite_ubo_buffer,
            );
            for (animation_index, &sprite) in animated.iter().enumerate() {
                let slice = gpu_buffer::slice_range(
                    &sprite_ubos,
                    i64::from(animation_index as i32 * ubo_block_size),
                    i64::from(ubo_block_size),
                );
                let state = self.create_animation_state(device, sprite, &slice, sprite_ubo_size);
                self.animated_textures_states.push(state);
            }
            self.sprite_ubos = Some(sprite_ubos);
        }

        self.upload_initial_contents(device, sampler_cache, pipelines);
    }

    /// `SpriteContents.createAnimationState` (its `AnimatedTexture`'s): a texture of each unique
    /// frame with every mip level, copied out of the sprite's levels.
    fn create_animation_state(
        &self,
        device: &dyn GpuDevice,
        sprite: usize,
        ubo_slice: &GpuBufferSlice,
        sprite_ubo_size: i32,
    ) -> AnimationState {
        let contents = self.sprites[sprite].contents();
        let animated = contents
            .animated_texture()
            .expect("an animated sprite")
            .clone();
        let by_mip_level = contents.by_mip_level();
        let encoder = device.create_command_encoder();
        let staging_buffers: Vec<GpuBufferSlice> = by_mip_level
            .iter()
            .map(|image| {
                gpu_buffer::slice(&device.create_buffer_with_data(
                    Some("Sprite staging"),
                    gpu_buffer::USAGE_COPY_SRC,
                    &image.get_pixel_bytes(),
                ))
            })
            .collect();
        let mut frame_textures_by_index = BTreeMap::new();
        for &frame in &animated.unique_frames {
            let texture = device.create_texture(
                Some(&format!("{} animation frame {frame}", contents.name())),
                gpu_texture::USAGE_COPY_DST | gpu_texture::USAGE_TEXTURE_BINDING,
                GpuFormat::RGBA8_UNORM,
                contents.width(),
                contents.height(),
                1,
                by_mip_level.len() as i32,
            );
            let offset_x = animated.get_frame_x(frame) * contents.width();
            let offset_y = animated.get_frame_y(frame) * contents.height();
            for (level, image) in by_mip_level.iter().enumerate() {
                let level = level as i32;
                encoder.copy_buffer_to_texture(
                    &staging_buffers[level as usize],
                    offset_x >> level,
                    offset_y >> level,
                    image.get_width(),
                    image.get_height(),
                    &*texture,
                    0,
                    0,
                    contents.width() >> level,
                    contents.height() >> level,
                    level,
                    0,
                );
            }
            frame_textures_by_index.insert(frame, device.create_texture_view(&texture));
        }
        let sprite_ubos_by_mip = (0..by_mip_level.len() as i64)
            .map(|level| {
                ubo_slice.slice(
                    level * i64::from(sprite_ubo_size),
                    i64::from(sprite_ubo_size),
                )
            })
            .collect();
        AnimationState {
            sprite,
            frame: 0,
            sub_frame: 0,
            animation_info: animated,
            frame_textures_by_index,
            sprite_ubos_by_mip,
            is_dirty: true,
        }
    }

    fn upload_initial_contents(
        &self,
        device: &dyn GpuDevice,
        sampler_cache: &SamplerCache,
        pipelines: &AnimationPipelines,
    ) {
        let sprite_ubo_size = Mth::round_toward_i32(
            SpriteContents::UBO_SIZE,
            device.get_device_info().limits.min_uniform_offset_alignment,
        );
        let ubo_block_size = sprite_ubo_size * self.mip_level_count;
        let sampler = sampler_cache.get_clamp_to_edge_mipmaps(FilterMode::NEAREST, true);
        let static_sprites: Vec<&TextureAtlasSprite> = self
            .sprites
            .iter()
            .map(|sprite| &**sprite)
            .filter(|sprite| !sprite.is_animated())
            .collect();
        let mut buffer = vec![0; static_sprites.len() * ubo_block_size as usize];
        let encoder = device.create_command_encoder();
        let mut scratch_textures: Vec<Vec<GpuTextureView>> = Vec::new();
        for (i, sprite) in static_sprites.iter().enumerate() {
            sprite.upload_sprite_ubo(
                &mut buffer,
                i as i32 * ubo_block_size,
                self.max_mip_level,
                self.width,
                self.height,
                sprite_ubo_size,
            );
            let scratch_texture = device.create_texture(
                Some(&sprite.contents().name().to_string()),
                gpu_texture::USAGE_COPY_DST | gpu_texture::USAGE_TEXTURE_BINDING,
                GpuFormat::RGBA8_UNORM,
                sprite.contents().width(),
                sprite.contents().height(),
                1,
                self.mip_level_count,
            );
            let mut views = Vec::new();
            for level in 0..=self.max_mip_level {
                // SpriteContents.uploadFirstFrame.
                encoder.write_to_texture_image(
                    &*scratch_texture,
                    &sprite.contents().by_mip_level()[level as usize],
                    level,
                    0,
                    0,
                    0,
                );
                views.push(device.create_texture_view(&scratch_texture));
            }
            scratch_textures.push(views);
        }

        let ubo = device.create_buffer_with_data(
            Some("SpriteAnimationInfo"),
            gpu_buffer::USAGE_UNIFORM,
            &buffer,
        );
        for level in 0..self.mip_level_count {
            let mut render_pass = encoder.create_render_pass_with(
                &format!("Animate {}", self.location),
                &self.mip_views[level as usize],
                None,
                None,
                None,
            );
            render_pass.set_pipeline(&pipelines.blit);
            for (i, views) in scratch_textures.iter().enumerate() {
                render_pass.set_uniform_sampler(
                    "Sprite",
                    Some(&views[level as usize]),
                    Some(sampler),
                );
                render_pass.set_uniform_slice(
                    "SpriteAnimationInfo",
                    &gpu_buffer::slice_range(
                        &ubo,
                        i64::from(i as i32 * ubo_block_size + level * sprite_ubo_size),
                        i64::from(SpriteContents::UBO_SIZE),
                    ),
                );
                render_pass.draw(6, 1, 0, 0);
            }
        }
        drop(scratch_textures);
        self.upload_animation_frames(device, sampler_cache, pipelines);
    }

    /// `dumpContents`: each mip level as `<id>_<level>.png` and the sprites' places as `<id>.txt`,
    /// where `<id>` is the atlas's location as a debug file name.
    pub fn dump_contents(
        &self,
        device: &dyn GpuDevice,
        self_id: &Identifier,
        dir: &Path,
    ) -> std::io::Result<()> {
        let output_id = self_id.to_debug_file_name();
        let texture = self.texture.as_ref().expect("Atlas not initialized");
        for (level, image) in TextureAtlas::read_mip_levels(device, &**texture, self.max_mip_level)
            .into_iter()
            .enumerate()
        {
            image.write_to_file(&dir.join(format!("{output_id}_{level}.png")))?;
        }
        let mut output = std::fs::File::create(dir.join(format!("{output_id}.txt")))?;
        let mut sprites: Vec<&TextureAtlasSprite> =
            self.sprites.iter().map(|sprite| &**sprite).collect();
        sprites.sort_by(|a, b| a.contents().name().cmp(b.contents().name()));
        for sprite in sprites {
            writeln!(
                output,
                "{}\tx={}\ty={}\tw={}\th={}",
                sprite.contents().name(),
                sprite.get_x(),
                sprite.get_y(),
                sprite.contents().width(),
                sprite.contents().height()
            )?;
        }
        Ok(())
    }

    /// `TextureUtil.writeAsPNG`'s readback: every mip level copied into one buffer, then each read
    /// into an image as it lies in memory, rows as GL stores them.
    pub fn read_mip_levels(
        device: &dyn GpuDevice,
        texture: &dyn GpuTexture,
        max_mip_level: i32,
    ) -> Vec<NativeImage> {
        let block_size = i64::from(texture.get_format().block_size());
        let sizes: Vec<(i32, i32)> = (0..=max_mip_level)
            .map(|level| (texture.get_width(level), texture.get_height(level)))
            .collect();
        let buffer_length: i64 = sizes
            .iter()
            .map(|(width, height)| block_size * i64::from(*width) * i64::from(*height))
            .sum();
        let buffer = device.create_buffer(
            Some("Texture output buffer"),
            gpu_buffer::USAGE_MAP_READ | gpu_buffer::USAGE_COPY_DST,
            buffer_length,
        );
        let encoder = device.create_command_encoder();
        let images = Rc::new(RefCell::new(Vec::new()));
        let mut offset = 0;
        for level in 0..=max_mip_level {
            let callback = if level == max_mip_level {
                let images = images.clone();
                let buffer = buffer.clone();
                let sizes = sizes.clone();
                Box::new(move || {
                    let data = buffer.map_read();
                    let mut offset = 0usize;
                    for (width, height) in sizes {
                        let mut image = NativeImage::new(width, height);
                        for y in 0..height {
                            for x in 0..width {
                                let start = offset + ((x + y * width) as i64 * block_size) as usize;
                                let abgr = i32::from_le_bytes(
                                    data[start..start + 4].try_into().expect("four bytes"),
                                );
                                image.set_pixel_abgr(x, y, abgr);
                            }
                        }
                        offset += (block_size * i64::from(width) * i64::from(height)) as usize;
                        images.borrow_mut().push(image);
                    }
                }) as Box<dyn FnOnce()>
            } else {
                Box::new(|| {})
            };
            encoder.copy_texture_to_buffer(texture, &buffer, offset, callback, level);
            offset += block_size
                * i64::from(texture.get_width(level))
                * i64::from(texture.get_height(level));
        }
        encoder.submit();
        images.take()
    }

    /// `cycleAnimationFrames` (`tick`): every animation a tick on, then the frames that changed
    /// drawn.
    pub fn cycle_animation_frames(
        &mut self,
        device: &dyn GpuDevice,
        sampler_cache: &SamplerCache,
        pipelines: &AnimationPipelines,
    ) {
        if self.texture.is_some() {
            for animation_state in &mut self.animated_textures_states {
                animation_state.tick();
            }
            self.upload_animation_frames(device, sampler_cache, pipelines);
        }
    }

    /// Every animation back at its first frame, drawn: the oracle's scenario start, which both
    /// clients do (TESTING.md, "Scenarios"). Java has no such method; the oracle client sets the
    /// same fields and calls `uploadAnimationFrames`.
    pub fn restart_animations(
        &mut self,
        device: &dyn GpuDevice,
        sampler_cache: &SamplerCache,
        pipelines: &AnimationPipelines,
    ) {
        if self.texture.is_some() {
            for animation_state in &mut self.animated_textures_states {
                animation_state.frame = 0;
                animation_state.sub_frame = 0;
                animation_state.is_dirty = true;
            }
            self.upload_animation_frames(device, sampler_cache, pipelines);
        }
    }

    fn upload_animation_frames(
        &self,
        device: &dyn GpuDevice,
        sampler_cache: &SamplerCache,
        pipelines: &AnimationPipelines,
    ) {
        if !self
            .animated_textures_states
            .iter()
            .any(AnimationState::needs_to_draw)
        {
            return;
        }
        let sampler = sampler_cache.get_clamp_to_edge_mipmaps(FilterMode::NEAREST, true);
        let encoder = device.create_command_encoder();
        for level in 0..=self.max_mip_level {
            let mut render_pass = encoder.create_render_pass_with(
                &format!("Animate {}", self.location),
                &self.mip_views[level as usize],
                None,
                None,
                None,
            );
            for animation_state in &self.animated_textures_states {
                if animation_state.needs_to_draw() {
                    animation_state.draw_to_atlas(
                        &mut *render_pass,
                        &animation_state.sprite_ubos_by_mip[level as usize],
                        pipelines,
                        sampler,
                    );
                }
            }
        }
    }

    /// Panics where Java throws `IllegalStateException` before the atlas is uploaded.
    pub fn get_sprite(&self, location: &Identifier) -> &TextureAtlasSprite {
        self.sprites
            .iter()
            .map(|sprite| &**sprite)
            .find(|sprite| sprite.contents().name() == location)
            .or_else(|| self.missing_sprite_or_none())
            .expect("Tried to lookup sprite, but atlas is not initialized")
    }

    fn missing_sprite_or_none(&self) -> Option<&TextureAtlasSprite> {
        let missing = MissingTextureAtlasSprite::get_location();
        self.sprites
            .iter()
            .map(|sprite| &**sprite)
            .find(|sprite| *sprite.contents().name() == missing)
    }

    pub fn missing_sprite(&self) -> &TextureAtlasSprite {
        self.missing_sprite_or_none()
            .expect("Atlas not initialized")
    }

    pub fn sprites(&self) -> &[Arc<TextureAtlasSprite>] {
        &self.sprites
    }

    /// Which sprite each animation draws, for callers that list them.
    pub fn animated_sprites(&self) -> Vec<&TextureAtlasSprite> {
        self.animated_textures_states
            .iter()
            .map(|state| &*self.sprites[state.sprite])
            .collect()
    }

    pub fn clear_texture_data(&mut self) {
        self.sprites.clear();
        self.animated_textures_states.clear();
        self.sprite_ubos = None;
    }

    pub fn location(&self) -> &Identifier {
        &self.location
    }

    pub fn max_supported_texture_size(&self) -> i32 {
        self.max_supported_texture_size
    }

    pub fn get_texture(&self) -> Option<&Rc<dyn GpuTexture>> {
        self.texture.as_ref()
    }

    pub fn get_texture_view(&self) -> Option<&GpuTextureView> {
        self.texture_view.as_ref()
    }

    pub fn get_sampler(&self) -> Option<&Rc<dyn GpuSampler>> {
        self.sampler.as_ref()
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn max_mip_level(&self) -> i32 {
        self.max_mip_level
    }
}
