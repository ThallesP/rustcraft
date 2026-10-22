//! net/minecraft/server/packs/resources/ResourceMetadata.java and
//! net/minecraft/server/packs/metadata/MetadataSectionType.java
//!
//! A resource's `.mcmeta`: a JSON object whose members are sections, each read by its type's codec
//! when it's asked for. Java's codecs are DataFixerUpper's `Codec`s over Gson's JSON; until those
//! are ported, a section type's codec is a function decoding the section's JSON (serde_json's), and
//! a failure is the `Err` (Java's `JsonParseException`) with the codec's message where the port
//! gives one. `MapBased`, the built-in metadata, comes with the pack repository.

use serde_json::{Map, Value};

/// `MetadataSectionType`: a section's name and how to decode it.
pub struct MetadataSectionType<T> {
    pub name: &'static str,
    pub codec: fn(&Value) -> Result<T, String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResourceMetadata {
    metadata: Map<String, Value>,
}

impl ResourceMetadata {
    /// `EMPTY`.
    pub fn empty() -> ResourceMetadata {
        ResourceMetadata::default()
    }

    /// `fromJsonStream`: the bytes as a JSON object (`GsonHelper.parse`). Java's `IOException`
    /// and `JsonParseException` are the `Err`.
    pub fn from_json_stream(bytes: &[u8]) -> Result<ResourceMetadata, String> {
        match serde_json::from_slice::<Value>(bytes) {
            Ok(Value::Object(metadata)) => Ok(ResourceMetadata { metadata }),
            Ok(_) => Err("Expected a JSON object".to_owned()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn get_section<T>(&self, serializer: &MetadataSectionType<T>) -> Result<Option<T>, String> {
        match self.metadata.get(serializer.name) {
            Some(raw_section) => (serializer.codec)(raw_section).map(Some),
            None => Ok(None),
        }
    }
}
