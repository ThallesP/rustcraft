//! net/minecraft/client/renderer/texture/atlas/SpriteResourceLoader.java
//!
//! `create(Set)`'s loader: a resource's metadata (with the additional sections the atlas asks
//! for) and image made into a sprite, or `None` where Java logs why and returns null.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::net::minecraft::client::renderer::texture::sprite_contents::{
    AdditionalMetadata, AdditionalMetadataType, SpriteContents,
};
use crate::net::minecraft::client::resources::metadata::animation::animation_metadata_section::AnimationMetadataSection;
use crate::net::minecraft::client::resources::metadata::animation::frame_size::FrameSize;
use crate::net::minecraft::client::resources::metadata::gui::gui_metadata_section::GuiMetadataSection;
use crate::net::minecraft::client::resources::metadata::texture::texture_metadata_section::TextureMetadataSection;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource::Resource;
use crate::net::minecraft::util::mth::Mth;

pub struct SpriteResourceLoader;

impl SpriteResourceLoader {
    pub fn load_sprite(
        sprite_location: &Identifier,
        resource: &Resource,
        additional_metadata_sections: &[AdditionalMetadataType],
    ) -> Option<SpriteContents> {
        let sections = resource.metadata().and_then(|metadata| {
            // ResourceMetadata.getTypedSections: each type the resource has, in the set's order.
            let mut additional_metadata = Vec::new();
            for section in additional_metadata_sections {
                match section {
                    AdditionalMetadataType::Gui => {
                        if let Some(gui) = metadata.get_section(&GuiMetadataSection::TYPE)? {
                            additional_metadata.push(AdditionalMetadata::Gui(gui));
                        }
                    }
                }
            }
            Ok((
                metadata.get_section(&AnimationMetadataSection::TYPE)?,
                metadata.get_section(&TextureMetadataSection::TYPE)?,
                additional_metadata,
            ))
        });
        let (animation_info, texture_info, additional_metadata) = match sections {
            Ok(sections) => sections,
            Err(error) => {
                eprintln!("Unable to parse metadata from {sprite_location}: {error}");
                return None;
            }
        };

        let image = match resource
            .open()
            .map_err(|error| error.to_string())
            .and_then(|bytes| NativeImage::read(&bytes))
        {
            Ok(image) => image,
            Err(error) => {
                eprintln!("Using missing texture, unable to load {sprite_location}: {error}");
                return None;
            }
        };

        let frame_size = match &animation_info {
            Some(animation) => {
                let frame_size =
                    animation.calculate_frame_size(image.get_width(), image.get_height());
                if !Mth::is_multiple_of(image.get_width(), frame_size.width)
                    || !Mth::is_multiple_of(image.get_height(), frame_size.height)
                {
                    eprintln!(
                        "Image {sprite_location} size {},{} is not multiple of frame size {},{}",
                        image.get_width(),
                        image.get_height(),
                        frame_size.width,
                        frame_size.height
                    );
                    return None;
                }
                frame_size
            }
            None => FrameSize {
                width: image.get_width(),
                height: image.get_height(),
            },
        };
        Some(SpriteContents::new(
            sprite_location.clone(),
            frame_size,
            image,
            animation_info,
            additional_metadata,
            texture_info,
        ))
    }
}
