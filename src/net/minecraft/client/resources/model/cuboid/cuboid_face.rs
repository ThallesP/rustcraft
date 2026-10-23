//! net/minecraft/client/resources/model/cuboid/CuboidFace.java

use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_object::JsonObject;
use crate::com::mojang::math::quadrant::Quadrant;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::gson_helper::GsonHelper;

/// One face of an element: the side it's culled with, its tint index, its texture slot, its UVs
/// (from the element's box when it has none) and their rotation.
#[derive(Clone, Debug, PartialEq)]
pub struct CuboidFace {
    pub cull_for_direction: Option<Direction>,
    pub tint_index: i32,
    pub texture: String,
    pub uvs: Option<UVs>,
    pub rotation: Quadrant,
}

/// `CuboidFace.UVs`, in sixteenths of the texture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UVs {
    pub min_u: f32,
    pub min_v: f32,
    pub max_u: f32,
    pub max_v: f32,
}

impl CuboidFace {
    pub const NO_TINT: i32 = -1;

    pub fn get_u(uvs: &UVs, rotation: Quadrant, vertex: i32) -> f32 {
        uvs.get_vertex_u(rotation.rotate_vertex_index(vertex)) / 16.0
    }

    pub fn get_v(uvs: &UVs, rotation: Quadrant, index: i32) -> f32 {
        uvs.get_vertex_v(rotation.rotate_vertex_index(index)) / 16.0
    }

    /// `CuboidFace.Deserializer`; the `Err` is Java's `JsonParseException`.
    pub fn deserialize(json: &JsonElement) -> Result<CuboidFace, String> {
        let object = json.get_as_json_object();
        let cull_direction =
            Direction::by_name(&GsonHelper::get_as_string_or(object, "cullface", "")?);
        let tint_index = GsonHelper::get_as_int_or(object, "tintindex", CuboidFace::NO_TINT)?;
        let texture = GsonHelper::get_as_string(object, "texture")?;
        let uvs = CuboidFace::get_uvs(object)?;
        let rotation = Quadrant::parse_json(GsonHelper::get_as_int_or(object, "rotation", 0)?)?;
        Ok(CuboidFace {
            cull_for_direction: cull_direction,
            tint_index,
            texture,
            uvs,
            rotation,
        })
    }

    fn get_uvs(object: &JsonObject) -> Result<Option<UVs>, String> {
        if !object.has("uv") {
            return Ok(None);
        }

        let uv_array = GsonHelper::get_as_json_array(object, "uv")?;
        if uv_array.len() != 4 {
            return Err(format!("Expected 4 uv values, found: {}", uv_array.len()));
        }

        Ok(Some(UVs {
            min_u: GsonHelper::convert_to_float(&uv_array[0], "minU")?,
            min_v: GsonHelper::convert_to_float(&uv_array[1], "minV")?,
            max_u: GsonHelper::convert_to_float(&uv_array[2], "maxU")?,
            max_v: GsonHelper::convert_to_float(&uv_array[3], "maxV")?,
        }))
    }
}

impl UVs {
    pub fn get_vertex_u(&self, index: i32) -> f32 {
        if index != 0 && index != 1 {
            self.max_u
        } else {
            self.min_u
        }
    }

    pub fn get_vertex_v(&self, index: i32) -> f32 {
        if index != 0 && index != 3 {
            self.max_v
        } else {
            self.min_v
        }
    }
}
