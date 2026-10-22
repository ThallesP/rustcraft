//! com/mojang/blaze3d/buffers/Std140Builder.java
//!
//! Writes a uniform block's members with std140's alignment, little-endian like LWJGL's native
//! buffers, from where the buffer's position was when it started. `onStack` is a buffer of the
//! size the builder owns, and `get()` its bytes up to the position (Java's `flip()`); bytes the
//! alignment skips are zero, where Java's stack memory holds whatever was there, which no shader
//! reads. `intoBuffer` comes with the dynamic uniforms' mapped buffers.

use crate::net::minecraft::util::mth::Mth;
use crate::org::joml::matrix4f::Matrix4f;

pub struct Std140Builder {
    buffer: Vec<u8>,
    position: usize,
    start: usize,
}

impl Std140Builder {
    pub fn on_stack(size: i32) -> Std140Builder {
        Std140Builder {
            buffer: vec![0; size as usize],
            position: 0,
            start: 0,
        }
    }

    /// The bytes written, from the buffer's start.
    pub fn get(self) -> Vec<u8> {
        let mut buffer = self.buffer;
        buffer.truncate(self.position);
        buffer
    }

    pub fn align(&mut self, alignment: i32) -> &mut Self {
        let written = (self.position - self.start) as i32;
        self.position = self.start + Mth::round_toward_i32(written, alignment) as usize;
        self
    }

    /// Panics where Java throws `BufferOverflowException`.
    fn put_bytes(&mut self, bytes: [u8; 4]) {
        self.buffer[self.position..self.position + 4].copy_from_slice(&bytes);
        self.position += 4;
    }

    pub fn put_float(&mut self, value: f32) -> &mut Self {
        self.align(4);
        self.put_bytes(value.to_le_bytes());
        self
    }

    pub fn put_int(&mut self, value: i32) -> &mut Self {
        self.align(4);
        self.put_bytes(value.to_le_bytes());
        self
    }

    pub fn put_vec2(&mut self, x: f32, y: f32) -> &mut Self {
        self.align(8);
        self.put_bytes(x.to_le_bytes());
        self.put_bytes(y.to_le_bytes());
        self
    }

    pub fn put_ivec2(&mut self, x: i32, y: i32) -> &mut Self {
        self.align(8);
        self.put_bytes(x.to_le_bytes());
        self.put_bytes(y.to_le_bytes());
        self
    }

    pub fn put_vec3(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.align(16);
        for value in [x, y, z] {
            self.put_bytes(value.to_le_bytes());
        }
        self
    }

    pub fn put_ivec3(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.align(16);
        for value in [x, y, z] {
            self.put_bytes(value.to_le_bytes());
        }
        self
    }

    pub fn put_vec4(&mut self, x: f32, y: f32, z: f32, w: f32) -> &mut Self {
        self.align(16);
        for value in [x, y, z, w] {
            self.put_bytes(value.to_le_bytes());
        }
        self
    }

    pub fn put_ivec4(&mut self, x: i32, y: i32, z: i32, w: i32) -> &mut Self {
        self.align(16);
        for value in [x, y, z, w] {
            self.put_bytes(value.to_le_bytes());
        }
        self
    }

    /// JOML's `get(ByteBuffer)`: the columns in order.
    pub fn put_mat4f(&mut self, matrix: &Matrix4f) -> &mut Self {
        self.align(16);
        for value in matrix.get_column_major() {
            self.put_bytes(value.to_le_bytes());
        }
        self
    }
}
