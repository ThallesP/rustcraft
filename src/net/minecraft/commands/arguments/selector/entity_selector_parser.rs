//! net/minecraft/commands/arguments/selector/EntitySelectorParser.java
//!
//! Not ported: the predicates the parse collects (`addPredicate`, `finalizePredicates`, the
//! options' tests of an entity), which only the server's search runs (`EntitySelector`), the
//! suggestions, and `allowSelectors(source)`, which reads a command source's permissions.

use std::sync::{Arc, LazyLock};

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::exceptions::dynamic_command_exception_type::DynamicCommandExceptionType;
use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::com::mojang::brigadier::message::Message;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::advancements::predicates::min_max_bounds::{
    Doubles, FloatDegrees, Ints,
};
use crate::net::minecraft::commands::arguments::selector::entity_selector::{
    EntitySelector, Order,
};
use crate::net::minecraft::commands::arguments::selector::options::entity_selector_options::EntitySelectorOptions;
use crate::net::minecraft::commands::arguments::selector::options::invertable_set_option_state::InvertableSetOptionState;
use crate::net::minecraft::commands::arguments::selector::options::set_once_option_state::SetOnceOptionState;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::contents::translatable_contents::TranslationArgument;
use crate::net::minecraft::world::entity::entity_type::EntityType;
use crate::net::minecraft::world::entity::entity_types::EntityTypes;
use crate::net::minecraft::world::phys::aabb::AABB;

fn simple(key: &str) -> SimpleCommandExceptionType {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(key)))
}

/// `Component.translatableEscape(key, argument)`, the message of a dynamic error.
pub(crate) fn translatable(key: &str, argument: String) -> Arc<dyn Message> {
    Arc::new(Component::from(Component::translatable_with_args(
        key,
        vec![TranslationArgument::String(argument)],
    )))
}

pub static ERROR_INVALID_NAME_OR_UUID: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("argument.entity.invalid"));
pub static ERROR_UNKNOWN_SELECTOR_TYPE: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|r#type| {
        translatable("argument.entity.selector.unknown", r#type)
    });
pub static ERROR_SELECTORS_NOT_ALLOWED: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("argument.entity.selector.not_allowed"));
pub static ERROR_MISSING_SELECTOR_TYPE: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("argument.entity.selector.missing"));
pub static ERROR_EXPECTED_END_OF_OPTIONS: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("argument.entity.options.unterminated"));
pub static ERROR_EXPECTED_OPTION_VALUE: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|name| {
        translatable("argument.entity.options.valueless", name)
    });

const SYNTAX_SELECTOR_START: u16 = b'@' as u16;
const SYNTAX_OPTIONS_START: u16 = b'[' as u16;
const SYNTAX_OPTIONS_END: u16 = b']' as u16;
const SYNTAX_OPTIONS_KEY_VALUE_SEPARATOR: u16 = b'=' as u16;
const SYNTAX_OPTIONS_SEPARATOR: u16 = b',' as u16;
const SYNTAX_NOT: u16 = b'!' as u16;
const SYNTAX_TAG: u16 = b'#' as u16;

/// Reads a player's name, an entity's UUID, or a selector (`@e[type=pig,limit=2]`).
pub struct EntitySelectorParser<'a> {
    reader: &'a mut StringReader,
    allow_selectors: bool,
    max_results: i32,
    includes_entities: bool,
    world_limited: bool,
    distance: Option<Doubles>,
    level: Option<Ints>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    delta_x: Option<f64>,
    delta_y: Option<f64>,
    delta_z: Option<f64>,
    rot_x: Option<FloatDegrees>,
    rot_y: Option<FloatDegrees>,
    order: Order,
    current_entity: bool,
    player_name: Option<String>,
    start_position: i32,
    entity_uuid: Option<UUID>,
    name_option: InvertableSetOptionState,
    limited_option: SetOnceOptionState,
    sorted_option: SetOnceOptionState,
    gamemode_option: InvertableSetOptionState,
    team_option: InvertableSetOptionState,
    r#type: Option<EntityType>,
    type_option: InvertableSetOptionState,
    scores_option: SetOnceOptionState,
    advancements_option: SetOnceOptionState,
    uses_selectors: bool,
}

impl<'a> EntitySelectorParser<'a> {
    pub fn new(reader: &'a mut StringReader, allow_selectors: bool) -> EntitySelectorParser<'a> {
        EntitySelectorParser {
            reader,
            allow_selectors,
            max_results: 0,
            includes_entities: false,
            world_limited: false,
            distance: None,
            level: None,
            x: None,
            y: None,
            z: None,
            delta_x: None,
            delta_y: None,
            delta_z: None,
            rot_x: None,
            rot_y: None,
            order: Order::ARBITRARY,
            current_entity: false,
            player_name: None,
            start_position: 0,
            entity_uuid: None,
            name_option: InvertableSetOptionState::default(),
            limited_option: SetOnceOptionState::default(),
            sorted_option: SetOnceOptionState::default(),
            gamemode_option: InvertableSetOptionState::default(),
            team_option: InvertableSetOptionState::default(),
            r#type: None,
            type_option: InvertableSetOptionState::default(),
            scores_option: SetOnceOptionState::default(),
            advancements_option: SetOnceOptionState::default(),
            uses_selectors: false,
        }
    }

    pub fn get_selector(&self) -> EntitySelector {
        let aabb = if self.delta_x.is_none() && self.delta_y.is_none() && self.delta_z.is_none() {
            self.distance
                .and_then(|distance| distance.max())
                .map(|max_range| {
                    AABB::new(
                        -max_range,
                        -max_range,
                        -max_range,
                        max_range + 1.0,
                        max_range + 1.0,
                        max_range + 1.0,
                    )
                })
        } else {
            Some(EntitySelectorParser::create_aabb(
                self.delta_x.unwrap_or(0.0),
                self.delta_y.unwrap_or(0.0),
                self.delta_z.unwrap_or(0.0),
            ))
        };
        EntitySelector::new(
            self.max_results,
            self.includes_entities,
            self.world_limited,
            self.distance,
            [self.x, self.y, self.z],
            aabb,
            self.order,
            self.current_entity,
            self.player_name.clone(),
            self.entity_uuid,
            self.r#type,
            self.uses_selectors,
        )
    }

    fn create_aabb(x: f64, y: f64, z: f64) -> AABB {
        let x_neg = x < 0.0;
        let y_neg = y < 0.0;
        let z_neg = z < 0.0;
        let x_min = if x_neg { x } else { 0.0 };
        let y_min = if y_neg { y } else { 0.0 };
        let z_min = if z_neg { z } else { 0.0 };
        let x_max = (if x_neg { 0.0 } else { x }) + 1.0;
        let y_max = (if y_neg { 0.0 } else { y }) + 1.0;
        let z_max = (if z_neg { 0.0 } else { z }) + 1.0;
        AABB::new(x_min, y_min, z_min, x_max, y_max, z_max)
    }

    fn parse_selector(&mut self) -> Result<(), CommandSyntaxException> {
        self.uses_selectors = true;
        if !self.reader.can_read() {
            return Err(ERROR_MISSING_SELECTOR_TYPE.create_with_context(self.reader));
        }
        let start = self.reader.get_cursor();
        let r#type = self.reader.read();
        match u8::try_from(r#type).unwrap_or(0) {
            b'a' => {
                self.max_results = EntitySelector::INFINITE;
                self.includes_entities = false;
                self.order = Order::ARBITRARY;
                self.limit_to_type(EntityTypes::PLAYER);
            }
            b'e' => {
                self.max_results = EntitySelector::INFINITE;
                self.includes_entities = true;
                self.order = Order::ARBITRARY;
            }
            b'n' => {
                self.max_results = 1;
                self.includes_entities = true;
                self.order = Order::NEAREST;
            }
            b'p' => {
                self.max_results = 1;
                self.includes_entities = false;
                self.order = Order::NEAREST;
                self.limit_to_type(EntityTypes::PLAYER);
            }
            b'r' => {
                self.max_results = 1;
                self.includes_entities = false;
                self.order = Order::RANDOM;
                self.limit_to_type(EntityTypes::PLAYER);
            }
            b's' => {
                self.max_results = 1;
                self.includes_entities = true;
                self.current_entity = true;
            }
            _ => {
                self.reader.set_cursor(start);
                let name = format!("@{}", String::from_utf16_lossy(&[r#type]));
                return Err(ERROR_UNKNOWN_SELECTOR_TYPE.create_with_context(self.reader, name));
            }
        }
        if self.reader.can_read() && self.reader.peek() == SYNTAX_OPTIONS_START {
            self.reader.skip();
            self.parse_options()?;
        }
        Ok(())
    }

    fn parse_name_or_uuid(&mut self) -> Result<(), CommandSyntaxException> {
        let start = self.reader.get_cursor();
        let name = self.reader.read_string()?;
        match UUID::from_string(&name) {
            Ok(uuid) => {
                self.entity_uuid = Some(uuid);
                self.includes_entities = true;
            }
            Err(_) => {
                if name.is_empty() || name.encode_utf16().count() > 16 {
                    self.reader.set_cursor(start);
                    return Err(ERROR_INVALID_NAME_OR_UUID.create_with_context(self.reader));
                }
                self.includes_entities = false;
                self.player_name = Some(name);
            }
        }
        self.max_results = 1;
        Ok(())
    }

    fn parse_options(&mut self) -> Result<(), CommandSyntaxException> {
        self.reader.skip_whitespace();
        while self.reader.can_read() && self.reader.peek() != SYNTAX_OPTIONS_END {
            self.reader.skip_whitespace();
            let start = self.reader.get_cursor();
            let key = self.reader.read_string()?;
            let modifier = EntitySelectorOptions::get(self, &key, start)?;
            self.reader.skip_whitespace();
            if !self.reader.can_read() || self.reader.peek() != SYNTAX_OPTIONS_KEY_VALUE_SEPARATOR {
                self.reader.set_cursor(start);
                return Err(ERROR_EXPECTED_OPTION_VALUE.create_with_context(self.reader, key));
            }
            self.reader.skip();
            self.reader.skip_whitespace();
            modifier(self)?;
            self.reader.skip_whitespace();
            if self.reader.can_read() {
                if self.reader.peek() != SYNTAX_OPTIONS_SEPARATOR {
                    if self.reader.peek() != SYNTAX_OPTIONS_END {
                        return Err(ERROR_EXPECTED_END_OF_OPTIONS.create_with_context(self.reader));
                    }
                    break;
                }
                self.reader.skip();
            }
        }
        if self.reader.can_read() {
            self.reader.skip();
            Ok(())
        } else {
            Err(ERROR_EXPECTED_END_OF_OPTIONS.create_with_context(self.reader))
        }
    }

    pub fn should_invert_value(&mut self) -> bool {
        self.reader.skip_whitespace();
        if self.reader.can_read() && self.reader.peek() == SYNTAX_NOT {
            self.reader.skip();
            self.reader.skip_whitespace();
            true
        } else {
            false
        }
    }

    pub fn is_tag(&mut self) -> bool {
        self.reader.skip_whitespace();
        if self.reader.can_read() && self.reader.peek() == SYNTAX_TAG {
            self.reader.skip();
            self.reader.skip_whitespace();
            true
        } else {
            false
        }
    }

    pub fn get_reader(&mut self) -> &mut StringReader {
        self.reader
    }

    pub fn set_world_limited(&mut self) {
        self.world_limited = true;
    }

    pub fn get_distance(&self) -> Option<Doubles> {
        self.distance
    }

    pub fn set_distance(&mut self, distance: Doubles) {
        self.distance = Some(distance);
    }

    pub fn get_level(&self) -> Option<Ints> {
        self.level
    }

    pub fn set_level(&mut self, level: Ints) {
        self.level = Some(level);
    }

    pub fn get_rot_x(&self) -> Option<FloatDegrees> {
        self.rot_x
    }

    pub fn set_rot_x(&mut self, rot_x: FloatDegrees) {
        self.rot_x = Some(rot_x);
    }

    pub fn get_rot_y(&self) -> Option<FloatDegrees> {
        self.rot_y
    }

    pub fn set_rot_y(&mut self, rot_y: FloatDegrees) {
        self.rot_y = Some(rot_y);
    }

    pub fn get_x(&self) -> Option<f64> {
        self.x
    }

    pub fn get_y(&self) -> Option<f64> {
        self.y
    }

    pub fn get_z(&self) -> Option<f64> {
        self.z
    }

    pub fn set_x(&mut self, x: f64) {
        self.x = Some(x);
    }

    pub fn set_y(&mut self, y: f64) {
        self.y = Some(y);
    }

    pub fn set_z(&mut self, z: f64) {
        self.z = Some(z);
    }

    pub fn set_delta_x(&mut self, delta_x: f64) {
        self.delta_x = Some(delta_x);
    }

    pub fn set_delta_y(&mut self, delta_y: f64) {
        self.delta_y = Some(delta_y);
    }

    pub fn set_delta_z(&mut self, delta_z: f64) {
        self.delta_z = Some(delta_z);
    }

    pub fn get_delta_x(&self) -> Option<f64> {
        self.delta_x
    }

    pub fn get_delta_y(&self) -> Option<f64> {
        self.delta_y
    }

    pub fn get_delta_z(&self) -> Option<f64> {
        self.delta_z
    }

    pub fn set_max_results(&mut self, max_results: i32) {
        self.max_results = max_results;
    }

    pub fn set_includes_entities(&mut self, includes_entities: bool) {
        self.includes_entities = includes_entities;
    }

    pub fn get_order(&self) -> Order {
        self.order
    }

    pub fn set_order(&mut self, order: Order) {
        self.order = order;
    }

    /// A selector (`@…`, where selectors are allowed), or a name or UUID.
    pub fn parse(mut self) -> Result<EntitySelector, CommandSyntaxException> {
        self.start_position = self.reader.get_cursor();
        if self.reader.can_read() && self.reader.peek() == SYNTAX_SELECTOR_START {
            if !self.allow_selectors {
                return Err(ERROR_SELECTORS_NOT_ALLOWED.create_with_context(self.reader));
            }
            self.reader.skip();
            self.parse_selector()?;
        } else {
            self.parse_name_or_uuid()?;
        }
        Ok(self.get_selector())
    }

    pub fn is_current_entity(&self) -> bool {
        self.current_entity
    }

    pub fn name_option(&mut self) -> &mut InvertableSetOptionState {
        &mut self.name_option
    }

    pub fn limited_option(&mut self) -> &mut SetOnceOptionState {
        &mut self.limited_option
    }

    pub fn sorted_option(&mut self) -> &mut SetOnceOptionState {
        &mut self.sorted_option
    }

    pub fn gamemode_option(&mut self) -> &mut InvertableSetOptionState {
        &mut self.gamemode_option
    }

    pub fn team_option(&mut self) -> &mut InvertableSetOptionState {
        &mut self.team_option
    }

    pub fn limit_to_type(&mut self, r#type: EntityType) {
        self.r#type = Some(r#type);
    }

    pub fn type_option(&mut self) -> &mut InvertableSetOptionState {
        &mut self.type_option
    }

    pub fn scores_option(&mut self) -> &mut SetOnceOptionState {
        &mut self.scores_option
    }

    pub fn advancements_option(&mut self) -> &mut SetOnceOptionState {
        &mut self.advancements_option
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::EntitySelectorParser;
    use crate::com::mojang::brigadier::string_reader::StringReader;
    use crate::com::mojang::serialization::data_result::tests::encode_result;
    use crate::com::mojang::serialization::decoder::Decoder;
    use crate::com::mojang::serialization::dynamic_ops::DynamicOps;
    use crate::golden::{Encode, Golden};
    use crate::net::minecraft::commands::arguments::selector::entity_selector::{
        EntitySelector, Order,
    };
    use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
    use crate::net::minecraft::nbt::nbt_ops::NbtOps;
    use crate::net::minecraft::nbt::string_tag::StringTag;
    use crate::net::minecraft::nbt::tag::Tag;

    fn bits(value: f64) -> Value {
        json!(format!("{:016x}", value.to_bits()))
    }

    // The suite's description of a parsed selector.
    fn describe(selector: &EntitySelector) -> Value {
        let position = selector.position();
        json!([
            selector.get_max_results(),
            selector.includes_entities(),
            selector.is_self_selector(),
            selector.is_world_limited(),
            selector.uses_selector(),
            selector.player_name(),
            selector.entity_uuid().map(|uuid| uuid.to_string()),
            selector
                .r#type()
                .map(|r#type| BuiltInRegistries::ENTITY_TYPE.get_key(r#type).to_string()),
            selector
                .range()
                .map(|range| json!([range.min().map(bits), range.max().map(bits)])),
            [
                position[0].map(bits),
                position[1].map(bits),
                position[2].map(bits)
            ],
            selector.aabb().map(|aabb| {
                json!([
                    bits(aabb.min_x),
                    bits(aabb.min_y),
                    bits(aabb.min_z),
                    bits(aabb.max_x),
                    bits(aabb.max_y),
                    bits(aabb.max_z),
                ])
            }),
            match selector.order() {
                Order::ARBITRARY => "arbitrary",
                Order::NEAREST => "nearest",
                Order::FURTHEST => "furthest",
                Order::RANDOM => "random",
            },
        ])
    }

    fn parse(input: &str, allow_selectors: bool) -> Value {
        let mut reader = StringReader::new(input);
        match EntitySelectorParser::new(&mut reader, allow_selectors).parse() {
            Ok(selector) => json!([describe(&selector), reader.get_cursor()]),
            Err(exception) => json!(["error", exception.get_message(), reader.get_cursor()]),
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.commands.arguments.selector.EntitySelectorParser");
        golden.check("parse", |input: String| parse(&input, true));
        golden.check("parseNoSelectors", |input: String| parse(&input, false));
        golden.check(
            "COMPILABLE_CODEC.decode",
            |input: String| -> Box<dyn Encode> {
                let ops: &dyn DynamicOps<Tag> = &NbtOps::INSTANCE;
                let decoded = EntitySelector::COMPILABLE_CODEC
                    .decode(ops, &Tag::String(StringTag::value_of(input.clone())))
                    .map(|pair| pair.into_first());
                encode_result(&decoded, |compilable| {
                    Box::new(Value::from(compilable.source()))
                })
            },
        );
        golden.finish();
    }
}
