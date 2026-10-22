//! net/minecraft/util/DelegateDataOutput.java
//!
//! Only what `DataOutput` asks of it here: `NbtIo.StringFallbackDataOutput` holds one and
//! changes `writeUTF`.

use crate::java::io::data_output::DataOutput;

/// Writes everything to another output.
pub struct DelegateDataOutput<'a> {
    parent: &'a mut dyn DataOutput,
}

impl<'a> DelegateDataOutput<'a> {
    pub fn new(parent: &'a mut dyn DataOutput) -> DelegateDataOutput<'a> {
        DelegateDataOutput { parent }
    }
}

impl DataOutput for DelegateDataOutput<'_> {
    fn write(&mut self, b: &[i8]) {
        self.parent.write(b);
    }

    fn write_range(&mut self, b: &[i8], off: i32, len: i32) {
        self.parent.write_range(b, off, len);
    }

    fn write_byte(&mut self, v: i32) {
        self.parent.write_byte(v);
    }

    fn write_short(&mut self, v: i32) {
        self.parent.write_short(v);
    }

    fn write_int(&mut self, v: i32) {
        self.parent.write_int(v);
    }

    fn write_long(&mut self, v: i64) {
        self.parent.write_long(v);
    }

    fn write_float(&mut self, v: f32) {
        self.parent.write_float(v);
    }

    fn write_double(&mut self, v: f64) {
        self.parent.write_double(v);
    }

    fn write_utf(&mut self, s: &str) {
        self.parent.write_utf(s);
    }
}
