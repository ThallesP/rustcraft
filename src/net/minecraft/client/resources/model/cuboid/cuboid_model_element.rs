//! net/minecraft/client/resources/model/cuboid/CuboidModelElement.java
//!
//! The faces are Java's `EnumMap`, by direction in `Direction`'s order, which is the order they're
//! baked in.

use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_object::JsonObject;
use crate::net::minecraft::client::resources::model::cuboid::cuboid_face::CuboidFace;
use crate::net::minecraft::client::resources::model::cuboid::cuboid_rotation::{
    CuboidRotation, RotationValue,
};
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::util::gson_helper::GsonHelper;
use crate::org::joml::vector3f::Vector3f;

/// A box of a model, in sixteenths of a block, and its faces.
#[derive(Clone, Debug)]
pub struct CuboidModelElement {
    pub from: Vector3f,
    pub to: Vector3f,
    pub faces: Vec<(Direction, CuboidFace)>,
    pub rotation: Option<CuboidRotation>,
    pub shade_direction_override: Option<Direction>,
    pub light_emission: i32,
}

const MIN_EXTENT: f32 = -16.0;
const MAX_EXTENT: f32 = 32.0;
const DEFAULT_RESCALE: bool = false;
const DEFAULT_LIGHT_EMISSION: i32 = 0;

impl CuboidModelElement {
    pub fn new(
        from: Vector3f,
        to: Vector3f,
        faces: Vec<(Direction, CuboidFace)>,
    ) -> CuboidModelElement {
        CuboidModelElement {
            from,
            to,
            faces,
            rotation: None,
            shade_direction_override: None,
            light_emission: 0,
        }
    }

    /// `CuboidModelElement.Deserializer`; the `Err` is Java's `JsonParseException`.
    pub fn deserialize(json: &JsonElement) -> Result<CuboidModelElement, String> {
        let object = json.get_as_json_object();
        let from = get_position(object, "from")?;
        let to = get_position(object, "to")?;
        let rotation = get_rotation(object)?;
        let faces = get_faces(object)?;
        let shade_direction_override = get_shade_direction_override(object)?;
        let mut light_emission = DEFAULT_LIGHT_EMISSION;
        if object.has("light_emission") {
            let is_number = GsonHelper::is_number_value(object, "light_emission");
            if is_number {
                light_emission = GsonHelper::get_as_int(object, "light_emission")?;
            }

            if !is_number || !(0..=15).contains(&light_emission) {
                return Err(
                    "Expected 'light_emission' to be an Integer between (inclusive) 0 and 15"
                        .to_owned(),
                );
            }
        }

        Ok(CuboidModelElement {
            from,
            to,
            faces,
            rotation,
            shade_direction_override,
            light_emission,
        })
    }
}

fn get_shade_direction_override(object: &JsonObject) -> Result<Option<Direction>, String> {
    if !object.has("shade_direction_override") {
        return Ok(None);
    }
    let shade_direction_name = GsonHelper::get_as_string(object, "shade_direction_override")?;
    match Direction::by_name(&shade_direction_name) {
        Some(direction) => Ok(Some(direction)),
        None => Err(format!(
            "Unknown shade direction override: {shade_direction_name}"
        )),
    }
}

fn get_rotation(object: &JsonObject) -> Result<Option<CuboidRotation>, String> {
    if !object.has("rotation") {
        return Ok(None);
    }

    let rotation_object = GsonHelper::get_as_json_object(object, "rotation")?;
    let mut origin = get_vector3f(rotation_object, "origin")?;
    origin.mul_scalar(0.0625);
    let rotation_value = if !rotation_object.has("axis") && !rotation_object.has("angle") {
        if !rotation_object.has("x") && !rotation_object.has("y") && !rotation_object.has("z") {
            return Err(
                "Missing rotation value, expected either 'axis' and 'angle' or 'x', 'y' and 'z'"
                    .to_owned(),
            );
        }

        RotationValue::EulerXYZ {
            x: GsonHelper::get_as_float_or(rotation_object, "x", 0.0)?,
            y: GsonHelper::get_as_float_or(rotation_object, "y", 0.0)?,
            z: GsonHelper::get_as_float_or(rotation_object, "z", 0.0)?,
        }
    } else {
        let axis = get_axis(rotation_object)?;
        let angle = GsonHelper::get_as_float(rotation_object, "angle")?;
        RotationValue::SingleAxis { axis, angle }
    };

    let rescale = GsonHelper::get_as_boolean_or(rotation_object, "rescale", DEFAULT_RESCALE)?;
    Ok(Some(CuboidRotation::new(origin, rotation_value, rescale)))
}

fn get_axis(object: &JsonObject) -> Result<Axis, String> {
    let axis_name = GsonHelper::get_as_string(object, "axis")?;
    Axis::by_name(&axis_name.to_lowercase())
        .ok_or_else(|| format!("Invalid rotation axis: {axis_name}"))
}

fn get_faces(object: &JsonObject) -> Result<Vec<(Direction, CuboidFace)>, String> {
    let faces = filter_null_from_faces(object)?;
    if faces.is_empty() {
        Err("Expected between 1 and 6 unique faces, got 0".to_owned())
    } else {
        Ok(faces)
    }
}

fn filter_null_from_faces(object: &JsonObject) -> Result<Vec<(Direction, CuboidFace)>, String> {
    let mut result: [Option<CuboidFace>; 6] = Default::default();
    let face_objects = GsonHelper::get_as_json_object(object, "faces")?;
    for (name, value) in face_objects.entry_set() {
        let direction =
            Direction::by_name(name).ok_or_else(|| format!("Unknown facing: {name}"))?;
        result[direction as usize] = Some(CuboidFace::deserialize(value)?);
    }
    Ok(Direction::VALUES
        .into_iter()
        .zip(result)
        .filter_map(|(direction, face)| Some((direction, face?)))
        .collect())
}

// Java's negated comparisons, which let NaN through.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn get_position(object: &JsonObject, key: &str) -> Result<Vector3f, String> {
    let from = get_vector3f(object, key)?;
    if !(from.x < MIN_EXTENT)
        && !(from.y < MIN_EXTENT)
        && !(from.z < MIN_EXTENT)
        && !(from.x > MAX_EXTENT)
        && !(from.y > MAX_EXTENT)
        && !(from.z > MAX_EXTENT)
    {
        Ok(from)
    } else {
        Err(format!(
            "'{key}' specifier exceeds the allowed boundaries: {from:?}"
        ))
    }
}

fn get_vector3f(object: &JsonObject, key: &str) -> Result<Vector3f, String> {
    let vec_array = GsonHelper::get_as_json_array(object, key)?;
    if vec_array.len() != 3 {
        return Err(format!(
            "Expected 3 {key} values, found: {}",
            vec_array.len()
        ));
    }

    let mut elements = [0.0f32; 3];
    for (i, element) in elements.iter_mut().enumerate() {
        *element = GsonHelper::convert_to_float(&vec_array[i], &format!("{key}[{i}]"))?;
    }
    Ok(Vector3f::from_xyz(elements[0], elements[1], elements[2]))
}
