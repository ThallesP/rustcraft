//! org/joml/Vector3i.java (JOML 1.10.9)
//!
//! Only the methods the client calls so far: the golden suite in oracle/golden/ lists them.

#[derive(Clone, Copy, Debug, Default)]
pub struct Vector3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vector3i {
    pub const fn from_xyz(x: i32, y: i32, z: i32) -> Vector3i {
        Vector3i { x, y, z }
    }

    pub fn set_xyz(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    /// Panics on a component other than 0, 1 or 2, where JOML throws
    /// `IllegalArgumentException`.
    pub fn get(&self, component: i32) -> i32 {
        match component {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("IllegalArgumentException: no component {component}"),
        }
    }

    /// Panics on a component other than 0, 1 or 2, like `get`.
    pub fn set_component(&mut self, component: i32, value: i32) -> &mut Self {
        match component {
            0 => self.x = value,
            1 => self.y = value,
            2 => self.z = value,
            _ => panic!("IllegalArgumentException: no component {component}"),
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::Vector3i;
    use crate::golden::{Golden, encode_fields};

    encode_fields!(Vector3i {
        x: i32,
        y: i32,
        z: i32
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("org.joml.Vector3i");
        golden.check("<init>(III)V", Vector3i::from_xyz);
        golden.check_method(
            "set(III)Lorg/joml/Vector3i;",
            |this: &mut Vector3i, x, y, z| *this.set_xyz(x, y, z),
        );
        golden.check_method("get(I)I", |this: &mut Vector3i, component| {
            this.get(component)
        });
        golden.check_method(
            "setComponent(II)Lorg/joml/Vector3i;",
            |this: &mut Vector3i, component, value| *this.set_component(component, value),
        );
        golden.finish();
    }
}
