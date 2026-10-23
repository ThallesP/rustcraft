//! net/minecraft/world/level/chunk/DataLayer.java
//!
//! A section's light, a nibble for each of its 4096 blocks, which the client gets from the
//! server's light data. Its array is made on first write (`getData`), so a layer that's only ever
//! filled holds none. `getData` hands out the array the caller writes into, which here is
//! `get_data` making it and lending it out.

use std::fmt::Write;

/// 16³ values of 4 bits, two to a byte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataLayer {
    data: Option<Box<[i8; 2048]>>,
    default_value: i32,
}

impl Default for DataLayer {
    fn default() -> DataLayer {
        DataLayer::new()
    }
}

impl DataLayer {
    pub const LAYER_COUNT: i32 = 16;
    pub const LAYER_SIZE: i32 = 128;
    pub const SIZE: i32 = 2048;

    /// `DataLayer()`: every value 0.
    pub fn new() -> DataLayer {
        DataLayer::with_default(0)
    }

    /// `DataLayer(int)`: every value the default, without an array.
    pub fn with_default(default_value: i32) -> DataLayer {
        DataLayer {
            data: None,
            default_value,
        }
    }

    /// `DataLayer(byte[])`: panics unless there are 2048 bytes, where Java throws
    /// `IllegalArgumentException`.
    pub fn from_data(data: &[i8]) -> DataLayer {
        let data: [i8; 2048] = data
            .try_into()
            .unwrap_or_else(|_| panic!("DataLayer should be 2048 bytes not: {}", data.len()));
        DataLayer {
            data: Some(Box::new(data)),
            default_value: 0,
        }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> i32 {
        self.get_by_index(get_index(x, y, z))
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, val: i32) {
        self.set_by_index(get_index(x, y, z), val);
    }

    /// Panics past the array, where Java throws `ArrayIndexOutOfBoundsException`.
    #[inline]
    fn get_by_index(&self, index: i32) -> i32 {
        match &self.data {
            None => self.default_value,
            Some(data) => {
                let position = get_byte_index(index);
                let nibble = get_nibble_index(index);
                (data[position as usize] as i32) >> (4 * nibble) & 15
            }
        }
    }

    fn set_by_index(&mut self, index: i32, val: i32) {
        let data = self.get_data();
        let position = get_byte_index(index);
        let nibble = get_nibble_index(index);
        let mask = !(15 << (4 * nibble));
        let value_to_set = (val & 15) << (4 * nibble);
        data[position as usize] = (data[position as usize] as i32 & mask | value_to_set) as i8;
    }

    /// Drops the array: every value is the one given.
    pub fn fill(&mut self, value: i32) {
        self.default_value = value;
        self.data = None;
    }

    /// The bytes, made from the default value the first time.
    pub fn get_data(&mut self) -> &mut [i8; 2048] {
        let default_value = self.default_value;
        self.data.get_or_insert_with(|| {
            let packed = if default_value != 0 {
                pack_filled(default_value)
            } else {
                0
            };
            Box::new([packed; 2048])
        })
    }

    pub fn copy(&self) -> DataLayer {
        self.clone()
    }

    /// `layerToString(layer)`: the first 256 values (Java's ignores the layer), 16 to a line, in
    /// hex.
    pub fn layer_to_string(&self, _layer: i32) -> String {
        let mut builder = String::new();
        for i in 0..256 {
            write!(builder, "{:x}", self.get_by_index(i)).expect("a String takes any text");
            if i & 15 == 15 {
                builder.push('\n');
            }
        }
        builder
    }

    pub fn is_definitely_homogenous(&self) -> bool {
        self.data.is_none()
    }

    pub fn is_definitely_filled_with(&self, value: i32) -> bool {
        self.data.is_none() && self.default_value == value
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_none() && self.default_value == 0
    }
}

/// `toString()`: every value in hex, 16 to a line, and a blank line after each of the 16 layers.
impl std::fmt::Display for DataLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..4096 {
            write!(f, "{:x}", self.get_by_index(i))?;
            if i & 15 == 15 {
                f.write_char('\n')?;
            }
            if i & 0xff == 255 {
                f.write_char('\n')?;
            }
        }
        Ok(())
    }
}

fn get_index(x: i32, y: i32, z: i32) -> i32 {
    y << 8 | z << 4 | x
}

fn get_nibble_index(index: i32) -> i32 {
    index & 1
}

fn get_byte_index(position: i32) -> i32 {
    position >> 1
}

/// The value in both nibbles of a byte (Java's `(byte)value | value << 4`, cut to a byte).
fn pack_filled(value: i32) -> i8 {
    let mut packed = value as i8;
    let mut i = 4;
    while i < 8 {
        packed = (packed as i32 | value << i) as i8;
        i += 4;
    }
    packed
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::DataLayer;
    use crate::golden::{Golden, catch, hex, unhex};

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.chunk.DataLayer");
        golden.field("LAYER_COUNT", DataLayer::LAYER_COUNT);
        golden.field("LAYER_SIZE", DataLayer::LAYER_SIZE);
        golden.field("SIZE", DataLayer::SIZE);
        golden.check("programs", |init: Value, ops: Value| run(&init, &ops));
        golden.finish();
    }

    // Mirrors DataLayerSuite.run in oracle/golden/: `[output, isEmpty, isDefinitelyHomogenous]`
    // after each op, a panic ending them.
    fn run(init: &Value, ops: &Value) -> Vec<Value> {
        let mut steps = Vec::new();
        let made = catch(|| match init[0].as_str().expect("an init's kind") {
            "new" => DataLayer::new(),
            "default" => DataLayer::with_default(init[1].as_i64().expect("an int") as i32),
            _ => DataLayer::from_data(&unhex(init[1].as_str().expect("hex"))),
        });
        let Ok(mut layer) = made else {
            steps.push(json!({ "throws": "panic" }));
            return steps;
        };
        for op in ops.as_array().expect("ops") {
            let int = |index: usize| op[index].as_i64().expect("an int") as i32;
            let step = catch(|| {
                let output = match op[0].as_str().expect("an op's name") {
                    "get" => Value::from(layer.get(int(1), int(2), int(3))),
                    "set" => {
                        layer.set(int(1), int(2), int(3), int(4));
                        Value::Null
                    }
                    "fill" => {
                        layer.fill(int(1));
                        Value::Null
                    }
                    "getData" => Value::from(hex(layer.get_data())),
                    "copy" => {
                        layer = layer.copy();
                        Value::Null
                    }
                    "isDefinitelyFilledWith" => {
                        Value::from(layer.is_definitely_filled_with(int(1)))
                    }
                    "layerToString" => Value::from(layer.layer_to_string(int(1))),
                    "toString" => Value::from(layer.to_string()),
                    name => panic!("unknown op {name}"),
                };
                json!([output, layer.is_empty(), layer.is_definitely_homogenous()])
            });
            match step {
                Ok(step) => steps.push(step),
                Err(_) => {
                    steps.push(json!({ "throws": "panic" }));
                    break;
                }
            }
        }
        steps
    }
}
