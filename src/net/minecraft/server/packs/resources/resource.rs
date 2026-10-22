//! net/minecraft/server/packs/resources/Resource.java
//!
//! A resource a manager found: the pack it's from, its bytes, and its metadata, found when the
//! resource was (Java looks for the `.mcmeta` when the metadata is first asked for, which reads
//! the same file) and parsed once when it's asked for.

use crate::net::minecraft::server::packs::resources::io_supplier::IoSupplier;
use crate::net::minecraft::server::packs::resources::resource_metadata::ResourceMetadata;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct Resource {
    source_pack_id: String,
    stream_supplier: IoSupplier,
    metadata_supplier: Option<IoSupplier>,
    cached_metadata: OnceLock<Result<ResourceMetadata, String>>,
}

impl Resource {
    /// A resource whose metadata is the `.mcmeta` at `metadata_supplier`, or empty.
    pub fn new(
        source_pack_id: &str,
        stream_supplier: IoSupplier,
        metadata_supplier: Option<IoSupplier>,
    ) -> Resource {
        Resource {
            source_pack_id: source_pack_id.to_owned(),
            stream_supplier,
            metadata_supplier,
            cached_metadata: OnceLock::new(),
        }
    }

    pub fn source_pack_id(&self) -> &str {
        &self.source_pack_id
    }

    /// `open()`, read whole.
    pub fn open(&self) -> std::io::Result<Vec<u8>> {
        self.stream_supplier.get()
    }

    /// `readAllAsString()`, as UTF-8 (Java's reader replaces malformed input, which vanilla's
    /// files have none of).
    pub fn read_all_as_string(&self) -> std::io::Result<String> {
        Ok(String::from_utf8_lossy(&self.open()?).into_owned())
    }

    /// Java's `IOException` (and the metadata's `JsonParseException`) are the `Err`.
    pub fn metadata(&self) -> Result<ResourceMetadata, String> {
        self.cached_metadata
            .get_or_init(|| match &self.metadata_supplier {
                Some(supplier) => supplier
                    .get()
                    .map_err(|error| error.to_string())
                    .and_then(|bytes| ResourceMetadata::from_json_stream(&bytes)),
                None => Ok(ResourceMetadata::empty()),
            })
            .clone()
    }

    /// Where the bytes come from, for callers that report it.
    pub fn stream_supplier(&self) -> &IoSupplier {
        &self.stream_supplier
    }
}
