//! net/minecraft/network/chat/ComponentSerialization.java
//!
//! How a component is read and written: as a string (plain text), a non-empty list (the first
//! component with the rest appended), or a map of its contents' fields, `extra` siblings and
//! style. The contents are matched by their `"type"` when there is one (the dispatch), else by
//! trying each kind's codec in turn and taking the first that reads (`FuzzyCodec`); written
//! without the type. `CODEC` is recursive, a component's siblings and arguments being components.
//!
//! The stream codecs come with the packets, and `flatRestrictedCodec` with its first caller.

use std::sync::LazyLock;

use crate::com::mojang::datafixers::util::either::Either;
use crate::com::mojang::serialization::codec::{
    self, BoxedCodec, CodecExt, RecursiveRef, STRING, StaticCodec,
};
use crate::com::mojang::serialization::codecs::either_codec::EitherCodec;
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::Valued;
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::map_codec::MapCodecExt;
use crate::com::mojang::serialization::map_codec::SharedMapCodec;
use crate::com::mojang::serialization::map_decoder::MapDecoder;
use crate::com::mojang::serialization::map_encoder::MapEncoder;
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_contents::{ComponentContents, ContentType};
use crate::net::minecraft::network::chat::mutable_component::MutableComponent;
use crate::net::minecraft::network::chat::style;
use crate::net::minecraft::util::extra_codecs::{ExtraCodecs, LateBoundIdMapper};

pub struct ComponentSerialization;

static CODEC: StaticCodec<Component> =
    LazyLock::new(|| BoxedCodec::new(codec::recursive(ComponentSerialization::create_codec)));

impl ComponentSerialization {
    pub const CODEC: &'static StaticCodec<Component> = &CODEC;

    // `createFromList`: the first component's copy, with the others appended.
    fn create_from_list(list: Vec<Component>) -> Component {
        let mut components = list.into_iter();
        let mut result = components.next().expect("a non-empty list").copy();
        for component in components {
            result.append(component);
        }
        result.into()
    }

    // `createCodec`.
    fn create_codec(top_serializer: RecursiveRef<Component>) -> BoxedCodec<Component> {
        let mut content_types = LateBoundIdMapper::new();
        ComponentSerialization::bootstrap(&mut content_types);
        let compressed_contents_codec =
            ComponentSerialization::create_legacy_component_matcher(&content_types, "type");
        let full_codec = record_codec_builder::create(
            (
                compressed_contents_codec
                    .for_getter(|component: &Component| component.get_contents().clone()),
                ExtraCodecs::non_empty_list(top_serializer.clone().list_of())
                    .optional_field_of_default("extra", Vec::new())
                    .for_getter(|component: &Component| component.get_siblings().to_vec()),
                style::Serializer::MAP_CODEC
                    .shared()
                    .for_getter(|component: &Component| component.get_style().clone()),
            ),
            |contents, siblings, style| {
                Component::from(MutableComponent::new(contents, siblings, style))
            },
        );
        EitherCodec::new(
            EitherCodec::new(
                STRING,
                ExtraCodecs::non_empty_list(top_serializer.list_of()),
            ),
            full_codec,
        )
        .xmap(
            |special_or_component| {
                special_or_component.map(
                    |special| {
                        special.map(
                            |text: String| Component::from(Component::literal(&text)),
                            ComponentSerialization::create_from_list,
                        )
                    },
                    |component| component,
                )
            },
            |component: &Component| match component.try_collapse_to_string() {
                Some(text) => Either::Left(Either::Left(text.to_owned())),
                None => Either::Right(component.clone()),
            },
        )
        .boxed()
    }

    // `bootstrap`: each kind of contents under its type's name.
    fn bootstrap(content_types: &mut LateBoundIdMapper<String, ContentType>) {
        content_types.put("text".to_owned(), ContentType::Text);
        content_types.put("translatable".to_owned(), ContentType::Translatable);
        content_types.put("keybind".to_owned(), ContentType::Keybind);
        content_types.put("score".to_owned(), ContentType::Score);
        content_types.put("selector".to_owned(), ContentType::Selector);
        content_types.put("nbt".to_owned(), ContentType::Nbt);
        content_types.put("object".to_owned(), ContentType::Object);
    }

    /// `createLegacyComponentMatcher`, of the contents: by type when there's one, else fuzzily
    /// (`StrictEither`), or by type for an ops that compresses maps.
    fn create_legacy_component_matcher(
        types: &LateBoundIdMapper<String, ContentType>,
        type_field_name: &'static str,
    ) -> impl MapCodecExt<Value = ComponentContents> + Clone + Send + Sync + 'static {
        let compact_codec = FuzzyCodec {
            codecs: types.values(),
        };
        let (by_id, by_value) = (types.values(), types.values());
        let ids: Vec<(String, ContentType)> = by_id
            .iter()
            .map(|value| (types.get_id(value).expect("a value has its id"), *value))
            .collect();
        let ids_back = ids.clone();
        let _ = by_value;
        let discriminator_codec = ExtraCodecs::id_resolver_codec(
            STRING,
            move |id: &String| {
                ids.iter()
                    .find(|(key, _)| key == id)
                    .map(|(_, value)| *value)
            },
            move |value: &ContentType| {
                ids_back
                    .iter()
                    .find(|(_, candidate)| candidate == value)
                    .map(|(id, _)| id.clone())
            },
        )
        .field_of(type_field_name)
        .dispatch_map(ComponentContents::codec, |content_type: &ContentType| {
            content_type.codec()
        });
        let contents_codec = StrictEither {
            type_field_name,
            typed: discriminator_codec.clone(),
            fuzzy: compact_codec,
        };
        ExtraCodecs::or_compressed_map(contents_codec, discriminator_codec)
    }
}

/// `FuzzyCodec`: the first of the codecs that reads the map; written with the value's own.
#[derive(Clone)]
struct FuzzyCodec {
    codecs: Vec<ContentType>,
}

impl Valued for FuzzyCodec {
    type Value = ComponentContents;
}

/// Each codec's keys, without repeats.
impl<T: OpsValue> Keyable<T> for FuzzyCodec {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        let mut keys: Vec<T> = Vec::new();
        for codec in &self.codecs {
            for key in codec.codec().keys(ops) {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
        keys
    }
}

impl<T: OpsValue> MapDecoder<T> for FuzzyCodec {
    fn decode(
        &self,
        ops: &dyn DynamicOps<T>,
        input: &dyn MapLike<T>,
    ) -> DataResult<ComponentContents> {
        for codec in &self.codecs {
            let result = codec.codec().decode(ops, input);
            if result.result().is_some() {
                return result;
            }
        }
        DataResult::error(|| "No matching codec found".to_owned())
    }
}

impl<T: OpsValue> MapEncoder<T> for FuzzyCodec {
    fn encode(
        &self,
        input: &ComponentContents,
        ops: &dyn DynamicOps<T>,
        prefix: &mut dyn RecordBuilder<T>,
    ) {
        input.codec().codec().encode(input, ops, prefix);
    }
}

/// `StrictEither`: the typed codec when the map has the type field, else the fuzzy one; written
/// with the fuzzy one.
#[derive(Clone)]
struct StrictEither<M1, M2> {
    type_field_name: &'static str,
    typed: M1,
    fuzzy: M2,
}

impl<M1, M2: Valued> Valued for StrictEither<M1, M2> {
    type Value = M2::Value;
}

impl<T: OpsValue, M1: Keyable<T>, M2: Keyable<T>> Keyable<T> for StrictEither<M1, M2> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        let mut keys: Vec<T> = Vec::new();
        for key in self.typed.keys(ops).into_iter().chain(self.fuzzy.keys(ops)) {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys
    }
}

impl<T: OpsValue, M1: MapDecoder<T, Value = M2::Value>, M2: MapDecoder<T>> MapDecoder<T>
    for StrictEither<M1, M2>
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<M2::Value> {
        if input.get_str(self.type_field_name).is_some() {
            self.typed.decode(ops, input)
        } else {
            self.fuzzy.decode(ops, input)
        }
    }
}

impl<T: OpsValue, M1: Keyable<T>, M2: MapEncoder<T>> MapEncoder<T> for StrictEither<M1, M2> {
    fn encode(
        &self,
        input: &M2::Value,
        ops: &dyn DynamicOps<T>,
        prefix: &mut dyn RecordBuilder<T>,
    ) {
        self.fuzzy.encode(input, ops, prefix);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::Value;

    use super::ComponentSerialization;
    use crate::com::mojang::serialization::data_result::tests::encode_result;
    use crate::com::mojang::serialization::decoder::Decoder;
    use crate::com::mojang::serialization::encoder::Encoder;
    use crate::golden::{Decode, Encode, Golden};
    use crate::java::lang::number::tests::{decode_boxed, encode_boxed};
    use crate::net::minecraft::nbt::nbt_ops::NbtOps;
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::network::chat::click_event::ClickEvent;
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::chat::component_contents::ComponentContents;
    use crate::net::minecraft::network::chat::contents::keybind_contents::KeybindContents;
    use crate::net::minecraft::network::chat::contents::plain_text_contents::PlainTextContents;
    use crate::net::minecraft::network::chat::contents::translatable_contents::{
        TranslatableContents, TranslationArgument,
    };
    use crate::net::minecraft::network::chat::font_description::FontDescription;
    use crate::net::minecraft::network::chat::formatted_text::FormattedText;
    use crate::net::minecraft::network::chat::hover_event::HoverEvent;
    use crate::net::minecraft::network::chat::mutable_component::MutableComponent;
    use crate::net::minecraft::network::chat::style::Style;
    use crate::net::minecraft::network::chat::text_color::TextColor;
    use crate::net::minecraft::resources::identifier::Identifier;

    // The component a spec says (ComponentSuite).
    pub(crate) fn component(spec: &Value) -> Component {
        let spec = spec.as_array().unwrap();
        let siblings = spec[2].as_array().unwrap().iter().map(component).collect();
        MutableComponent::new(contents(&spec[0]), siblings, style(&spec[1])).into()
    }

    fn string(value: &Value) -> String {
        value.as_str().unwrap().to_owned()
    }

    fn contents(spec: &Value) -> ComponentContents {
        let spec = spec.as_array().unwrap();
        match spec[0].as_str().unwrap() {
            "text" => ComponentContents::Text(PlainTextContents::create(spec[1].as_str().unwrap())),
            "translatable" => ComponentContents::Translatable(TranslatableContents::new(
                string(&spec[1]),
                spec[2].as_str().map(str::to_owned),
                spec[3].as_array().unwrap().iter().map(argument).collect(),
            )),
            _ => ComponentContents::Keybind(KeybindContents::new(string(&spec[1]))),
        }
    }

    fn argument(spec: &Value) -> TranslationArgument {
        let spec = spec.as_array().unwrap();
        match spec[0].as_str().unwrap() {
            "string" => TranslationArgument::String(string(&spec[1])),
            "number" => TranslationArgument::Number(decode_boxed(&Value::from(spec[1..].to_vec()))),
            "boolean" => TranslationArgument::Boolean(spec[1].as_bool().unwrap()),
            _ => TranslationArgument::Component(component(&spec[1])),
        }
    }

    fn style(spec: &Value) -> Style {
        let Some(spec) = spec.as_array() else {
            return Style::EMPTY;
        };
        let flag = |index: usize| spec[index].as_bool();
        let color = spec[0].as_array().map(|color| {
            if color[0] == "rgb" {
                TextColor::from_rgb(color[1].as_i64().unwrap() as i32)
            } else {
                TextColor::parse_color(color[1].as_str().unwrap()).get_or_throw()
            }
        });
        let click_event = spec[7].as_array().map(|click| {
            let field = || string(&click[1]);
            match click[0].as_str().unwrap() {
                "run_command" => ClickEvent::RunCommand { command: field() },
                "suggest_command" => ClickEvent::SuggestCommand { command: field() },
                "change_page" => ClickEvent::ChangePage {
                    page: click[1].as_i64().unwrap() as i32,
                },
                "copy_to_clipboard" => ClickEvent::CopyToClipboard { value: field() },
                "open_file" => ClickEvent::OpenFile { path: field() },
                _ => ClickEvent::Custom {
                    id: Identifier::parse(&field()),
                    payload: Option::<Tag>::decode(&click[2]),
                },
            }
        });
        let hover_event = spec[8].as_array().map(|hover| HoverEvent::ShowText {
            value: component(&hover[1]),
        });
        let font = spec[10].as_array().map(|font| {
            if font[0] == "resource" {
                FontDescription::Resource {
                    id: Identifier::parse(&string(&font[1])),
                }
            } else {
                FontDescription::AtlasSprite {
                    atlas_id: Identifier::parse(&string(&font[1])),
                    sprite_id: Identifier::parse(&string(&font[2])),
                }
            }
        });
        Style::new(
            color,
            spec[1].as_i64().map(|shadow| shadow as i32),
            flag(2),
            flag(3),
            flag(4),
            flag(5),
            flag(6),
            click_event,
            hover_event,
            spec[9].as_str().map(str::to_owned),
            font,
        )
    }

    // A component's spec, matching floats' NaNs and tags like their encodings do.
    pub(crate) fn spec(component: &Component) -> Box<dyn Encode> {
        let siblings: Vec<Box<dyn Encode>> = component.get_siblings().iter().map(spec).collect();
        Box::new((
            contents_spec(component.get_contents()),
            style_spec(component.get_style()),
            siblings,
        ))
    }

    fn contents_spec(contents: &ComponentContents) -> Box<dyn Encode> {
        match contents {
            ComponentContents::Text(text) => Box::new(("text".to_owned(), text.text().to_owned())),
            ComponentContents::Translatable(translatable) => {
                let args: Vec<Box<dyn Encode>> = translatable
                    .get_args()
                    .iter()
                    .map(|arg| -> Box<dyn Encode> {
                        match arg {
                            TranslationArgument::String(s) => {
                                Box::new(("string".to_owned(), s.clone()))
                            }
                            TranslationArgument::Number(n) => {
                                let boxed = encode_boxed(n).encode();
                                let boxed = boxed.as_array().unwrap();
                                Box::new(("number".to_owned(), boxed[0].clone(), NumberValue(*n)))
                            }
                            TranslationArgument::Boolean(b) => Box::new(("boolean".to_owned(), *b)),
                            TranslationArgument::Component(c) => {
                                Box::new(("component".to_owned(), spec(c)))
                            }
                        }
                    })
                    .collect();
                Box::new((
                    "translatable".to_owned(),
                    translatable.get_key().to_owned(),
                    translatable.get_fallback().map(str::to_owned),
                    args,
                ))
            }
            ComponentContents::Keybind(keybind) => {
                Box::new(("keybind".to_owned(), keybind.get_name().to_owned()))
            }
        }
    }

    // A number's value, encoded by its box.
    struct NumberValue(crate::java::lang::number::Number);

    impl Encode for NumberValue {
        fn encode(&self) -> Value {
            encode_boxed(&self.0).encode()[1].clone()
        }

        fn matches(&self, expected: &Value) -> bool {
            let boxed = encode_boxed(&self.0).encode();
            let name = boxed[0].clone();
            encode_boxed(&self.0).matches(&Value::from(vec![name, expected.clone()]))
        }
    }

    fn style_spec(style: &Style) -> Box<dyn Encode> {
        if style.is_empty() {
            return Box::new(Value::Null);
        }
        let color: Option<(String, Value)> = style.get_color().map(|color| {
            let serialized = color.serialize();
            if serialized.starts_with('#') {
                ("rgb".to_owned(), Value::from(color.get_value()))
            } else {
                ("named".to_owned(), Value::from(serialized))
            }
        });
        let [bold, italic, underlined, strikethrough, obfuscated] = style.flags();
        let click: Option<Box<dyn Encode>> = style.get_click_event().map(|click| -> Box<dyn Encode> {
            let action = click.action();
            let name = crate::net::minecraft::util::string_representable::StringRepresentable::get_serialized_name(&action).to_owned();
            match click {
                ClickEvent::RunCommand { command } | ClickEvent::SuggestCommand { command } => {
                    Box::new((name, command.clone()))
                }
                ClickEvent::ChangePage { page } => Box::new((name, *page)),
                ClickEvent::CopyToClipboard { value } => Box::new((name, value.clone())),
                ClickEvent::OpenFile { path } => Box::new((name, path.clone())),
                ClickEvent::Custom { id, payload } => Box::new((name, id.to_string(), payload.clone())),
            }
        });
        let hover: Option<Box<dyn Encode>> =
            style.get_hover_event().map(|hover| -> Box<dyn Encode> {
                match hover {
                    HoverEvent::ShowText { value } => {
                        Box::new(("show_text".to_owned(), spec(value)))
                    }
                }
            });
        let font: Option<Box<dyn Encode>> = style.font().map(|font| -> Box<dyn Encode> {
            match font {
                FontDescription::Resource { id } => {
                    Box::new(("resource".to_owned(), id.to_string()))
                }
                FontDescription::AtlasSprite {
                    atlas_id,
                    sprite_id,
                } => Box::new((
                    "atlas_sprite".to_owned(),
                    atlas_id.to_string(),
                    sprite_id.to_string(),
                )),
            }
        });
        Box::new((
            color,
            style.get_shadow_color(),
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
            click,
            hover,
            style.get_insertion().map(str::to_owned),
            font,
        ))
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.chat.ComponentSerialization");
        let ops = &NbtOps::INSTANCE;
        golden.check("toString", |spec_value: Value| {
            component(&spec_value).to_string()
        });
        golden.check("getString", |spec_value: Value| {
            component(&spec_value).get_string()
        });
        golden.check("encode", |spec_value: Value| {
            let component = component(&spec_value);
            encode_result(
                &ComponentSerialization::CODEC.encode_start(ops, &component),
                |tag| Box::new(tag.clone()),
            )
        });
        golden.check("decode", |input: Tag| {
            encode_result(&ComponentSerialization::CODEC.decode(ops, &input), |pair| {
                Box::new((spec(pair.get_first()), pair.get_second().clone()))
            })
        });
        golden.finish();
    }
}
