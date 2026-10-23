//! net/minecraft/client/renderer/texture/atlas/SpriteSourceList.java
//!
//! An atlas's definition, `atlases/<atlas>.json` in every pack that has one, bottom first. Java
//! decodes it with `SpriteSources.FILE_CODEC` (DataFixerUpper, over Gson's strict parse); the port
//! decodes the kinds `SpriteSource` has by hand, and logs a definition it can't read like Java.

use crate::net::minecraft::client::renderer::texture::atlas::sources::directory_lister::DirectoryLister;
use crate::net::minecraft::client::renderer::texture::atlas::sources::paletted_permutations::{
    DEFAULT_SEPARATOR, PalettedPermutations, PalettedSpriteSupplier,
};
use crate::net::minecraft::client::renderer::texture::atlas::sources::single_file::SingleFile;
use crate::net::minecraft::client::renderer::texture::atlas::sprite_resource_loader::SpriteResourceLoader;
use crate::net::minecraft::client::renderer::texture::atlas::sprite_source::{
    Output, SpriteSource,
};
use crate::net::minecraft::client::renderer::texture::missing_texture_atlas_sprite::MissingTextureAtlasSprite;
use crate::net::minecraft::client::renderer::texture::sprite_contents::SpriteContents;
use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource::Resource;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use serde_json::Value;

pub struct SpriteSourceList {
    sources: Vec<SpriteSource>,
}

/// `SpriteSource.Loader`: the missing sprite, a sprite's resource, or a permutation's supplier.
pub enum Loader {
    Missing,
    Resource(Identifier, Resource),
    Paletted(PalettedSpriteSupplier),
}

impl Loader {
    pub fn get(&self) -> Option<SpriteContents> {
        match self {
            Loader::Missing => Some(MissingTextureAtlasSprite::create()),
            Loader::Resource(id, resource) => SpriteResourceLoader::load_sprite(id, resource),
            Loader::Paletted(supplier) => supplier.get(),
        }
    }
}

impl SpriteSourceList {
    /// The missing sprite first, then every sprite the sources add.
    pub fn list(&self, resource_manager: &dyn ResourceManager) -> Vec<Loader> {
        let mut output = Output::default();
        for source in &self.sources {
            source.run(resource_manager, &mut output);
        }
        let mut result = vec![Loader::Missing];
        result.extend(output.sprites.into_values());
        result
    }

    pub fn load(resource_manager: &dyn ResourceManager, atlas_id: &Identifier) -> SpriteSourceList {
        let resource_id = FileToIdConverter::json("atlases").id_to_file(atlas_id);
        let mut loaders = Vec::new();
        for entry in resource_manager.get_resource_stack(&resource_id) {
            let parsed = entry
                .open()
                .map_err(|error| error.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<Value>(&bytes).map_err(|error| error.to_string())
                })
                .and_then(|contents| SpriteSourceList::file_codec(&contents));
            match parsed {
                Ok(sources) => loaders.extend(sources),
                Err(error) => eprintln!(
                    "Failed to parse atlas definition {resource_id} in pack {}: {error}",
                    entry.source_pack_id()
                ),
            }
        }
        SpriteSourceList { sources: loaders }
    }

    /// `SpriteSources.FILE_CODEC`: `{"sources": [{"type": …, …}, …]}`.
    fn file_codec(contents: &Value) -> Result<Vec<SpriteSource>, String> {
        let sources = contents
            .get("sources")
            .and_then(Value::as_array)
            .ok_or("No key sources in MapLike")?;
        sources.iter().map(SpriteSourceList::source_codec).collect()
    }

    fn source_codec(source: &Value) -> Result<SpriteSource, String> {
        let string = |key: &str| {
            source
                .get(key)
                .and_then(Value::as_str)
                .ok_or_else(|| format!("No key {key} in MapLike"))
        };
        let identifier = |text: &str| {
            Identifier::try_parse(text)
                .ok_or_else(|| format!("Not a valid resource location: {text}"))
        };
        let kind = identifier(string("type")?)?;
        if kind.get_namespace() != "minecraft" {
            return Err(format!("Unknown element id: {kind}"));
        }
        match kind.get_path() {
            "directory" => Ok(SpriteSource::Directory(DirectoryLister {
                source_path: string("source")?.to_owned(),
                id_prefix: string("prefix")?.to_owned(),
            })),
            "single" => Ok(SpriteSource::Single(SingleFile {
                resource_id: identifier(string("resource")?)?,
                sprite_id: source
                    .get("sprite")
                    .and_then(Value::as_str)
                    .map(identifier)
                    .transpose()?,
            })),
            "paletted_permutations" => {
                let identifiers = |key: &str| {
                    source
                        .get(key)
                        .and_then(Value::as_array)
                        .ok_or_else(|| format!("No key {key} in MapLike"))?
                        .iter()
                        .map(|value| identifier(value.as_str().ok_or("Not a string")?))
                        .collect::<Result<Vec<_>, String>>()
                };
                let permutations = source
                    .get("permutations")
                    .and_then(Value::as_object)
                    .ok_or("No key permutations in MapLike")?
                    .iter()
                    .map(|(suffix, palette)| {
                        Ok((
                            suffix.clone(),
                            identifier(palette.as_str().ok_or("Not a string")?)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Ok(SpriteSource::PalettedPermutations(PalettedPermutations {
                    textures: identifiers("textures")?,
                    palette_key: identifier(string("palette_key")?)?,
                    permutations,
                    separator: source
                        .get("separator")
                        .map(|separator| separator.as_str().ok_or("Not a string"))
                        .transpose()?
                        .unwrap_or(DEFAULT_SEPARATOR)
                        .to_owned(),
                }))
            }
            _ => Err(format!("Unknown element id: {kind}")),
        }
    }
}
