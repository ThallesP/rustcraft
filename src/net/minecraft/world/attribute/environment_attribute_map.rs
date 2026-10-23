//! net/minecraft/world/attribute/EnvironmentAttributeMap.java
//!
//! What a dimension type or a biome does to attributes: an entry each, a modifier and its
//! argument. `CODEC` and `NETWORK_CODEC` are `from_json`, decoded by hand from Gson's tree like
//! their dispatched map: each attribute's entry is its value (an override) or `{"modifier",
//! "argument"}`. The client reads its known pack's dimension types and biomes through
//! `NETWORK_CODEC`, which keeps the syncable attributes only. Attributes whose types aren't
//! ported are left out (`EnvironmentAttributes::UNPORTED`). `equals`, `hashCode` and `toString`
//! aren't ported: Java's maps of these (`SpatialAttributeInterpolator`) go by identity.

use crate::com::google::gson::json_element::{JsonElement, JsonObject};
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

impl EnvironmentAttributeMap {
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
        let object = json.as_json_object().ok_or_else(|| {
            format!(
                "{}: not a value or an entry: {}",
                info.name,
                json.to_json_string()
            )
        })?;
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
