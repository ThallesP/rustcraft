//! net/minecraft/commands/arguments/selector/options/EntitySelectorOptions.java
//!
//! Each option's reading of its value and when it applies, which decide whether a selector parses.
//! Not ported: the tests each option adds of an entity (its predicate, the server's), the options'
//! descriptions and `suggestNames`. Java registers the options in a `HashMap` at bootstrap, whose
//! order only the suggestions read; here `get` finds them by name.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::exceptions::dynamic_command_exception_type::DynamicCommandExceptionType;
use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::net::minecraft::advancements::predicates::min_max_bounds::{
    Doubles, FloatDegrees, Ints,
};
use crate::net::minecraft::commands::arguments::selector::entity_selector::Order;
use crate::net::minecraft::commands::arguments::selector::entity_selector_parser::{
    EntitySelectorParser, translatable,
};
use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
use crate::net::minecraft::nbt::tag_parser::TagParser;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::entity::entity_types::EntityTypes;
use crate::net::minecraft::world::level::game_type::GameType;

fn simple(key: &str) -> SimpleCommandExceptionType {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(key)))
}

pub static ERROR_UNKNOWN_OPTION: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|name| translatable("argument.entity.options.unknown", name));
pub static ERROR_INAPPLICABLE_OPTION: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|name| {
        translatable("argument.entity.options.inapplicable", name)
    });
pub static ERROR_RANGE_NEGATIVE: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("argument.entity.options.distance.negative"));
pub static ERROR_LEVEL_NEGATIVE: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("argument.entity.options.level.negative"));
pub static ERROR_LIMIT_TOO_SMALL: LazyLock<SimpleCommandExceptionType> =
    LazyLock::new(|| simple("argument.entity.options.limit.toosmall"));
pub static ERROR_SORT_UNKNOWN: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|name| {
        translatable("argument.entity.options.sort.irreversible", name)
    });
pub static ERROR_GAME_MODE_INVALID: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|name| {
        translatable("argument.entity.options.mode.invalid", name)
    });
pub static ERROR_ENTITY_TYPE_INVALID: DynamicCommandExceptionType<String> =
    DynamicCommandExceptionType::new(|r#type| {
        translatable("argument.entity.options.type.invalid", r#type)
    });

/// What an option does with its value: `Modifier.handle`.
pub type Modifier = fn(&mut EntitySelectorParser) -> Result<(), CommandSyntaxException>;

/// An option's handler, and when a selector can take it (`canUse`).
struct Option_ {
    modifier: Modifier,
    can_use: fn(&mut EntitySelectorParser) -> bool,
}

const LEFT_BRACE: u16 = b'{' as u16;
const RIGHT_BRACE: u16 = b'}' as u16;
const EQUALS: u16 = b'=' as u16;
const COMMA: u16 = b',' as u16;

static OPTIONS: LazyLock<HashMap<&'static str, Option_>> = LazyLock::new(|| {
    let mut options: HashMap<&'static str, Option_> = HashMap::new();
    let mut register = |name, modifier, can_use| {
        options.insert(name, Option_ { modifier, can_use });
    };
    register("name", name as Modifier, |s| {
        s.name_option().can_parse_any()
    });
    register("distance", distance, |s| s.get_distance().is_none());
    register("level", level, |s| s.get_level().is_none());
    register(
        "x",
        |parser| {
            parser.set_world_limited();
            let x = parser.get_reader().read_double()?;
            parser.set_x(x);
            Ok(())
        },
        |s| s.get_x().is_none(),
    );
    register(
        "y",
        |parser| {
            parser.set_world_limited();
            let y = parser.get_reader().read_double()?;
            parser.set_y(y);
            Ok(())
        },
        |s| s.get_y().is_none(),
    );
    register(
        "z",
        |parser| {
            parser.set_world_limited();
            let z = parser.get_reader().read_double()?;
            parser.set_z(z);
            Ok(())
        },
        |s| s.get_z().is_none(),
    );
    register(
        "dx",
        |parser| {
            parser.set_world_limited();
            let delta_x = parser.get_reader().read_double()?;
            parser.set_delta_x(delta_x);
            Ok(())
        },
        |s| s.get_delta_x().is_none(),
    );
    register(
        "dy",
        |parser| {
            parser.set_world_limited();
            let delta_y = parser.get_reader().read_double()?;
            parser.set_delta_y(delta_y);
            Ok(())
        },
        |s| s.get_delta_y().is_none(),
    );
    register(
        "dz",
        |parser| {
            parser.set_world_limited();
            let delta_z = parser.get_reader().read_double()?;
            parser.set_delta_z(delta_z);
            Ok(())
        },
        |s| s.get_delta_z().is_none(),
    );
    register(
        "x_rotation",
        |parser| {
            let rot_x = FloatDegrees::from_reader(parser.get_reader())?;
            parser.set_rot_x(rot_x);
            Ok(())
        },
        |s| s.get_rot_x().is_none(),
    );
    register(
        "y_rotation",
        |parser| {
            let rot_y = FloatDegrees::from_reader(parser.get_reader())?;
            parser.set_rot_y(rot_y);
            Ok(())
        },
        |s| s.get_rot_y().is_none(),
    );
    register("limit", limit, |s| {
        !s.is_current_entity() && s.limited_option().can_parse()
    });
    register("sort", sort, |s| {
        !s.is_current_entity() && s.sorted_option().can_parse()
    });
    register("gamemode", gamemode, |s| {
        s.gamemode_option().can_parse_any()
    });
    register("team", team, |s| s.team_option().can_parse_any());
    register("type", r#type, |s| s.type_option().can_parse_any());
    register(
        "tag",
        |parser| {
            parser.should_invert_value();
            parser.get_reader().read_unquoted_string();
            Ok(())
        },
        |_| true,
    );
    register(
        "nbt",
        |parser| {
            parser.should_invert_value();
            TagParser::parse_compound_as_argument(parser.get_reader())?;
            Ok(())
        },
        |_| true,
    );
    register("scores", scores, |s| s.scores_option().can_parse());
    register("advancements", advancements, |s| {
        s.advancements_option().can_parse()
    });
    register(
        "predicate",
        |parser| {
            parser.should_invert_value();
            Identifier::read_from(parser.get_reader())?;
            Ok(())
        },
        |_| true,
    );
    options
});

fn name(parser: &mut EntitySelectorParser) -> Result<(), CommandSyntaxException> {
    let start = parser.get_reader().get_cursor();
    let inverted = parser.should_invert_value();
    parser.get_reader().read_string()?;
    if !parser.name_option().can_parse_element(inverted) {
        return Err(rollback_and_throw_dynamic(
            parser,
            start,
            &ERROR_INAPPLICABLE_OPTION,
            "name",
        ));
    }
    parser.name_option().mark_parsed_element(inverted);
    Ok(())
}

fn distance(parser: &mut EntitySelectorParser) -> Result<(), CommandSyntaxException> {
    let start = parser.get_reader().get_cursor();
    let value = Doubles::from_reader(parser.get_reader())?;
    if value.min().is_some_and(|min| min < 0.0) || value.max().is_some_and(|max| max < 0.0) {
        return Err(rollback_and_throw(parser, start, &ERROR_RANGE_NEGATIVE));
    }
    parser.set_distance(value);
    parser.set_world_limited();
    Ok(())
}

fn level(parser: &mut EntitySelectorParser) -> Result<(), CommandSyntaxException> {
    let start = parser.get_reader().get_cursor();
    let value = Ints::from_reader(parser.get_reader())?;
    if value.min().is_some_and(|min| min < 0) || value.max().is_some_and(|max| max < 0) {
        return Err(rollback_and_throw(parser, start, &ERROR_LEVEL_NEGATIVE));
    }
    parser.set_level(value);
    parser.set_includes_entities(false);
    Ok(())
}

fn limit(parser: &mut EntitySelectorParser) -> Result<(), CommandSyntaxException> {
    let start = parser.get_reader().get_cursor();
    let count = parser.get_reader().read_int()?;
    if count < 1 {
        return Err(rollback_and_throw(parser, start, &ERROR_LIMIT_TOO_SMALL));
    }
    parser.set_max_results(count);
    parser.limited_option().mark_parsed();
    Ok(())
}

fn sort(parser: &mut EntitySelectorParser) -> Result<(), CommandSyntaxException> {
    let start = parser.get_reader().get_cursor();
    let name = parser.get_reader().read_unquoted_string();
    let order = match name.as_str() {
        "nearest" => Order::NEAREST,
        "furthest" => Order::FURTHEST,
        "random" => Order::RANDOM,
        "arbitrary" => Order::ARBITRARY,
        _ => {
            return Err(rollback_and_throw_dynamic(
                parser,
                start,
                &ERROR_SORT_UNKNOWN,
                &name,
            ));
        }
    };
    parser.set_order(order);
    parser.sorted_option().mark_parsed();
    Ok(())
}

fn gamemode(parser: &mut EntitySelectorParser) -> Result<(), CommandSyntaxException> {
    let start = parser.get_reader().get_cursor();
    let inverted = parser.should_invert_value();
    if !parser.gamemode_option().can_parse_element(inverted) {
        return Err(rollback_and_throw_dynamic(
            parser,
            start,
            &ERROR_INAPPLICABLE_OPTION,
            "gamemode",
        ));
    }
    let name = parser.get_reader().read_unquoted_string();
    if GameType::by_name(&name).is_none() {
        return Err(rollback_and_throw_dynamic(
            parser,
            start,
            &ERROR_GAME_MODE_INVALID,
            &name,
        ));
    }
    parser.set_includes_entities(false);
    parser.gamemode_option().mark_parsed_element(inverted);
    Ok(())
}

fn team(parser: &mut EntitySelectorParser) -> Result<(), CommandSyntaxException> {
    let start = parser.get_reader().get_cursor();
    let inverted = parser.should_invert_value();
    parser.get_reader().read_unquoted_string();
    if !parser.team_option().can_parse_element(inverted) {
        return Err(rollback_and_throw_dynamic(
            parser,
            start,
            &ERROR_INAPPLICABLE_OPTION,
            "team",
        ));
    }
    parser.team_option().mark_parsed_element(inverted);
    Ok(())
}

fn r#type(parser: &mut EntitySelectorParser) -> Result<(), CommandSyntaxException> {
    let start = parser.get_reader().get_cursor();
    let inverted = parser.should_invert_value();
    if parser.is_tag() {
        if !parser.type_option().can_parse_any_tag() {
            return Err(rollback_and_throw_dynamic(
                parser,
                start,
                &ERROR_INAPPLICABLE_OPTION,
                "type",
            ));
        }
        let id = Identifier::read_from(parser.get_reader())?;
        if !parser.type_option().can_parse_tag(&id) {
            return Err(rollback_and_throw_dynamic(
                parser,
                start,
                &ERROR_INAPPLICABLE_OPTION,
                "type",
            ));
        }
        parser.type_option().mark_parsed_tag(id);
    } else {
        if !parser.type_option().can_parse_element(inverted) {
            return Err(rollback_and_throw_dynamic(
                parser,
                start,
                &ERROR_INAPPLICABLE_OPTION,
                "type",
            ));
        }
        let id = Identifier::read_from(parser.get_reader())?;
        let Some(r#type) = BuiltInRegistries::ENTITY_TYPE.get_optional(&id) else {
            return Err(rollback_and_throw_dynamic(
                parser,
                start,
                &ERROR_ENTITY_TYPE_INVALID,
                &id.to_string(),
            ));
        };
        if r#type == EntityTypes::PLAYER && !inverted {
            parser.set_includes_entities(false);
        }
        if !inverted {
            parser.limit_to_type(r#type);
        }
        parser.type_option().mark_parsed_element(inverted);
    }
    Ok(())
}

fn scores(parser: &mut EntitySelectorParser) -> Result<(), CommandSyntaxException> {
    let reader = parser.get_reader();
    reader.expect(LEFT_BRACE)?;
    reader.skip_whitespace();
    while reader.can_read() && reader.peek() != RIGHT_BRACE {
        reader.skip_whitespace();
        reader.read_unquoted_string();
        reader.skip_whitespace();
        reader.expect(EQUALS)?;
        reader.skip_whitespace();
        Ints::from_reader(reader)?;
        reader.skip_whitespace();
        if reader.can_read() && reader.peek() == COMMA {
            reader.skip();
        }
    }
    reader.expect(RIGHT_BRACE)?;
    parser.scores_option().mark_parsed();
    Ok(())
}

fn advancements(parser: &mut EntitySelectorParser) -> Result<(), CommandSyntaxException> {
    let reader = parser.get_reader();
    let mut expected = 0;
    reader.expect(LEFT_BRACE)?;
    reader.skip_whitespace();
    while reader.can_read() && reader.peek() != RIGHT_BRACE {
        reader.skip_whitespace();
        Identifier::read_from(reader)?;
        reader.skip_whitespace();
        reader.expect(EQUALS)?;
        reader.skip_whitespace();
        if reader.can_read() && reader.peek() == LEFT_BRACE {
            reader.skip_whitespace();
            reader.expect(LEFT_BRACE)?;
            reader.skip_whitespace();
            while reader.can_read() && reader.peek() != RIGHT_BRACE {
                reader.skip_whitespace();
                reader.read_unquoted_string();
                reader.skip_whitespace();
                reader.expect(EQUALS)?;
                reader.skip_whitespace();
                reader.read_boolean()?;
                reader.skip_whitespace();
                if reader.can_read() && reader.peek() == COMMA {
                    reader.skip();
                }
            }
            reader.skip_whitespace();
            reader.expect(RIGHT_BRACE)?;
            reader.skip_whitespace();
        } else {
            reader.read_boolean()?;
        }
        expected += 1;
        reader.skip_whitespace();
        if reader.can_read() && reader.peek() == COMMA {
            reader.skip();
        }
    }
    reader.expect(RIGHT_BRACE)?;
    // Java keys the advancements in a map, and restricts the selector to players once it's
    // not empty.
    if expected > 0 {
        parser.set_includes_entities(false);
    }
    parser.advancements_option().mark_parsed();
    Ok(())
}

fn rollback_and_throw(
    parser: &mut EntitySelectorParser,
    start: i32,
    r#type: &SimpleCommandExceptionType,
) -> CommandSyntaxException {
    parser.get_reader().set_cursor(start);
    r#type.create_with_context(parser.get_reader())
}

fn rollback_and_throw_dynamic(
    parser: &mut EntitySelectorParser,
    start: i32,
    r#type: &DynamicCommandExceptionType<String>,
    argument: &str,
) -> CommandSyntaxException {
    parser.get_reader().set_cursor(start);
    r#type.create_with_context(parser.get_reader(), argument.to_owned())
}

/// The options' handlers and checks.
pub struct EntitySelectorOptions;

impl EntitySelectorOptions {
    /// The handler of the option named `key`, if the selector can take it; else the error, at the
    /// key's `start`.
    pub fn get(
        parser: &mut EntitySelectorParser,
        key: &str,
        start: i32,
    ) -> Result<Modifier, CommandSyntaxException> {
        match OPTIONS.get(key) {
            Some(option) if (option.can_use)(parser) => Ok(option.modifier),
            Some(_) => Err(rollback_and_throw_dynamic(
                parser,
                start,
                &ERROR_INAPPLICABLE_OPTION,
                key,
            )),
            None => Err(rollback_and_throw_dynamic(
                parser,
                start,
                &ERROR_UNKNOWN_OPTION,
                key,
            )),
        }
    }
}
