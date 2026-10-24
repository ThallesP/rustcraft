//! net/minecraft/client/renderer/UniformValue.java
//!
//! A uniform's value in a post effect's JSON, by its `type`: written into its block with std140's
//! layout. The codec is decoded by hand from serde_json's values, as the codecs read them. A
//! `matrix4x4` (`ExtraCodecs.MATRIX4F`), which no vanilla post effect has, isn't ported.

use serde_json::Value;

use crate::com::mojang::blaze3d::buffers::std140_builder::Std140Builder;
use crate::com::mojang::blaze3d::buffers::std140_size_calculator::Std140SizeCalculator;

#[derive(Clone, Debug, PartialEq)]
pub enum UniformValue {
    Int(i32),
    IVec3([i32; 3]),
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
}

impl UniformValue {
    pub fn write_to(&self, builder: &mut Std140Builder) {
        match self {
            UniformValue::Int(value) => builder.put_int(*value),
            UniformValue::IVec3([x, y, z]) => builder.put_ivec3(*x, *y, *z),
            UniformValue::Float(value) => builder.put_float(*value),
            UniformValue::Vec2([x, y]) => builder.put_vec2(*x, *y),
            UniformValue::Vec3([x, y, z]) => builder.put_vec3(*x, *y, *z),
            UniformValue::Vec4([x, y, z, w]) => builder.put_vec4(*x, *y, *z, *w),
        };
    }

    pub fn add_size(&self, calculator: Std140SizeCalculator) -> Std140SizeCalculator {
        match self {
            UniformValue::Int(_) => calculator.put_int(),
            UniformValue::IVec3(_) => calculator.put_ivec3(),
            UniformValue::Float(_) => calculator.put_float(),
            UniformValue::Vec2(_) => calculator.put_vec2(),
            UniformValue::Vec3(_) => calculator.put_vec3(),
            UniformValue::Vec4(_) => calculator.put_vec4(),
        }
    }

    /// `CODEC`: `{"type": …, "value": …}`.
    pub fn decode(value: &Value) -> Result<UniformValue, String> {
        let uniform_type = value
            .get("type")
            .and_then(Value::as_str)
            .ok_or("a uniform without a type")?;
        let field = value.get("value").ok_or("a uniform without a value")?;
        let floats = |count: usize| -> Result<Vec<f32>, String> {
            let list = field.as_array().ok_or("not a list")?;
            if list.len() != count {
                return Err(format!("Input is not a list of {count} elements"));
            }
            list.iter()
                .map(|value| {
                    value
                        .as_f64()
                        .map(|value| value as f32)
                        .ok_or("not a number".to_string())
                })
                .collect()
        };
        Ok(match uniform_type {
            "int" => UniformValue::Int(field.as_i64().ok_or("not an int")? as i32),
            "ivec3" => {
                let list = floats(3)?;
                UniformValue::IVec3([list[0] as i32, list[1] as i32, list[2] as i32])
            }
            "float" => UniformValue::Float(field.as_f64().ok_or("not a float")? as f32),
            "vec2" => {
                let list = floats(2)?;
                UniformValue::Vec2([list[0], list[1]])
            }
            "vec3" => {
                let list = floats(3)?;
                UniformValue::Vec3([list[0], list[1], list[2]])
            }
            "vec4" => {
                let list = floats(4)?;
                UniformValue::Vec4([list[0], list[1], list[2], list[3]])
            }
            "matrix4x4" => return Err("a matrix4x4 uniform isn't ported".to_string()),
            other => return Err(format!("Unknown uniform type {other}")),
        })
    }
}
