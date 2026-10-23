//! net/minecraft/client/renderer/texture/atlas/SpriteSource.java
//!
//! An atlas definition's source: what it adds to the atlas's sprites. Java's sources are records
//! a dispatch codec makes; here they're an enum of the kinds ported, `directory`, `single` and
//! `paletted_permutations`, which are all the block and item atlases use. `filter` and `unstitch`
//! come with the atlases that use them. A loader is what makes a sprite when the atlas is
//! stitched: a sprite's resource, or a permutation's supplier.

use crate::net::minecraft::client::renderer::texture::atlas::sources::directory_lister::DirectoryLister;
use crate::net::minecraft::client::renderer::texture::atlas::sources::paletted_permutations::{
    PalettedPermutations, PalettedSpriteSupplier,
};
use crate::net::minecraft::client::renderer::texture::atlas::sources::single_file::SingleFile;
use crate::net::minecraft::client::renderer::texture::atlas::sprite_source_list::Loader;
use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource::Resource;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use std::collections::BTreeMap;

pub fn texture_id_converter() -> FileToIdConverter {
    FileToIdConverter::new("textures", ".png")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpriteSource {
    Directory(DirectoryLister),
    Single(SingleFile),
    PalettedPermutations(PalettedPermutations),
}

impl SpriteSource {
    pub fn run(&self, resource_manager: &dyn ResourceManager, output: &mut Output) {
        match self {
            SpriteSource::Directory(source) => source.run(resource_manager, output),
            SpriteSource::Single(source) => source.run(resource_manager, output),
            SpriteSource::PalettedPermutations(source) => source.run(resource_manager, output),
        }
    }
}

/// `SpriteSource.Output`, as `SpriteSourceList.list` makes it: the sprites by id, a later one
/// replacing an earlier one. Java's map is a `HashMap`, whose order only decides the order the
/// sprites load in, which the stitcher sorts away.
#[derive(Default)]
pub struct Output {
    pub sprites: BTreeMap<Identifier, Loader>,
}

impl Output {
    pub fn add(&mut self, id: Identifier, resource: Resource) {
        self.sprites
            .insert(id.clone(), Loader::Resource(id, resource));
    }

    /// `add(Identifier, DiscardableLoader)`. A loader replaced by a later one is discarded, like
    /// Java's `discard()` in `SpriteSourceList.list`.
    pub fn add_loader(&mut self, id: Identifier, loader: PalettedSpriteSupplier) {
        if let Some(Loader::Paletted(previous)) = self.sprites.insert(id, Loader::Paletted(loader))
        {
            previous.discard();
        }
    }

    pub fn remove_all(&mut self, predicate: impl Fn(&Identifier) -> bool) {
        self.sprites.retain(|id, _| !predicate(id));
    }
}
