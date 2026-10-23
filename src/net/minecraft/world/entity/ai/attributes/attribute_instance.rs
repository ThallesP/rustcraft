//! net/minecraft/world/entity/ai/attributes/AttributeInstance.java
//!
//! The modifiers of each operation are a `Vec` in the order they were added, where Java keeps
//! them in an `Object2ObjectOpenHashMap` by id and adds or multiplies them in its table's order.
//! The two only differ with two or more modifiers of one operation, which can round differently
//! when applied in another order; that order (fastutil's open hashing of the ids) isn't ported
//! yet. `onDirty`, which tells the `AttributeMap`, is the map's to call (`AttributeMap::get_instance_mut`
//! marks what it hands out). Not ported yet: the permanent modifiers' `pack`/`apply` and
//! `replaceFrom`, which come with saving and respawning.

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::entity::ai::attributes::attribute::Attribute;
use crate::net::minecraft::world::entity::ai::attributes::attribute_modifier::{
    AttributeModifier, Operation,
};

/// An attribute's base value and modifiers on one entity, and the value they make.
#[derive(Clone, Debug)]
pub struct AttributeInstance {
    attribute: &'static Attribute,
    modifiers_by_operation: [Vec<AttributeModifier>; 3],
    permanent_modifiers: Vec<Identifier>,
    base_value: f64,
}

impl AttributeInstance {
    pub fn new(attribute: &'static Attribute) -> AttributeInstance {
        AttributeInstance {
            attribute,
            modifiers_by_operation: Default::default(),
            permanent_modifiers: Vec::new(),
            base_value: attribute.get_default_value(),
        }
    }

    pub fn get_attribute(&self) -> &'static Attribute {
        self.attribute
    }

    pub fn get_base_value(&self) -> f64 {
        self.base_value
    }

    /// Returns whether it changed, which Java tells the map through `onDirty`.
    pub fn set_base_value(&mut self, base_value: f64) -> bool {
        if base_value != self.base_value {
            self.base_value = base_value;
            self.set_dirty();
            true
        } else {
            false
        }
    }

    /// Every modifier, in the order of their operations.
    pub fn get_modifiers(&self) -> Vec<AttributeModifier> {
        self.modifiers_by_operation
            .iter()
            .flatten()
            .cloned()
            .collect()
    }

    pub fn get_modifier(&self, id: &Identifier) -> Option<&AttributeModifier> {
        self.modifiers_by_operation
            .iter()
            .flatten()
            .find(|modifier| modifier.is(id))
    }

    pub fn has_modifier(&self, id: &Identifier) -> bool {
        self.get_modifier(id).is_some()
    }

    /// Throws `IllegalArgumentException` when a modifier with the id is on already.
    fn add_modifier(&mut self, modifier: AttributeModifier) {
        assert!(
            !self.has_modifier(modifier.id()),
            "Modifier is already applied on this attribute!"
        );
        self.modifiers_by_operation[modifier.operation() as usize].push(modifier);
        self.set_dirty();
    }

    pub fn add_transient_modifier(&mut self, modifier: AttributeModifier) {
        self.add_modifier(modifier);
    }

    pub fn add_permanent_modifier(&mut self, modifier: AttributeModifier) {
        let id = modifier.id().clone();
        self.add_modifier(modifier);
        self.permanent_modifiers.push(id);
    }

    /// Java marks the cached value stale; here the value is worked out each time it's asked for,
    /// which gives the same value.
    fn set_dirty(&mut self) {}

    /// `removeModifier(Identifier)`: whether there was one.
    pub fn remove_modifier(&mut self, id: &Identifier) -> bool {
        for modifiers in &mut self.modifiers_by_operation {
            if let Some(index) = modifiers.iter().position(|modifier| modifier.is(id)) {
                modifiers.remove(index);
                self.permanent_modifiers.retain(|permanent| permanent != id);
                self.set_dirty();
                return true;
            }
        }
        false
    }

    pub fn remove_modifiers(&mut self) {
        for modifier in self.get_modifiers() {
            self.remove_modifier(modifier.id());
        }
    }

    pub fn get_value(&self) -> f64 {
        self.calculate_value()
    }

    /// The base plus each added value, plus the base sum times each base multiple, times one plus
    /// each total multiple, kept in the attribute's range.
    fn calculate_value(&self) -> f64 {
        let mut base = self.get_base_value();
        for modifier in &self.modifiers_by_operation[Operation::ADD_VALUE as usize] {
            base += modifier.amount();
        }

        let mut result = base;
        for modifier in &self.modifiers_by_operation[Operation::ADD_MULTIPLIED_BASE as usize] {
            result += base * modifier.amount();
        }

        for modifier in &self.modifiers_by_operation[Operation::ADD_MULTIPLIED_TOTAL as usize] {
            result *= 1.0 + modifier.amount();
        }

        self.attribute.sanitize_value(result)
    }
}

#[cfg(test)]
mod tests {
    use std::panic::AssertUnwindSafe;

    use serde_json::Value;

    use super::AttributeInstance;
    use crate::golden::{Decode, Encode, Golden, Thrown, catch};
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::world::entity::ai::attributes::attribute::Attribute;
    use crate::net::minecraft::world::entity::ai::attributes::attribute_modifier::{
        AttributeModifier, Operation,
    };
    use crate::net::minecraft::world::entity::ai::attributes::attributes::Attributes;

    fn attribute(name: &str) -> &'static Attribute {
        match name {
            "movement_speed" => Attributes::MOVEMENT_SPEED,
            "sneaking_speed" => Attributes::SNEAKING_SPEED,
            "jump_strength" => Attributes::JUMP_STRENGTH,
            "gravity" => Attributes::GRAVITY,
            "step_height" => Attributes::STEP_HEIGHT,
            "scale" => Attributes::SCALE,
            "friction_modifier" => Attributes::FRICTION_MODIFIER,
            "air_drag_modifier" => Attributes::AIR_DRAG_MODIFIER,
            "bounciness" => Attributes::BOUNCINESS,
            "movement_efficiency" => Attributes::MOVEMENT_EFFICIENCY,
            "water_movement_efficiency" => Attributes::WATER_MOVEMENT_EFFICIENCY,
            "max_health" => Attributes::MAX_HEALTH,
            _ => panic!("no attribute {name}"),
        }
    }

    fn operation(name: &str) -> Operation {
        match name {
            "ADD_VALUE" => Operation::ADD_VALUE,
            "ADD_MULTIPLIED_BASE" => Operation::ADD_MULTIPLIED_BASE,
            "ADD_MULTIPLIED_TOTAL" => Operation::ADD_MULTIPLIED_TOTAL,
            _ => panic!("no operation {name}"),
        }
    }

    /// Mirrors `PlayerSuites.AttributeInstanceSuite`: each step's output and the value after.
    fn run(name: String, ops: Vec<Value>) -> Vec<Box<dyn Encode>> {
        let mut instance = AttributeInstance::new(attribute(&name));
        let mut steps: Vec<Box<dyn Encode>> = Vec::new();
        for op in ops {
            let op = op.as_array().expect("an op is an array").clone();
            let step = catch(AssertUnwindSafe(|| {
                let id = || Identifier::parse(op[1].as_str().expect("an id"));
                let output: Box<dyn Encode> = match op[0].as_str().expect("an op's name") {
                    "base" => {
                        instance.set_base_value(f64::decode(&op[1]));
                        Box::new(())
                    }
                    "add" => {
                        instance.add_transient_modifier(AttributeModifier::new(
                            id(),
                            f64::decode(&op[2]),
                            operation(op[3].as_str().expect("an operation")),
                        ));
                        Box::new(())
                    }
                    "remove" => Box::new(instance.remove_modifier(&id())),
                    "removeAll" => {
                        instance.remove_modifiers();
                        Box::new(())
                    }
                    "value" => Box::new(instance.get_value()),
                    name => panic!("unknown op {name}"),
                };
                Box::new((output, instance.get_value())) as Box<dyn Encode>
            }));
            match step {
                Ok(step) => steps.push(step),
                Err(_) => {
                    steps.push(Box::new(Thrown));
                    break;
                }
            }
        }
        steps
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.ai.attributes.AttributeInstance");
        golden.check("programs", run);
        golden.finish();
    }
}
