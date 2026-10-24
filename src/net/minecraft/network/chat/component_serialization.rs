//! net/minecraft/network/chat/ComponentSerialization.java
//!
//! How a component is read and written: as a string (plain text), a non-empty list (the first
//! component with the rest appended), or a map of its contents' fields, `extra` siblings and
//! style. The contents are matched by their `"type"` when there is one (the dispatch), else by
//! trying each kind's codec in turn and taking the first that reads (`FuzzyCodec`); written
//! without the type. `CODEC` is recursive, a component's siblings and arguments being components.
//!
//! The stream codecs read the component as NBT (`ByteBufCodecs.fromCodec…`), over the buffer's
//! registries for the game protocol's. `flatRestrictedCodec` comes with its first caller.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::datafixers::util::either::Either;
use crate::com::mojang::serialization::codec::{
    self, BoxedCodec, CodecExt, RecursiveRef, STRING, SharedCodec, StaticCodec,
};
use crate::com::mojang::serialization::codecs::either_codec::EitherCodec;
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::Valued;
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::map_codec::SharedMapCodec;
use crate::com::mojang::serialization::map_codec::{BoxedMapCodec, MapCodecExt};
use crate::com::mojang::serialization::map_decoder::MapDecoder;
use crate::com::mojang::serialization::map_encoder::MapEncoder;
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;
use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_contents::{ComponentContents, ContentType};
use crate::net::minecraft::network::chat::mutable_component::MutableComponent;
use crate::net::minecraft::network::chat::style;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::util::extra_codecs::{ExtraCodecs, LateBoundIdMapper};

pub struct ComponentSerialization;

static CODEC: StaticCodec<Component> =
    LazyLock::new(|| BoxedCodec::new(codec::recursive(ComponentSerialization::create_codec)));

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, Component> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::from_codec_with_registries(
        CODEC.shared(),
        NbtAccounter::default_quota,
    ))
});

static OPTIONAL_STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, Option<Component>> =
    LazyLock::new(|| Box::new(ByteBufCodecs::optional(&**STREAM_CODEC)));

static TRUSTED_STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, Component> =
    LazyLock::new(|| {
        Box::new(ByteBufCodecs::from_codec_with_registries(
            CODEC.shared(),
            NbtAccounter::unlimited_heap,
        ))
    });

static TRUSTED_OPTIONAL_STREAM_CODEC: StaticStreamCodec<
    RegistryFriendlyByteBuf,
    Option<Component>,
> = LazyLock::new(|| Box::new(ByteBufCodecs::optional(&**TRUSTED_STREAM_CODEC)));

static TRUSTED_CONTEXT_FREE_STREAM_CODEC: StaticStreamCodec<ByteBuf, Component> =
    LazyLock::new(|| {
        Box::new(ByteBufCodecs::from_codec(
            CODEC.shared(),
            NbtAccounter::unlimited_heap,
        ))
    });

impl ComponentSerialization {
    pub const CODEC: &'static StaticCodec<Component> = &CODEC;
    pub const STREAM_CODEC: &'static StaticStreamCodec<RegistryFriendlyByteBuf, Component> =
        &STREAM_CODEC;
    pub const OPTIONAL_STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        Option<Component>,
    > = &OPTIONAL_STREAM_CODEC;
    pub const TRUSTED_STREAM_CODEC: &'static StaticStreamCodec<RegistryFriendlyByteBuf, Component> =
        &TRUSTED_STREAM_CODEC;
    pub const TRUSTED_OPTIONAL_STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        Option<Component>,
    > = &TRUSTED_OPTIONAL_STREAM_CODEC;
    pub const TRUSTED_CONTEXT_FREE_STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Component> =
        &TRUSTED_CONTEXT_FREE_STREAM_CODEC;

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

    /// `createLegacyComponentMatcher`: a value by its type when there's one, else fuzzily
    /// (`StrictEither`), or by type for an ops that compresses maps. Java takes the codecs by type
    /// and the value's codec; here the type is a `LegacyType`, which knows both.
    pub fn create_legacy_component_matcher<L: LegacyType>(
        types: &LateBoundIdMapper<String, L>,
        type_field_name: &'static str,
    ) -> impl MapCodecExt<Value = L::Value> + Clone + Send + Sync + 'static {
        let compact_codec = FuzzyCodec {
            codecs: types.values(),
        };
        let ids: Vec<(String, L)> = types
            .values()
            .iter()
            .map(|value| (types.get_id(value).expect("a value has its id"), *value))
            .collect();
        let ids_back = ids.clone();
        let discriminator_codec = ExtraCodecs::id_resolver_codec(
            STRING,
            move |id: &String| {
                ids.iter()
                    .find(|(key, _)| key == id)
                    .map(|(_, value)| *value)
            },
            move |value: &L| {
                ids_back
                    .iter()
                    .find(|(_, candidate)| candidate == value)
                    .map(|(id, _)| id.clone())
            },
        )
        .field_of(type_field_name)
        .dispatch_map(L::of, |r#type: &L| r#type.codec());
        let contents_codec = StrictEither {
            type_field_name,
            typed: discriminator_codec.clone(),
            fuzzy: compact_codec,
        };
        ExtraCodecs::or_compressed_map(contents_codec, discriminator_codec)
    }
}

/// What a legacy matcher reads values by (Java's `MapCodec` of each type, and `codecGetter`): the
/// contents' types, the NBT contents' data sources' and the objects'.
pub trait LegacyType: Copy + PartialEq + fmt::Display + Send + Sync + 'static {
    type Value: 'static;

    /// The type's map codec.
    fn codec(self) -> BoxedMapCodec<Self::Value>;

    /// The type of a value, whose codec writes it.
    fn of(value: &Self::Value) -> Self;
}

impl LegacyType for ContentType {
    type Value = ComponentContents;

    fn codec(self) -> BoxedMapCodec<ComponentContents> {
        ContentType::codec(self)
    }

    fn of(value: &ComponentContents) -> ContentType {
        value.codec()
    }
}

/// `FuzzyCodec`: the first of the codecs that reads the map; written with the value's own.
#[derive(Clone)]
struct FuzzyCodec<L> {
    codecs: Vec<L>,
}

impl<L: LegacyType> Valued for FuzzyCodec<L> {
    type Value = L::Value;
}

/// Each codec's keys, without repeats.
impl<T: OpsValue, L: LegacyType> Keyable<T> for FuzzyCodec<L> {
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

impl<T: OpsValue, L: LegacyType> MapDecoder<T> for FuzzyCodec<L> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<L::Value> {
        for codec in &self.codecs {
            let result = codec.codec().decode(ops, input);
            if result.result().is_some() {
                return result;
            }
        }
        DataResult::error(|| "No matching codec found".to_owned())
    }
}

impl<T: OpsValue, L: LegacyType> MapEncoder<T> for FuzzyCodec<L> {
    fn encode(&self, input: &L::Value, ops: &dyn DynamicOps<T>, prefix: &mut dyn RecordBuilder<T>) {
        L::of(input).codec().encode(input, ops, prefix);
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
    use crate::com::google::gson::gson_builder::GsonBuilder;
    use crate::com::google::gson::json_element::JsonElement;
    use crate::com::mojang::datafixers::util::either::Either;
    use crate::com::mojang::serialization::data_result::tests::encode_result;
    use crate::com::mojang::serialization::decoder::Decoder;
    use crate::com::mojang::serialization::encoder::Encoder;
    use crate::com::mojang::serialization::json_ops::JsonOps;
    use crate::golden::{Decode, Encode, Golden};
    use crate::java::lang::number::tests::{decode_boxed, encode_boxed};
    use crate::java::util::uuid::UUID;
    use crate::net::minecraft::commands::arguments::selector::entity_selector::EntitySelector;
    use crate::net::minecraft::core::component::data_component_patch::DataComponentPatch;
    use crate::net::minecraft::nbt::nbt_ops::NbtOps;
    use crate::net::minecraft::nbt::string_tag::StringTag;
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::network::chat::click_event::ClickEvent;
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::network::chat::component_contents::ComponentContents;
    use crate::net::minecraft::network::chat::contents::data::block_data_source::BlockDataSource;
    use crate::net::minecraft::network::chat::contents::data::data_source::DataSource;
    use crate::net::minecraft::network::chat::contents::data::entity_data_source::EntityDataSource;
    use crate::net::minecraft::network::chat::contents::data::storage_data_source::StorageDataSource;
    use crate::net::minecraft::network::chat::contents::keybind_contents::KeybindContents;
    use crate::net::minecraft::network::chat::contents::nbt_contents::NbtContents;
    use crate::net::minecraft::network::chat::contents::object_contents::ObjectContents;
    use crate::net::minecraft::network::chat::contents::objects::atlas_sprite::AtlasSprite;
    use crate::net::minecraft::network::chat::contents::objects::object_info::ObjectInfo;
    use crate::net::minecraft::network::chat::contents::objects::player_sprite::PlayerSprite;
    use crate::net::minecraft::network::chat::contents::plain_text_contents::PlainTextContents;
    use crate::net::minecraft::network::chat::contents::score_contents::ScoreContents;
    use crate::net::minecraft::network::chat::contents::selector_contents::SelectorContents;
    use crate::net::minecraft::network::chat::contents::translatable_contents::{
        TranslatableContents, TranslationArgument,
    };
    use crate::net::minecraft::network::chat::font_description::FontDescription;
    use crate::net::minecraft::network::chat::formatted_text::FormattedText;
    use crate::net::minecraft::network::chat::hover_event::{EntityTooltipInfo, HoverEvent};
    use crate::net::minecraft::network::chat::mutable_component::MutableComponent;
    use crate::net::minecraft::network::chat::style::Style;
    use crate::net::minecraft::network::chat::text_color::TextColor;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::util::compilable_string::CompilableString;
    use crate::net::minecraft::world::entity::entity_types;
    use crate::net::minecraft::world::item::component::resolvable_profile::ResolvableProfile;
    use crate::net::minecraft::world::item::item::Item;
    use crate::net::minecraft::world::item::item_stack_template::ItemStackTemplate;

    /// A component as the suites give and record one: its spec (ComponentSuite).
    impl Decode for Component {
        fn decode(value: &Value) -> Self {
            component(value)
        }
    }

    impl Encode for Component {
        fn encode(&self) -> Value {
            spec(self).encode()
        }

        fn matches(&self, expected: &Value) -> bool {
            spec(self).matches(expected)
        }
    }

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
            "score" => {
                let name = spec[1].as_array().unwrap();
                let name = if name[0] == "left" {
                    Either::Left(compilable_selector(&string(&name[1])))
                } else {
                    Either::Right(string(&name[1]))
                };
                ComponentContents::Score(ScoreContents::new(name, string(&spec[2])))
            }
            "selector" => ComponentContents::Selector(SelectorContents::new(
                compilable_selector(&string(&spec[1])),
                Option::<Value>::decode(&spec[2]).map(|separator| component(&separator)),
            )),
            "nbt" => {
                let source = spec[5].as_array().unwrap();
                let data_source = match source[0].as_str().unwrap() {
                    "entity" => DataSource::Entity(EntityDataSource {
                        selector: compilable_selector(&string(&source[1])),
                    }),
                    "block" => DataSource::Block(BlockDataSource {
                        coordinates: BlockDataSource::block_pos_codec()
                            .parse(
                                &NbtOps::INSTANCE,
                                &Tag::String(StringTag::value_of(string(&source[1]))),
                            )
                            .get_or_throw(),
                    }),
                    _ => DataSource::Storage(StorageDataSource {
                        id: Identifier::parse(&string(&source[1])),
                    }),
                };
                ComponentContents::Nbt(NbtContents::new(
                    NbtContents::nbt_path_codec()
                        .parse(
                            &NbtOps::INSTANCE,
                            &Tag::String(StringTag::value_of(string(&spec[1]))),
                        )
                        .get_or_throw(),
                    spec[2].as_bool().unwrap(),
                    spec[3].as_bool().unwrap(),
                    Option::<Value>::decode(&spec[4]).map(|separator| component(&separator)),
                    data_source,
                ))
            }
            "object" => {
                let info = spec[1].as_array().unwrap();
                let info = if info[0] == "atlas" {
                    ObjectInfo::Atlas(AtlasSprite::new(
                        Identifier::parse(&string(&info[1])),
                        Identifier::parse(&string(&info[2])),
                    ))
                } else {
                    let profile = ResolvableProfile::CODEC
                        .parse(&NbtOps::INSTANCE, &Tag::decode(&info[2]))
                        .get_or_throw();
                    assert_eq!(
                        matches!(profile, ResolvableProfile::Static { .. }),
                        info[1] == "static",
                        "the profile decodes to the other kind"
                    );
                    ObjectInfo::Player(PlayerSprite::new(profile, info[3].as_bool().unwrap()))
                };
                ComponentContents::Object(ObjectContents::new(
                    info,
                    Option::<Value>::decode(&spec[2]).map(|fallback| component(&fallback)),
                ))
            }
            _ => ComponentContents::Keybind(KeybindContents::new(string(&spec[1]))),
        }
    }

    // A selector the suite made, which parses.
    fn compilable_selector(source: &str) -> CompilableString<EntitySelector> {
        EntitySelector::COMPILABLE_CODEC
            .parse(
                &NbtOps::INSTANCE,
                &Tag::String(StringTag::value_of(source.to_owned())),
            )
            .get_or_throw()
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
                "open_url" => ClickEvent::OpenUrl {
                    uri: crate::java::net::uri::URI::create(&field()),
                },
                _ => ClickEvent::Custom {
                    id: Identifier::parse(&field()),
                    payload: Option::<Tag>::decode(&click[2]),
                },
            }
        });
        let hover_event = spec[8].as_array().map(|hover| {
            if hover[0] == "show_text" {
                HoverEvent::ShowText {
                    value: component(&hover[1]),
                }
            } else if hover[0] == "show_item" {
                HoverEvent::ShowItem {
                    item: ItemStackTemplate::new(
                        Item::CODEC
                            .parse(
                                &NbtOps::INSTANCE,
                                &Tag::String(StringTag::value_of(string(&hover[1]))),
                            )
                            .get_or_throw(),
                        hover[2].as_i64().unwrap() as i32,
                        DataComponentPatch::CODEC
                            .parse(&NbtOps::INSTANCE, &Tag::decode(&hover[3]))
                            .get_or_throw(),
                    ),
                }
            } else {
                HoverEvent::ShowEntity {
                    entity: EntityTooltipInfo {
                        r#type: entity_types::REGISTRY
                            .get_value(&Identifier::parse(&string(&hover[1]))),
                        uuid: UUID::decode(&hover[2]),
                        name: Option::<Value>::decode(&hover[3]).map(|name| component(&name)),
                    },
                }
            }
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
                                Box::new((
                                    "number".to_owned(),
                                    boxed[0].clone(),
                                    NumberValue(n.clone()),
                                ))
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
            ComponentContents::Score(score) => {
                let name = match score.name() {
                    Either::Left(selector) => ("left".to_owned(), selector.source().to_owned()),
                    Either::Right(name) => ("right".to_owned(), name.clone()),
                };
                Box::new(("score".to_owned(), name, score.objective().to_owned()))
            }
            ComponentContents::Selector(selector) => Box::new((
                "selector".to_owned(),
                selector.selector().source().to_owned(),
                selector.separator().map(spec),
            )),
            ComponentContents::Nbt(nbt) => {
                let source = match nbt.data_source() {
                    DataSource::Entity(entity) => {
                        ("entity".to_owned(), entity.selector.source().to_owned())
                    }
                    DataSource::Block(block) => {
                        ("block".to_owned(), block.coordinates.source().to_owned())
                    }
                    DataSource::Storage(storage) => ("storage".to_owned(), storage.id.to_string()),
                };
                Box::new((
                    "nbt".to_owned(),
                    nbt.nbt_path().source().to_owned(),
                    nbt.interpreting(),
                    nbt.plain(),
                    nbt.separator().map(spec),
                    source,
                ))
            }
            ComponentContents::Object(object) => {
                let info: Box<dyn Encode> = match object.contents() {
                    ObjectInfo::Atlas(atlas) => Box::new((
                        "atlas".to_owned(),
                        atlas.atlas().to_string(),
                        atlas.sprite().to_string(),
                    )),
                    ObjectInfo::Player(player) => Box::new((
                        "player".to_owned(),
                        match player.player() {
                            ResolvableProfile::Static { .. } => "static".to_owned(),
                            ResolvableProfile::Dynamic { .. } => "dynamic".to_owned(),
                        },
                        ResolvableProfile::CODEC
                            .encode_start(&NbtOps::INSTANCE, player.player())
                            .get_or_throw(),
                        player.hat(),
                    )),
                };
                Box::new(("object".to_owned(), info, object.fallback().map(spec)))
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
                ClickEvent::OpenUrl { uri } => Box::new((name, uri.to_string())),
                ClickEvent::Custom { id, payload } => Box::new((name, id.to_string(), payload.clone())),
            }
        });
        let hover: Option<Box<dyn Encode>> =
            style.get_hover_event().map(|hover| -> Box<dyn Encode> {
                match hover {
                    HoverEvent::ShowText { value } => {
                        Box::new(("show_text".to_owned(), spec(value)))
                    }
                    HoverEvent::ShowItem { item } => Box::new((
                        "show_item".to_owned(),
                        item.item()
                            .unwrap_key()
                            .expect("a reference has its key")
                            .identifier()
                            .to_string(),
                        item.count(),
                        DataComponentPatch::CODEC
                            .encode_start(&NbtOps::INSTANCE, item.components())
                            .get_or_throw(),
                    )),
                    HoverEvent::ShowEntity { entity } => Box::new((
                        "show_entity".to_owned(),
                        entity_types::REGISTRY.get_key(entity.r#type).to_string(),
                        entity.uuid,
                        entity.name.as_ref().map(spec),
                    )),
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
                // The suite sets no style's font to a player's head.
                FontDescription::PlayerSprite { .. } => panic!("a player sprite font in a style"),
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
        let json = &JsonOps::INSTANCE;
        golden.check("encodeJson", |spec_value: Value| {
            let component = component(&spec_value);
            encode_result(
                &ComponentSerialization::CODEC.encode_start(json, &component),
                |element| Box::new(element.clone()),
            )
        });
        golden.check("toJson", |spec_value: Value| {
            let component = component(&spec_value);
            let element = ComponentSerialization::CODEC
                .encode_start(json, &component)
                .get_or_throw();
            GsonBuilder::new()
                .disable_html_escaping()
                .create()
                .to_json(&element)
                .unwrap_or_else(|error| panic!("{error}"))
        });
        golden.check("decodeJson", |input: JsonElement| {
            encode_result(
                &ComponentSerialization::CODEC.decode(json, &input),
                |pair| Box::new((spec(pair.get_first()), pair.get_second().clone())),
            )
        });
        golden.finish();
    }
}
