//! net/minecraft/commands/arguments/selector/EntitySelector.java
//!
//! What a selector parses to. Finding the entities it selects (`findEntities`, `findPlayers` and
//! the predicates the parser collects, `checkPermissions`) is the server's: the client only parses
//! the selectors text components hold, so here's what the parse decides, without the predicates.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{BoxedCodec, StaticCodec};
use crate::java::util::uuid::UUID;
use crate::net::minecraft::advancements::predicates::min_max_bounds::Doubles;
use crate::net::minecraft::commands::arguments::selector::entity_selector_parser::EntitySelectorParser;
use crate::net::minecraft::util::compilable_string::{CompilableString, parse_command};
use crate::net::minecraft::world::entity::entity_type::EntityType;
use crate::net::minecraft::world::phys::aabb::AABB;

static COMPILABLE_CODEC: StaticCodec<CompilableString<EntitySelector>> = LazyLock::new(|| {
    CompilableString::codec(|contents| {
        parse_command(
            contents,
            |reader| EntitySelectorParser::new(reader, true).parse(),
            |original, exception| {
                format!(
                    "Invalid selector component: {original}: {}",
                    exception.get_message()
                )
            },
        )
    })
});

/// How a selector orders what it finds: Java's `BiConsumer`s, which only the server runs.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    ARBITRARY,
    NEAREST,
    FURTHEST,
    RANDOM,
}

/// A parsed selector.
#[derive(Clone, Debug)]
pub struct EntitySelector {
    max_results: i32,
    includes_entities: bool,
    world_limited: bool,
    range: Option<Doubles>,
    // The position's coordinates the selector sets, each in place of the source's.
    position: [Option<f64>; 3],
    aabb: Option<AABB>,
    order: Order,
    current_entity: bool,
    player_name: Option<String>,
    entity_uuid: Option<UUID>,
    r#type: Option<EntityType>,
    uses_selector: bool,
}

impl EntitySelector {
    pub const INFINITE: i32 = i32::MAX;
    pub const COMPILABLE_CODEC: &'static StaticCodec<CompilableString<EntitySelector>> =
        &COMPILABLE_CODEC;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_results: i32,
        includes_entities: bool,
        world_limited: bool,
        range: Option<Doubles>,
        position: [Option<f64>; 3],
        aabb: Option<AABB>,
        order: Order,
        current_entity: bool,
        player_name: Option<String>,
        entity_uuid: Option<UUID>,
        r#type: Option<EntityType>,
        uses_selector: bool,
    ) -> EntitySelector {
        EntitySelector {
            max_results,
            includes_entities,
            world_limited,
            range,
            position,
            aabb,
            order,
            current_entity,
            player_name,
            entity_uuid,
            r#type,
            uses_selector,
        }
    }

    /// The codec as a value, to make other codecs with.
    pub fn compilable_codec() -> BoxedCodec<CompilableString<EntitySelector>> {
        (*COMPILABLE_CODEC).clone()
    }

    pub fn get_max_results(&self) -> i32 {
        self.max_results
    }

    pub fn includes_entities(&self) -> bool {
        self.includes_entities
    }

    pub fn is_self_selector(&self) -> bool {
        self.current_entity
    }

    pub fn is_world_limited(&self) -> bool {
        self.world_limited
    }

    pub fn uses_selector(&self) -> bool {
        self.uses_selector
    }

    pub fn range(&self) -> Option<&Doubles> {
        self.range.as_ref()
    }

    pub fn position(&self) -> [Option<f64>; 3] {
        self.position
    }

    pub fn aabb(&self) -> Option<&AABB> {
        self.aabb.as_ref()
    }

    pub fn order(&self) -> Order {
        self.order
    }

    pub fn player_name(&self) -> Option<&str> {
        self.player_name.as_deref()
    }

    pub fn entity_uuid(&self) -> Option<UUID> {
        self.entity_uuid
    }

    pub fn r#type(&self) -> Option<EntityType> {
        self.r#type
    }
}
