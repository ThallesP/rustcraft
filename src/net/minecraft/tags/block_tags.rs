//! net/minecraft/tags/BlockTags.java
//!
//! Only the tags an entity's movement and blocks' shapes ask about, and the ones entity types are immune to
//! (`EntityType.Builder.immuneTo`). In Java a block's tags are bound to its registry
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
    pub const BLOCKS_FLUID_FLOW: TagKey = TagKey("blocks_fluid_flow");
    pub const FALL_DAMAGE_RESETTING: TagKey = TagKey("fall_damage_resetting");
    pub const FOX_IMMUNE_TO: TagKey = TagKey("fox_immune_to");
    pub const POLAR_BEAR_IMMUNE_TO: TagKey = TagKey("polar_bear_immune_to");
    pub const SNOW_GOLEM_IMMUNE_TO: TagKey = TagKey("snow_golem_immune_to");
    pub const STRAY_IMMUNE_TO: TagKey = TagKey("stray_immune_to");
    pub const WITHER_IMMUNE_TO: TagKey = TagKey("wither_immune_to");
    pub const WITHER_SKELETON_IMMUNE_TO: TagKey = TagKey("wither_skeleton_immune_to");
    pub const DEFAULT_IMMUNE_TO: TagKey = TagKey("default_immune_to");
    pub const BARS: TagKey = TagKey("bars");
}

/// Each tag's blocks by registered name, resolved as they're first asked for.
static TAGS: LazyLock<Mutex<HashMap<String, HashSet<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// A tag of the registry whose tags are in `data/minecraft/tags/<directory>/` (`block`, `fluid`),
/// its nested tags resolved, by registered name: the vanilla data pack's, which is what a vanilla
/// server sends.
fn resolve_in(
    directory: &str,
    name: &str,
    tags: &mut HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let key = format!("{directory}/{name}");
    if let Some(entries) = tags.get(&key) {
        return entries.clone();
    }
    let path = VanillaPackResources::vanilla_jar_root()
        .join("data/minecraft/tags")
        .join(directory)
        .join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("can't read {}: {error}", path.display()));
    let json: Value = serde_json::from_str(&text).expect("a tag file is JSON");
    let mut entries = HashSet::new();
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
                entries.extend(resolve_in(directory, nested, tags));
            }
            None => {
                entries.insert(entry.to_string());
            }
        }
    }
    tags.insert(key, entries.clone());
    entries
}

/// Whether the registry's tag holds the entry (a registered name), resolving the tag the first
/// time it's asked about.
pub(crate) fn tag_contains(directory: &str, name: &str, entry: &str) -> bool {
    let mut tags = TAGS.lock().expect("no one panics holding the tags");
    if let Some(entries) = tags.get(&format!("{directory}/{name}")) {
        return entries.contains(entry);
    }
    resolve_in(directory, name, &mut tags).contains(entry)
}

impl BlockState {
    /// `is(TagKey<Block>)`, from vanilla's data pack until the server's tags are ported.
    pub fn is_tag(&self, tag: TagKey) -> bool {
        tag_contains(
            "block",
            tag.0,
            &blocks::registered_name(self.get_block()).to_string(),
        )
    }
}
