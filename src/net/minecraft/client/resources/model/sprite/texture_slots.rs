//! net/minecraft/client/resources/model/sprite/TextureSlots.java
//!
//! A model's `textures`: each slot a material, or a `#` reference to another slot, resolved down
//! a model's parents. Java keeps the slots in `HashMap`s and `Map.copyOf`s, whose order changes
//! from run to run, and resolves references until nothing changes, which ends the same whatever
//! the order; the port keeps them in insertion order.

use std::collections::HashMap;

use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_object::JsonObject;
use crate::net::minecraft::client::resources::model::model_debug_name::ModelDebugName;
use crate::net::minecraft::client::resources::model::sprite::material::Material;
use crate::net::minecraft::util::gson_helper::GsonHelper;

/// A model's slots, resolved to materials.
#[derive(Clone, Debug, Default)]
pub struct TextureSlots {
    resolved_values: HashMap<String, Material>,
}

/// `TextureSlots.Data`: one model file's slots as it wrote them.
#[derive(Clone, Debug, Default)]
pub struct Data {
    values: Vec<(String, SlotContents)>,
}

/// `TextureSlots.SlotContents`: a material, or the slot a `#` reference names.
#[derive(Clone, Debug, PartialEq)]
pub enum SlotContents {
    Value(Material),
    Reference(String),
}

/// `TextureSlots.Data.Builder`.
#[derive(Default)]
pub struct DataBuilder {
    texture_map: Vec<(String, SlotContents)>,
}

/// `TextureSlots.Resolver`: a model's data, then each parent's.
#[derive(Default)]
pub struct Resolver {
    entries: Vec<Data>,
}

// Java's `texture.charAt(0) == '#'`, which throws for an empty name.
fn is_texture_reference(texture: &str) -> bool {
    match texture.chars().next() {
        Some(first) => first == '#',
        None => panic!("StringIndexOutOfBoundsException: index 0, length 0"),
    }
}

impl TextureSlots {
    pub fn empty() -> TextureSlots {
        TextureSlots::default()
    }

    pub fn get_material(&self, reference: &str) -> Option<&Material> {
        let reference = if is_texture_reference(reference) {
            &reference[1..]
        } else {
            reference
        };
        self.resolved_values.get(reference)
    }

    /// `parseTextureMap`; the `Err` is the `JsonParseException` a material that doesn't decode
    /// throws.
    pub fn parse_texture_map(textures_object: &JsonObject) -> Result<Data, String> {
        let mut builder = DataBuilder::default();
        for (slot, value) in textures_object.entry_set() {
            TextureSlots::parse_entry(slot, value, &mut builder)?;
        }
        Ok(builder.build())
    }

    fn parse_entry(
        slot: &str,
        value: &JsonElement,
        output: &mut DataBuilder,
    ) -> Result<(), String> {
        if GsonHelper::is_string_value(value) && is_texture_reference(&value.get_as_string()) {
            output.add_reference(slot, &value.get_as_string()[1..]);
        } else {
            output.add_texture(slot, Material::parse(value)?);
        }
        Ok(())
    }
}

impl Data {
    pub fn empty() -> Data {
        Data::default()
    }

    pub fn values(&self) -> &[(String, SlotContents)] {
        &self.values
    }
}

impl DataBuilder {
    /// A slot set again keeps its place, like a `HashMap`'s `put`.
    fn put(&mut self, slot: &str, contents: SlotContents) {
        match self.texture_map.iter_mut().find(|(name, _)| name == slot) {
            Some(entry) => entry.1 = contents,
            None => self.texture_map.push((slot.to_owned(), contents)),
        }
    }

    pub fn add_reference(&mut self, slot: &str, reference: &str) -> &mut DataBuilder {
        self.put(slot, SlotContents::Reference(reference.to_owned()));
        self
    }

    pub fn add_texture(&mut self, slot: &str, material: Material) -> &mut DataBuilder {
        self.put(slot, SlotContents::Value(material));
        self
    }

    pub fn build(self) -> Data {
        Data {
            values: self.texture_map,
        }
    }
}

impl Resolver {
    pub fn add_last(&mut self, data: Data) -> &mut Resolver {
        self.entries.push(data);
        self
    }

    pub fn add_first(&mut self, data: Data) -> &mut Resolver {
        self.entries.insert(0, data);
        self
    }

    /// The slots from the last data to the first, each overriding the ones before, then every
    /// reference resolved to its target's material until no more resolve. Java warns about the
    /// ones left.
    pub fn resolve(&self, debug_name_provider: &dyn ModelDebugName) -> TextureSlots {
        if self.entries.is_empty() {
            return TextureSlots::empty();
        }

        let mut resolved: Vec<(String, Material)> = Vec::new();
        let mut unresolved: Vec<(String, String)> = Vec::new();
        for data in self.entries.iter().rev() {
            for (slot, contents) in &data.values {
                match contents {
                    SlotContents::Value(material) => {
                        unresolved.retain(|(name, _)| name != slot);
                        put(&mut resolved, slot, material.clone());
                    }
                    SlotContents::Reference(target) => {
                        resolved.retain(|(name, _)| name != slot);
                        put(&mut unresolved, slot, target.clone());
                    }
                }
            }
        }

        let mut has_changes = !unresolved.is_empty();
        while has_changes {
            has_changes = false;
            let mut index = 0;
            while index < unresolved.len() {
                let (slot, target) = &unresolved[index];
                let maybe_resolved = resolved
                    .iter()
                    .find(|(name, _)| name == target)
                    .map(|(_, material)| material.clone());
                match maybe_resolved {
                    Some(material) => {
                        let slot = slot.clone();
                        put(&mut resolved, &slot, material);
                        unresolved.remove(index);
                        has_changes = true;
                    }
                    None => index += 1,
                }
            }
        }

        if !unresolved.is_empty() {
            let lines: String = unresolved
                .iter()
                .map(|(slot, target)| format!("\t#{slot}-> #{target}\n"))
                .collect();
            eprintln!(
                "Unresolved texture references in {}:\n{lines}",
                debug_name_provider.debug_name()
            );
        }

        TextureSlots {
            resolved_values: resolved.into_iter().collect(),
        }
    }
}

fn put<V>(entries: &mut Vec<(String, V)>, slot: &str, value: V) {
    match entries.iter_mut().find(|(name, _)| name == slot) {
        Some(entry) => entry.1 = value,
        None => entries.push((slot.to_owned(), value)),
    }
}
