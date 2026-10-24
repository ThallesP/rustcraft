//! net/minecraft/client/renderer/PostChainConfig.java
//!
//! A post effect's JSON (`post_effect/<id>.json`): its internal targets and its passes, each with
//! its shaders, the targets or textures it samples, the target it draws into and its uniform
//! blocks. The codec is decoded by hand from serde_json's values, field by field as it reads them,
//! with their defaults. Java's maps are `HashMap`s, whose order nothing that draws reads; here
//! they're kept sorted.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::net::minecraft::client::renderer::uniform_value::UniformValue;
use crate::net::minecraft::resources::identifier::Identifier;

#[derive(Clone, Debug, PartialEq)]
pub struct PostChainConfig {
    pub internal_targets: BTreeMap<Identifier, InternalTarget>,
    pub passes: Vec<Pass>,
}

/// `PostChainConfig.InternalTarget`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InternalTarget {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub persistent: bool,
    pub clear_color: i32,
}

/// `PostChainConfig.Input`: a target's color or depth, or a texture.
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    Target {
        sampler_name: String,
        target_id: Identifier,
        use_depth_buffer: bool,
        bilinear: bool,
    },
    Texture {
        sampler_name: String,
        location: Identifier,
        width: i32,
        height: i32,
        bilinear: bool,
    },
}

impl Input {
    pub fn sampler_name(&self) -> &str {
        match self {
            Input::Target { sampler_name, .. } | Input::Texture { sampler_name, .. } => {
                sampler_name
            }
        }
    }

    pub fn referenced_targets(&self) -> Vec<Identifier> {
        match self {
            Input::Target { target_id, .. } => vec![target_id.clone()],
            Input::Texture { .. } => Vec::new(),
        }
    }
}

/// `PostChainConfig.Pass`.
#[derive(Clone, Debug, PartialEq)]
pub struct Pass {
    pub vertex_shader_id: Identifier,
    pub fragment_shader_id: Identifier,
    pub inputs: Vec<Input>,
    pub output_target: Identifier,
    pub uniforms: BTreeMap<String, Vec<UniformValue>>,
}

impl Pass {
    pub fn referenced_targets(&self) -> Vec<Identifier> {
        let mut targets: Vec<Identifier> = self
            .inputs
            .iter()
            .flat_map(Input::referenced_targets)
            .collect();
        targets.push(self.output_target.clone());
        targets
    }
}

fn identifier(value: &Value, field: &str) -> Result<Identifier, String> {
    let text = value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("No key {field}"))?;
    Identifier::try_parse(text).ok_or_else(|| format!("Not a valid resource location: {text}"))
}

fn boolean(value: &Value, field: &str) -> Result<bool, String> {
    match value.get(field) {
        None => Ok(false),
        Some(field_value) => field_value
            .as_bool()
            .ok_or_else(|| format!("{field} isn't a boolean")),
    }
}

fn positive_int(value: &Value, field: &str) -> Result<Option<i32>, String> {
    match value.get(field) {
        None => Ok(None),
        Some(field_value) => {
            let number = field_value
                .as_i64()
                .ok_or_else(|| format!("{field} isn't an int"))? as i32;
            if number > 0 {
                Ok(Some(number))
            } else {
                Err(format!("Value must be positive: {number}"))
            }
        }
    }
}

impl PostChainConfig {
    /// `CODEC`'s decoding.
    pub fn decode(value: &Value) -> Result<PostChainConfig, String> {
        let mut internal_targets = BTreeMap::new();
        if let Some(targets) = value.get("targets") {
            for (id, target) in targets.as_object().ok_or("targets isn't a map")? {
                let id = Identifier::try_parse(id)
                    .ok_or_else(|| format!("Not a valid resource location: {id}"))?;
                let clear_color = match target.get("clear_color") {
                    None => 0,
                    Some(color) => color.as_i64().ok_or("clear_color isn't a color")? as i32,
                };
                internal_targets.insert(
                    id,
                    InternalTarget {
                        width: positive_int(target, "width")?,
                        height: positive_int(target, "height")?,
                        persistent: boolean(target, "persistent")?,
                        clear_color,
                    },
                );
            }
        }
        let mut passes = Vec::new();
        for pass in value
            .get("passes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            passes.push(PostChainConfig::decode_pass(pass)?);
        }
        Ok(PostChainConfig {
            internal_targets,
            passes,
        })
    }

    fn decode_pass(pass: &Value) -> Result<Pass, String> {
        let mut inputs = Vec::new();
        for input in pass
            .get("inputs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let sampler_name = input
                .get("sampler_name")
                .and_then(Value::as_str)
                .ok_or("No key sampler_name")?
                .to_string();
            // Codec.xor: a texture input, or a target input.
            let decoded = if input.get("location").is_some() {
                Input::Texture {
                    sampler_name,
                    location: identifier(input, "location")?,
                    width: positive_int(input, "width")?.ok_or("No key width")?,
                    height: positive_int(input, "height")?.ok_or("No key height")?,
                    bilinear: boolean(input, "bilinear")?,
                }
            } else {
                Input::Target {
                    sampler_name,
                    target_id: identifier(input, "target")?,
                    use_depth_buffer: boolean(input, "use_depth_buffer")?,
                    bilinear: boolean(input, "bilinear")?,
                }
            };
            if inputs
                .iter()
                .any(|other: &Input| other.sampler_name() == decoded.sampler_name())
            {
                return Err(format!(
                    "Encountered repeated sampler name: {}",
                    decoded.sampler_name()
                ));
            }
            inputs.push(decoded);
        }
        let mut uniforms = BTreeMap::new();
        if let Some(blocks) = pass.get("uniforms") {
            for (name, values) in blocks.as_object().ok_or("uniforms isn't a map")? {
                let values = values
                    .as_array()
                    .ok_or("a uniform block isn't a list")?
                    .iter()
                    .map(UniformValue::decode)
                    .collect::<Result<Vec<_>, _>>()?;
                uniforms.insert(name.clone(), values);
            }
        }
        Ok(Pass {
            vertex_shader_id: identifier(pass, "vertex_shader")?,
            fragment_shader_id: identifier(pass, "fragment_shader")?,
            inputs,
            output_target: identifier(pass, "output")?,
            uniforms,
        })
    }
}
