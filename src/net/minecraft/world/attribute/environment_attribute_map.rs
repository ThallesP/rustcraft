//! net/minecraft/world/attribute/EnvironmentAttributeMap.java
//!
//! What a dimension type or a biome does to attributes: an entry each, a modifier and its
//! argument. `CODEC` and `NETWORK_CODEC` are `from_json`, decoded by hand from Gson's tree like
//! their dispatched map: each attribute's entry is its value (an override) or `{"modifier",
//! "argument"}`. The client reads its known pack's dimension types and biomes through
//! `NETWORK_CODEC`, which keeps the syncable attributes only. Attributes whose types aren't
//! ported are left out (`EnvironmentAttributes::UNPORTED`). `hashCode` and `toString` aren't
//! ported: Java's maps of these (`SpatialAttributeInterpolator`) go by identity.
//!
//! `NETWORK_CODEC` as a codec (`Biome.NETWORK_CODEC`'s and `DimensionType.NETWORK_CODEC`'s
//! `attributes`) decodes through `from_json`, its input converted to JSON (`convertTo`), which
//! reads every value the attribute types take the same as their codecs would from NBT. Its
//! encoder, which the client never runs, isn't ported, but for the empty map.

use std::sync::LazyLock;

use crate::com::google::gson::json_element::{JsonElement, JsonObject};
use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::{CodecExt, StaticCodec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{self, DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::com::mojang::serialization::json_ops::JsonOps;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::attribute::attribute_type::AttributeValue;
use crate::net::minecraft::world::attribute::environment_attribute::EnvironmentAttribute;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::attribute::modifier::attribute_modifier::{
    AttributeModifier, OperationId,
};

#[derive(Clone, Debug, Default)]
pub struct EnvironmentAttributeMap {
    // Each attribute's entry, by the attribute's place, in no order Java's `Map.copyOf` keeps.
    entries: Vec<(usize, Entry)>,
}

/// `EnvironmentAttributeMap.Entry`.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub argument: AttributeValue,
    pub modifier: AttributeModifier,
}

impl Entry {
    pub fn apply_modifier(&self, subject: AttributeValue) -> AttributeValue {
        self.modifier.apply(subject, &self.argument)
    }
}

/// `equals`: the same entries, in any order.
impl PartialEq for EnvironmentAttributeMap {
    fn eq(&self, other: &EnvironmentAttributeMap) -> bool {
        self.entries.len() == other.entries.len()
            && self.entries.iter().all(|(id, entry)| {
                other.get(*id).is_some_and(|other| {
                    other.modifier == entry.modifier
                        && attribute_value_equals(&other.argument, &entry.argument)
                })
            })
    }
}

/// A record's `equals` of the values: floats by `Float.compare`, vectors by JOML's
/// `floatToIntBits`, which are the same.
fn attribute_value_equals(a: &AttributeValue, b: &AttributeValue) -> bool {
    let bits = |value: f32| crate::java::lang::float::Float::float_to_int_bits(value);
    match (a, b) {
        (AttributeValue::Boolean(a), AttributeValue::Boolean(b)) => a == b,
        (AttributeValue::Float(a), AttributeValue::Float(b)) => bits(*a) == bits(*b),
        (AttributeValue::Rgb(a), AttributeValue::Rgb(b)) => {
            [a.x, a.y, a.z].map(bits) == [b.x, b.y, b.z].map(bits)
        }
        (AttributeValue::Argb(a), AttributeValue::Argb(b)) => {
            [a.x, a.y, a.z, a.w].map(bits) == [b.x, b.y, b.z, b.w].map(bits)
        }
        (AttributeValue::MoonPhase(a), AttributeValue::MoonPhase(b)) => a == b,
        (AttributeValue::FloatWithAlpha(a), AttributeValue::FloatWithAlpha(b)) => a == b,
        (AttributeValue::BlendToGray(a), AttributeValue::BlendToGray(b)) => a == b,
        _ => false,
    }
}

static NETWORK_CODEC: StaticCodec<EnvironmentAttributeMap> = LazyLock::new(|| NetworkCodec.boxed());

impl EnvironmentAttributeMap {
    pub const NETWORK_CODEC: &'static StaticCodec<EnvironmentAttributeMap> = &NETWORK_CODEC;

    pub fn empty() -> EnvironmentAttributeMap {
        EnvironmentAttributeMap::default()
    }

    pub fn builder() -> Builder {
        Builder {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, attribute: usize) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|(id, _)| *id == attribute)
            .map(|(_, entry)| entry)
    }

    pub fn apply_modifier(&self, attribute: usize, base_value: AttributeValue) -> AttributeValue {
        match self.get(attribute) {
            Some(entry) => entry.apply_modifier(base_value),
            None => base_value,
        }
    }

    pub fn contains(&self, attribute: usize) -> bool {
        self.get(attribute).is_some()
    }

    /// The attributes it has entries for, by place.
    pub fn key_set(&self) -> impl Iterator<Item = usize> + '_ {
        self.entries.iter().map(|(id, _)| *id)
    }

    /// `NETWORK_CODEC` (`CODEC` with only the syncable attributes kept), over a map's object.
    pub fn from_json(object: &JsonObject) -> Result<EnvironmentAttributeMap, String> {
        let mut entries = Vec::new();
        for (name, json) in object.entry_set() {
            let Some(id) = EnvironmentAttributes::by_name(&Identifier::parse(name))? else {
                continue;
            };
            let info = EnvironmentAttributes::info(id);
            if !info.is_syncable {
                continue;
            }
            entries.push((id, Entry::from_json(id, json)?));
        }
        Ok(EnvironmentAttributeMap { entries })
    }
}

impl Entry {
    /// `Entry.createCodec(attribute)`: the attribute's value as an override, or a modifier the
    /// attribute's type takes and its argument.
    fn from_json(attribute: usize, json: &JsonElement) -> Result<Entry, String> {
        let info = EnvironmentAttributes::info(attribute);
        if let Ok(value) = info.decode_value(json) {
            return Ok(Entry {
                argument: value,
                modifier: AttributeModifier::Override,
            });
        }
        let object = json
            .as_json_object()
            .ok_or_else(|| format!("{}: not a value or an entry: {}", info.name, json))?;
        let modifier = OperationId::decode_modifier(
            info.r#type,
            object.get("modifier").ok_or("No key modifier in MapLike")?,
        )?;
        let argument = object.get("argument").ok_or("No key argument in MapLike")?;
        let argument = match modifier {
            AttributeModifier::Override => info.decode_value(argument)?,
            _ => modifier.decode_argument(info.r#type, argument)?,
        };
        Ok(Entry { argument, modifier })
    }
}

/// `NETWORK_CODEC`'s codec: `from_json` of the input as JSON, the input as the rest (the
/// dispatched map's).
#[derive(Clone, Copy)]
struct NetworkCodec;

impl Valued for NetworkCodec {
    type Value = EnvironmentAttributeMap;
}

impl<T: OpsValue> Decoder<T> for NetworkCodec {
    fn decode(
        &self,
        ops: &dyn DynamicOps<T>,
        input: &T,
    ) -> DataResult<Pair<EnvironmentAttributeMap, T>> {
        let json = dynamic_ops::convert_to(ops, &JsonOps::INSTANCE, input);
        let Some(object) = json.as_json_object() else {
            return DataResult::error(|| format!("Not a map: {input}"));
        };
        match EnvironmentAttributeMap::from_json(object) {
            Ok(map) => DataResult::success(Pair::new(map, input.clone())),
            Err(message) => DataResult::error(|| message),
        }
    }
}

impl<T: OpsValue> Encoder<T> for NetworkCodec {
    fn encode(
        &self,
        input: &EnvironmentAttributeMap,
        ops: &dyn DynamicOps<T>,
        prefix: T,
    ) -> DataResult<T> {
        if input.entries.is_empty() {
            let mut builder = ops.map_builder();
            return builder.build(prefix);
        }
        DataResult::error(|| "EnvironmentAttributeMap's encoder isn't ported".to_owned())
    }
}

/// `EnvironmentAttributeMap.Builder`.
pub struct Builder {
    entries: Vec<(usize, Entry)>,
}

impl Builder {
    /// `modify`, a later entry of the same attribute replacing the earlier one. Panics where Java
    /// throws `IllegalArgumentException` for a modifier the attribute's type doesn't take.
    pub fn modify<V>(
        mut self,
        attribute: EnvironmentAttribute<V>,
        modifier: AttributeModifier,
        argument: AttributeValue,
    ) -> Builder {
        let info = attribute.info();
        assert!(
            modifier == AttributeModifier::Override
                || OperationId::VALUES
                    .iter()
                    .any(|&operation| info.r#type.modifier(operation) == Some(modifier)),
            "Modifier {modifier:?} is not valid for {:?}",
            info.r#type
        );
        self.entries.retain(|(id, _)| *id != attribute.id());
        self.entries
            .push((attribute.id(), Entry { argument, modifier }));
        self
    }

    pub fn set<V>(self, attribute: EnvironmentAttribute<V>, value: AttributeValue) -> Builder {
        self.modify(attribute, AttributeModifier::Override, value)
    }

    pub fn build(self) -> EnvironmentAttributeMap {
        EnvironmentAttributeMap {
            entries: self.entries,
        }
    }
}
