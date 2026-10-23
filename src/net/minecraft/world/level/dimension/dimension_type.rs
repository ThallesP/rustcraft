//! net/minecraft/world/level/dimension/DimensionType.java
//!
//! Not ported yet: everything but `STREAM_CODEC` and the built-in types' heights, whether they
//! have a sky, and how their blocks' faces are lit. A dimension type's values (its ambient light,
//! effects, the values of one a server sends) come with the registries' elements; the client
//! reads the vanilla pack's for the types the server names as known
//! (`data/minecraft/dimension_type/*.json`), whose values these have.

use std::sync::LazyLock;

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::world::level::cardinal_lighting;
use crate::net::minecraft::world::level::level_height_accessor::{self, Height};

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
    LazyLock::new(|| Box::new(ByteBufCodecs::holder_registry(Registries::DIMENSION_TYPE)));

/// The kind of a dimension: how high it is, its light, its sky.
pub struct DimensionType;

impl DimensionType {
    /// A dimension type of the synchronized registry, by its id.
    pub const STREAM_CODEC: &'static StaticStreamCodec<RegistryFriendlyByteBuf, Holder> =
        &STREAM_CODEC;

    /// A built-in dimension type's `min_y` and `height`, from the vanilla pack. Panics for any
    /// other, whose values the server would send, which aren't read yet.
    pub fn height_of(dimension_type: &Holder) -> Height {
        let Holder::Reference(key) = dimension_type;
        match key.identifier().to_string().as_str() {
            "minecraft:overworld" | "minecraft:overworld_caves" => {
                level_height_accessor::create(-64, 384)
            }
            "minecraft:the_nether" | "minecraft:the_end" => level_height_accessor::create(0, 256),
            other => panic!("the dimension type {other} isn't one of the vanilla pack's"),
        }
    }

    /// A built-in dimension type's `has_skylight`, which decides whether its level has a sky
    /// light engine. Panics like `height_of`.
    pub fn has_sky_light_of(dimension_type: &Holder) -> bool {
        let Holder::Reference(key) = dimension_type;
        match key.identifier().to_string().as_str() {
            "minecraft:overworld" | "minecraft:overworld_caves" | "minecraft:the_end" => true,
            "minecraft:the_nether" => false,
            other => panic!("the dimension type {other} isn't one of the vanilla pack's"),
        }
    }

    /// A built-in dimension type's `cardinal_light` (`default` unless it says otherwise), how its
    /// blocks' faces are shaded. Panics like `height_of`.
    pub fn cardinal_light_type_of(dimension_type: &Holder) -> cardinal_lighting::Type {
        let Holder::Reference(key) = dimension_type;
        match key.identifier().to_string().as_str() {
            "minecraft:overworld" | "minecraft:overworld_caves" | "minecraft:the_end" => {
                cardinal_lighting::Type::DEFAULT
            }
            "minecraft:the_nether" => cardinal_lighting::Type::NETHER,
            other => panic!("the dimension type {other} isn't one of the vanilla pack's"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::Value;

    use super::DimensionType;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;

    /// The heights are the vanilla pack's own data, every built-in type of it.
    #[test]
    fn heights_match_the_vanilla_pack() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vanilla")
            .join(include_str!("../../../../../../MINECRAFT_VERSION").trim())
            .join("resources/data/minecraft/dimension_type");
        let mut checked = 0;
        for entry in fs::read_dir(&directory).expect("setup_vanilla.py extracts the vanilla pack") {
            let path = entry.expect("a directory entry").path();
            let name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("a file name");
            let data: Value =
                serde_json::from_str(&fs::read_to_string(&path).expect("readable")).expect("JSON");
            let key = ResourceKey::create(
                Registries::DIMENSION_TYPE,
                Identifier::with_default_namespace(name),
            );
            let height = DimensionType::height_of(&Holder::Reference(key));
            assert_eq!(
                height.get_min_y() as i64,
                data["min_y"].as_i64().expect("min_y"),
                "{name}"
            );
            assert_eq!(
                height.get_height() as i64,
                data["height"].as_i64().expect("height"),
                "{name}"
            );
            let holder = Holder::Reference(ResourceKey::create(
                Registries::DIMENSION_TYPE,
                Identifier::with_default_namespace(name),
            ));
            assert_eq!(
                DimensionType::has_sky_light_of(&holder),
                data["has_skylight"].as_bool().expect("has_skylight"),
                "{name}"
            );
            assert_eq!(
                DimensionType::cardinal_light_type_of(&holder).get_serialized_name(),
                data.get("cardinal_light")
                    .map(|light| light.as_str().expect("a name"))
                    .unwrap_or("default"),
                "{name}"
            );
            checked += 1;
        }
        assert_eq!(checked, 4);
    }
}
