//! net/minecraft/world/attribute/EnvironmentAttributeProbe.java
//!
//! The attributes at a moving point, the camera's: each tick it samples the biomes around its
//! position (`GaussianSampler`, a quarter of the position in noise-biome quarts), and each
//! attribute read blends between the value of the last tick and this one's by the partial tick.
//! An attribute not read during a tick is dropped at the next. Java's probe keeps the level it
//! was ticked in, which reads go to; here the level and its system are parameters of both, which
//! a caller passes the same.

use crate::net::minecraft::world::attribute::attribute_type::{AttributeValue, Value};
use crate::net::minecraft::world::attribute::environment_attribute::EnvironmentAttribute;
use crate::net::minecraft::world::attribute::environment_attribute_layer::LayerLevel;
use crate::net::minecraft::world::attribute::environment_attribute_system::EnvironmentAttributeSystem;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::attribute::gaussian_sampler::GaussianSampler;
use crate::net::minecraft::world::attribute::spatial_attribute_interpolator::SpatialAttributeInterpolator;
use crate::net::minecraft::world::phys::vec3::Vec3;

#[derive(Default)]
pub struct EnvironmentAttributeProbe {
    // Each attribute read since the last tick but one, in the order first read (Java's
    // `Reference2ObjectOpenHashMap`, whose order nothing reads).
    value_probes: Vec<(usize, ValueProbe)>,
    // Whether it was ticked in a level since `reset`, and where.
    position: Option<Vec3>,
    biome_interpolator: SpatialAttributeInterpolator,
}

struct ValueProbe {
    last_value: AttributeValue,
    new_value: Option<AttributeValue>,
}

impl EnvironmentAttributeProbe {
    pub fn new() -> EnvironmentAttributeProbe {
        EnvironmentAttributeProbe::default()
    }

    pub fn reset(&mut self) {
        self.position = None;
        self.biome_interpolator.clear();
        self.value_probes.clear();
    }

    pub fn tick(&mut self, level: &dyn LayerLevel, position: Vec3) {
        self.position = Some(position);
        self.value_probes.retain_mut(|(_, probe)| !probe.tick());
        self.biome_interpolator.clear();
        // position.scale(0.25).
        let quarts = Vec3::new(position.x * 0.25, position.y * 0.25, position.z * 0.25);
        let interpolator = &mut self.biome_interpolator;
        GaussianSampler::sample(
            &quarts,
            |x, y, z| level.get_noise_biome_at_quart(x, y, z),
            |weight, biome| {
                interpolator.accumulate(weight, &biome);
            },
        );
    }

    pub fn get_value<V: Value>(
        &mut self,
        system: &EnvironmentAttributeSystem,
        level: &dyn LayerLevel,
        attribute: &EnvironmentAttribute<V>,
        partial_ticks: f32,
    ) -> V {
        V::from_attribute_value(self.get_value_of(system, level, attribute.id(), partial_ticks))
    }

    pub fn get_value_of(
        &mut self,
        system: &EnvironmentAttributeSystem,
        level: &dyn LayerLevel,
        attribute: usize,
        partial_ticks: f32,
    ) -> AttributeValue {
        let index = match self
            .value_probes
            .iter()
            .position(|(id, _)| *id == attribute)
        {
            Some(index) => index,
            None => {
                let value = self.get_value_from_level(system, level, attribute);
                self.value_probes.push((
                    attribute,
                    ValueProbe {
                        last_value: value,
                        new_value: Some(value),
                    },
                ));
                self.value_probes.len() - 1
            }
        };
        if self.value_probes[index].1.new_value.is_none() {
            let value = self.get_value_from_level(system, level, attribute);
            self.value_probes[index].1.new_value = Some(value);
        }

        let probe = &self.value_probes[index].1;
        EnvironmentAttributes::info(attribute)
            .r#type
            .partial_tick_lerp()
            .apply(
                partial_ticks,
                probe.last_value,
                probe.new_value.expect("set above"),
            )
    }

    fn get_value_from_level(
        &self,
        system: &EnvironmentAttributeSystem,
        level: &dyn LayerLevel,
        attribute: usize,
    ) -> AttributeValue {
        match &self.position {
            Some(position) => {
                system.get_value_of(attribute, position, Some(&self.biome_interpolator), level)
            }
            None => EnvironmentAttributes::info(attribute).default_value,
        }
    }
}

impl ValueProbe {
    /// Whether to drop it: it wasn't read since the last tick.
    fn tick(&mut self) -> bool {
        match self.new_value.take() {
            None => true,
            Some(new_value) => {
                self.last_value = new_value;
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::EnvironmentAttributeProbe;
    use crate::golden::{Decode, Golden};
    use crate::net::minecraft::world::attribute::environment_attribute_system::tests::{
        TestLevel, attribute_id, encode, system,
    };
    use crate::net::minecraft::world::phys::vec3::Vec3;

    /// The camera's probe, ticked along a path while time and weather change, each read blended
    /// by its partial tick (`AttributeSuites`' `ProbeSuite`).
    #[test]
    fn readings_match_vanilla() {
        let golden = Golden::load("net.minecraft.world.attribute.EnvironmentAttributeProbe");
        golden.check("program", |source: String, ops: Vec<Value>| {
            let level = TestLevel::new(&source);
            let mut probe = EnvironmentAttributeProbe::new();
            let system = &system("overworld");
            let mut results = Vec::new();
            for op in &ops {
                let op = op.as_array().expect("an op is an array");
                match op[0].as_str() {
                    Some("tick") => {
                        level.ticks.set(i64::decode(&op[1]));
                        level.rain.set(f32::decode(&op[2]));
                        level.thunder.set(f32::decode(&op[3]));
                        system.invalidate_tick_cache();
                        probe.tick(
                            &level,
                            Vec3::new(
                                f64::decode(&op[4]),
                                f64::decode(&op[5]),
                                f64::decode(&op[6]),
                            ),
                        );
                    }
                    Some("get") => {
                        let attribute = attribute_id(op[1].as_str().expect("a name"));
                        results.push(encode(probe.get_value_of(
                            system,
                            &level,
                            attribute,
                            f32::decode(&op[2]),
                        )));
                    }
                    other => panic!("unknown op {other:?}"),
                }
            }
            Value::Array(results)
        });
        golden.finish();
    }
}
