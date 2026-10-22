//! org/joml/Vector2f.java (JOML 1.10.9)
//!
//! Only the methods the client calls so far: the golden suite in oracle/golden/ lists them.

#[derive(Clone, Copy, Debug)]
pub struct Vector2f {
    pub x: f32,
    pub y: f32,
}

impl Default for Vector2f {
    fn default() -> Vector2f {
        Vector2f::new()
    }
}

impl Vector2f {
    /// `(0, 0)`.
    pub const fn new() -> Vector2f {
        Vector2f::from_xy(0.0, 0.0)
    }

    pub const fn from_xy(x: f32, y: f32) -> Vector2f {
        Vector2f { x, y }
    }

    pub fn set_xy(&mut self, x: f32, y: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn zero(&mut self) -> &mut Self {
        self.set_xy(0.0, 0.0)
    }

    pub fn add_xy(&mut self, x: f32, y: f32) -> &mut Self {
        let this = *self;
        this.add_xy_into(x, y, self)
    }

    pub fn add_xy_into<'a>(&self, x: f32, y: f32, dest: &'a mut Vector2f) -> &'a mut Vector2f {
        dest.x = self.x + x;
        dest.y = self.y + y;
        dest
    }

    pub fn mul_xy(&mut self, x: f32, y: f32) -> &mut Self {
        let this = *self;
        this.mul_xy_into(x, y, self)
    }

    pub fn mul_xy_into<'a>(&self, x: f32, y: f32, dest: &'a mut Vector2f) -> &'a mut Vector2f {
        dest.x = self.x * x;
        dest.y = self.y * y;
        dest
    }
}

#[cfg(test)]
mod tests {
    use super::Vector2f;
    use crate::golden::{Golden, encode_fields};

    encode_fields!(Vector2f { x: f32, y: f32 });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Vector2f");
        golden.check("<init>()V", Vector2f::new);
        golden.check("<init>(FF)V", Vector2f::from_xy);
        golden.check_method("set(FF)Lorg/joml/Vector2f;", |this: &mut Vector2f, x, y| {
            *this.set_xy(x, y)
        });
        golden.check_method("zero()Lorg/joml/Vector2f;", |this: &mut Vector2f| {
            *this.zero()
        });
        golden.check_method("add(FF)Lorg/joml/Vector2f;", |this: &mut Vector2f, x, y| {
            *this.add_xy(x, y)
        });
        golden.check_method(
            "add(FFLorg/joml/Vector2f;)Lorg/joml/Vector2f;",
            |this: &mut Vector2f, x, y, mut dest: Vector2f| *this.add_xy_into(x, y, &mut dest),
        );
        golden.check_method("mul(FF)Lorg/joml/Vector2f;", |this: &mut Vector2f, x, y| {
            *this.mul_xy(x, y)
        });
        golden.check_method(
            "mul(FFLorg/joml/Vector2f;)Lorg/joml/Vector2f;",
            |this: &mut Vector2f, x, y, mut dest: Vector2f| *this.mul_xy_into(x, y, &mut dest),
        );
        golden.finish();
    }
}
