//! net/minecraft/resources/FileToIdConverter.java
//!
//! Not ported yet: `registry`, and `listMatchingResources` and `listMatchingResourceStacks`,
//! which come with the `ResourceManager`.

use crate::net::minecraft::resources::identifier::Identifier;

/// Turns a resource's file (`minecraft:shaders/core/gui.vsh`) into the id the game names it by
/// (`minecraft:core/gui`), and back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileToIdConverter {
    prefix: String,
    extension: String,
}

impl FileToIdConverter {
    pub fn new(prefix: &str, extension: &str) -> FileToIdConverter {
        FileToIdConverter {
            prefix: prefix.to_owned(),
            extension: extension.to_owned(),
        }
    }

    pub fn json(prefix: &str) -> FileToIdConverter {
        FileToIdConverter::new(prefix, ".json")
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn extension(&self) -> &str {
        &self.extension
    }

    pub fn id_to_file(&self, id: &Identifier) -> Identifier {
        id.with_path(&format!(
            "{}/{}{}",
            self.prefix,
            id.get_path(),
            self.extension
        ))
    }

    /// Panics where Java's `substring` throws, on a path shorter than the prefix and extension.
    /// Identifiers are ASCII, so byte indices are Java's UTF-16 ones.
    pub fn file_to_id(&self, file: &Identifier) -> Identifier {
        let path = file.get_path();
        let end = path
            .len()
            .checked_sub(self.extension.len())
            .expect("StringIndexOutOfBoundsException");
        let start = self.prefix.len() + 1;
        assert!(start <= end, "StringIndexOutOfBoundsException");
        file.with_path(&path[start..end])
    }

    pub fn extension_matches(&self, id: &Identifier) -> bool {
        id.get_path().ends_with(&self.extension)
    }

    pub fn prefix_matches(&self, id: &Identifier) -> bool {
        id.get_path().starts_with(&self.prefix)
    }

    pub fn matches(&self, id: &Identifier) -> bool {
        self.prefix_matches(id) && self.extension_matches(id)
    }
}
