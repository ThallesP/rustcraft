//! net/minecraft/tags/BlockTags.java
//!
//! Only the tags an entity's movement asks about. In Java a block's tags are bound to its registry
//! holder from what the server sends (`ClientboundUpdateTagsPacket`, in configuration), which
//! isn't ported yet. A vanilla server sends its data pack's, so until the packet comes,
//! `BlockState::is_tag` answers from vanilla's data pack (`data/minecraft/tags/block/` in
//! `vanilla/<version>/resources/`, nested tags resolved), read once.

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

use serde_json::Value;

use crate::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;
use crate::net::minecraft::world::level::block::blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

/// `TagKey<Block>`: a block tag's name, under `minecraft:`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TagKey(pub &'static str);

pub struct BlockTags;

impl BlockTags {
    pub const CLIMBABLE: TagKey = TagKey("climbable");
    pub const FENCES: TagKey = TagKey("fences");
    pub const WALLS: TagKey = TagKey("walls");
    pub const SUPPRESSES_BOUNCE: TagKey = TagKey("suppresses_bounce");
    pub const CAN_GLIDE_THROUGH: TagKey = TagKey("can_glide_through");
    pub const CAUSES_SUFFOCATION: TagKey = TagKey("causes_suffocation");
}

/// Each tag's blocks by registered name, resolved as they're first asked for.
static TAGS: LazyLock<Mutex<HashMap<String, HashSet<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn resolve(name: &str, tags: &mut HashMap<String, HashSet<String>>) -> HashSet<String> {
    if let Some(blocks) = tags.get(name) {
        return blocks.clone();
    }
    let path = VanillaPackResources::vanilla_jar_root()
        .join("data/minecraft/tags/block")
        .join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("can't read {}: {error}", path.display()));
    let json: Value = serde_json::from_str(&text).expect("a tag file is JSON");
    let mut blocks = HashSet::new();
    for value in json["values"].as_array().expect("a tag has values") {
        let entry = match value {
            Value::String(entry) => entry.as_str(),
            entry => entry["id"]
                .as_str()
                .expect("an entry is a name or has an id"),
        };
        match entry.strip_prefix('#') {
            Some(nested) => {
                let nested = nested.strip_prefix("minecraft:").unwrap_or(nested);
                blocks.extend(resolve(nested, tags));
            }
            None => {
                blocks.insert(entry.to_string());
            }
        }
    }
    tags.insert(name.to_string(), blocks.clone());
    blocks
}

impl BlockState {
    /// `is(TagKey<Block>)`, from vanilla's data pack until the server's tags are ported.
    pub fn is_tag(&self, tag: TagKey) -> bool {
        let mut tags = TAGS.lock().expect("no one panics holding the tags");
        resolve(tag.0, &mut tags).contains(&blocks::registered_name(self.get_block()).to_string())
    }
}
