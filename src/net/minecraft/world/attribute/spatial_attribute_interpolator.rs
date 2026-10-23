//! net/minecraft/world/attribute/SpatialAttributeInterpolator.java
//!
//! The weights of the biomes around a position, which blend their attributes. Java keys its
//! fastutil `Reference2DoubleArrayMap` by each biome's attribute map, by reference, in the order
//! they first come; each biome has its own map, so here the key is the biome.

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::world::attribute::attribute_type::AttributeValue;
use crate::net::minecraft::world::attribute::environment_attribute_layer::attributes_of;
use crate::net::minecraft::world::attribute::environment_attribute_map::EnvironmentAttributeMap;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;

#[derive(Clone, Debug, Default)]
pub struct SpatialAttributeInterpolator {
    weights_by_source: Vec<(Holder, f64)>,
}

impl SpatialAttributeInterpolator {
    pub fn new() -> SpatialAttributeInterpolator {
        SpatialAttributeInterpolator::default()
    }

    pub fn clear(&mut self) {
        self.weights_by_source.clear();
    }

    /// The biome's weight added to what it has (`mergeDouble(…, Double::sum)`).
    pub fn accumulate(&mut self, weight: f64, biome: &Holder) -> &mut Self {
        match self
            .weights_by_source
            .iter_mut()
            .find(|(source, _)| source == biome)
        {
            Some((_, accumulated)) => *accumulated += weight,
            None => self.weights_by_source.push((biome.clone(), weight)),
        }
        self
    }

    /// Each biome's value of the attribute, blended in by its share of the weight so far.
    /// `biomes` are the registry's biomes, each with its attribute map.
    pub fn apply_attribute_layer(
        &self,
        attribute: usize,
        base_value: AttributeValue,
        biomes: &[(Holder, EnvironmentAttributeMap)],
    ) -> AttributeValue {
        if self.weights_by_source.is_empty() {
            return base_value;
        }

        if self.weights_by_source.len() == 1 {
            let (source, _) = &self.weights_by_source[0];
            return attributes_of(biomes, source).apply_modifier(attribute, base_value);
        }

        let lerp = EnvironmentAttributes::info(attribute).r#type.spatial_lerp();
        let mut result_value: Option<AttributeValue> = None;
        let mut accumulated_weight = 0.0f64;

        for (source, source_weight) in &self.weights_by_source {
            let source_value = attributes_of(biomes, source).apply_modifier(attribute, base_value);
            accumulated_weight += source_weight;
            result_value = Some(match result_value {
                None => source_value,
                Some(result_value) => {
                    let relative_fraction = (source_weight / accumulated_weight) as f32;
                    lerp.apply(relative_fraction, result_value, source_value)
                }
            });
        }

        result_value.expect("weights aren't empty")
    }
}
