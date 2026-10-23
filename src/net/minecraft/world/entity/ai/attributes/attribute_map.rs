//! net/minecraft/world/entity/ai/attributes/AttributeMap.java, with what it reads of
//! `AttributeSupplier`.
//!
//! An entity's attributes: an instance for each attribute its type supplies, made with the
//! supplier's base values (Java makes each on first use, which gives the same values). Java's
//! `onAttributeModified` collects the instances that changed for `LivingEntity.refreshDirtyAttributes`;
//! here a mutable lookup marks the attribute as possibly changed. Not ported yet: the synced
//! attributes, `assignAllValues`, `assignBaseValues` and the modifiers' maps by attribute.

use crate::net::minecraft::world::entity::ai::attributes::attribute::Attribute;
use crate::net::minecraft::world::entity::ai::attributes::attribute_instance::AttributeInstance;

/// The attributes of one entity.
#[derive(Clone, Debug)]
pub struct AttributeMap {
    attributes: Vec<AttributeInstance>,
    attributes_to_update: Vec<&'static Attribute>,
}

impl AttributeMap {
    /// The supplier's attributes with their base values (`AttributeSupplier.Builder.add`).
    pub fn new(supplier: &[(&'static Attribute, f64)]) -> AttributeMap {
        let attributes = supplier
            .iter()
            .map(|&(attribute, base_value)| {
                let mut instance = AttributeInstance::new(attribute);
                instance.set_base_value(base_value);
                instance
            })
            .collect();
        AttributeMap {
            attributes,
            attributes_to_update: Vec::new(),
        }
    }

    pub fn get_instance(&self, attribute: &'static Attribute) -> Option<&AttributeInstance> {
        self.attributes
            .iter()
            .find(|instance| Attribute::same(instance.get_attribute(), attribute))
    }

    /// `getInstance`, to change it: the attribute goes on the list `getAttributesToUpdate` hands
    /// out.
    pub fn get_instance_mut(
        &mut self,
        attribute: &'static Attribute,
    ) -> Option<&mut AttributeInstance> {
        let instance = self
            .attributes
            .iter_mut()
            .find(|instance| Attribute::same(instance.get_attribute(), attribute))?;
        if !self
            .attributes_to_update
            .iter()
            .any(|updated| Attribute::same(updated, attribute))
        {
            self.attributes_to_update.push(attribute);
        }
        Some(instance)
    }

    pub fn has_attribute(&self, attribute: &'static Attribute) -> bool {
        self.get_instance(attribute).is_some()
    }

    /// The attribute's value. Throws `IllegalArgumentException` for one the entity doesn't
    /// have.
    pub fn get_value(&self, attribute: &'static Attribute) -> f64 {
        self.attributes
            .iter()
            .find(|instance| Attribute::same(instance.get_attribute(), attribute))
            .unwrap_or_else(|| panic!("Can't find attribute {}", attribute.get_description_id()))
            .get_value()
    }

    pub fn get_base_value(&self, attribute: &'static Attribute) -> f64 {
        self.get_instance(attribute)
            .unwrap_or_else(|| panic!("Can't find attribute {}", attribute.get_description_id()))
            .get_base_value()
    }

    /// The attributes changed since the list was last cleared.
    pub fn take_attributes_to_update(&mut self) -> Vec<&'static Attribute> {
        std::mem::take(&mut self.attributes_to_update)
    }
}
