//! com/mojang/math/Quadrant.java
//!
//! Not ported yet: `CODEC`, which waits for DataFixerUpper's codecs (`Variant`'s rotations are
//! decoded by hand with `parse_json`'s values, `BlockStateModelDispatcher`).

use crate::com::mojang::math::octahedral_group::OctahedralGroup;
use crate::net::minecraft::util::mth::Mth;

/// A quarter turn count: 0, 90, 180 or 270 degrees.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Quadrant {
    R0,
    R90,
    R180,
    R270,
}

impl Quadrant {
    pub const VALUES: [Quadrant; 4] = [Quadrant::R0, Quadrant::R90, Quadrant::R180, Quadrant::R270];

    pub fn values() -> [Quadrant; 4] {
        Quadrant::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Quadrant {
        Quadrant::VALUES
            .into_iter()
            .find(|value| format!("{value:?}") == name)
            .unwrap_or_else(|| panic!("IllegalArgumentException: no Quadrant constant {name}"))
    }

    pub fn shift(&self) -> i32 {
        *self as i32
    }

    pub fn rotation_x(&self) -> OctahedralGroup {
        match self {
            Quadrant::R0 => OctahedralGroup::IDENTITY,
            Quadrant::R90 => OctahedralGroup::BLOCK_ROT_X_90,
            Quadrant::R180 => OctahedralGroup::BLOCK_ROT_X_180,
            Quadrant::R270 => OctahedralGroup::BLOCK_ROT_X_270,
        }
    }

    pub fn rotation_y(&self) -> OctahedralGroup {
        match self {
            Quadrant::R0 => OctahedralGroup::IDENTITY,
            Quadrant::R90 => OctahedralGroup::BLOCK_ROT_Y_90,
            Quadrant::R180 => OctahedralGroup::BLOCK_ROT_Y_180,
            Quadrant::R270 => OctahedralGroup::BLOCK_ROT_Y_270,
        }
    }

    pub fn rotation_z(&self) -> OctahedralGroup {
        match self {
            Quadrant::R0 => OctahedralGroup::IDENTITY,
            Quadrant::R90 => OctahedralGroup::BLOCK_ROT_Z_90,
            Quadrant::R180 => OctahedralGroup::BLOCK_ROT_Z_180,
            Quadrant::R270 => OctahedralGroup::BLOCK_ROT_Z_270,
        }
    }

    /// Any multiple of 90 degrees, negative ones too; Java throws `JsonParseException` for
    /// another angle, which is the `Err`.
    pub fn parse_json(degrees: i32) -> Result<Quadrant, String> {
        match Mth::positive_modulo_i32(degrees, 360) {
            0 => Ok(Quadrant::R0),
            90 => Ok(Quadrant::R90),
            180 => Ok(Quadrant::R180),
            270 => Ok(Quadrant::R270),
            _ => Err(format!(
                "Invalid rotation {degrees} found, only 0/90/180/270 allowed"
            )),
        }
    }

    pub fn from_xy_angles(x_rotation: Quadrant, y_rotation: Quadrant) -> OctahedralGroup {
        y_rotation.rotation_y().compose(x_rotation.rotation_x())
    }

    pub fn from_xyz_angles(
        x_rotation: Quadrant,
        y_rotation: Quadrant,
        z_rotation: Quadrant,
    ) -> OctahedralGroup {
        z_rotation
            .rotation_z()
            .compose(y_rotation.rotation_y().compose(x_rotation.rotation_x()))
    }

    pub fn rotate_vertex_index(&self, index: i32) -> i32 {
        (index.wrapping_add(self.shift())) % 4
    }
}

#[cfg(test)]
mod tests {
    use super::Quadrant;
    use crate::golden::{Golden, encode_names};

    encode_names!(Quadrant {
        R0,
        R90,
        R180,
        R270
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.math.Quadrant");
        for value in Quadrant::VALUES {
            golden.field(&format!("{value:?}"), value);
        }
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.check("values()[Lcom/mojang/math/Quadrant;", Quadrant::values);
        golden.check(
            "valueOf(Ljava/lang/String;)Lcom/mojang/math/Quadrant;",
            |name: String| Quadrant::value_of(&name),
        );
        golden.check("parseJson(I)Lcom/mojang/math/Quadrant;", |degrees: i32| {
            Quadrant::parse_json(degrees).unwrap_or_else(|message| panic!("{message}"))
        });
        golden.check(
            "fromXYAngles(Lcom/mojang/math/Quadrant;Lcom/mojang/math/Quadrant;)Lcom/mojang/math/OctahedralGroup;",
            Quadrant::from_xy_angles,
        );
        golden.check(
            "fromXYZAngles(Lcom/mojang/math/Quadrant;Lcom/mojang/math/Quadrant;Lcom/mojang/math/Quadrant;)Lcom/mojang/math/OctahedralGroup;",
            Quadrant::from_xyz_angles,
        );
        golden.check_method("rotateVertexIndex(I)I", |this: &mut Quadrant, index| {
            this.rotate_vertex_index(index)
        });
        golden.finish();
    }
}
