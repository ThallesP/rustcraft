//! net/minecraft/client/renderer/block/dispatch/Variant.java
//!
//! Not ported yet: `VariantMutator`'s `with…` helpers, which only the data generator calls.
//! `MAP_CODEC` (DataFixerUpper's) is decoded by hand from Gson's tree (`Variant::parse`).

use std::sync::Arc;

use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_object::JsonObject;
use crate::com::mojang::math::quadrant::Quadrant;
use crate::net::minecraft::client::renderer::block::dispatch::block_model_rotation::BlockModelRotation;
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_part::{
    self, BlockStateModelPart,
};
use crate::net::minecraft::client::renderer::block::dispatch::model_state::ModelState;
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::resolvable_model::{
    ResolvableModel, Resolver,
};
use crate::net::minecraft::client::resources::model::simple_model_wrapper::SimpleModelWrapper;
use crate::net::minecraft::resources::identifier::Identifier;

/// A model, turned by quarter turns about x, then y, then z, with its textures locked or not.
#[derive(Clone, Debug, PartialEq)]
pub struct Variant {
    pub model_location: Identifier,
    pub model_state: SimpleModelState,
}

/// `Variant.SimpleModelState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimpleModelState {
    pub x: Quadrant,
    pub y: Quadrant,
    pub z: Quadrant,
    pub uv_lock: bool,
}

impl Variant {
    pub fn new(model_location: Identifier) -> Variant {
        Variant {
            model_location,
            model_state: SimpleModelState::DEFAULT,
        }
    }

    /// `MAP_CODEC`'s fields: `model`, an identifier, and the state's optional `x`, `y`, `z` and
    /// `uvlock`. Other fields are left alone, like a map codec does.
    pub fn parse(object: &JsonObject) -> Result<Variant, String> {
        let model = object.get("model").ok_or("No key model in MapLike")?;
        let model_location = parse_identifier(model)?;
        Ok(Variant {
            model_location,
            model_state: SimpleModelState::parse(object)?,
        })
    }
}

impl block_state_model_part::Unbaked for Variant {
    fn bake(&self, model_bakery: &dyn ModelBaker) -> Arc<dyn BlockStateModelPart> {
        SimpleModelWrapper::bake(
            model_bakery,
            &self.model_location,
            self.model_state.as_model_state(),
        )
    }
}

impl ResolvableModel for Variant {
    fn resolve_dependencies(&self, resolver: &mut dyn Resolver) {
        resolver.mark_dependency(&self.model_location);
    }
}

impl SimpleModelState {
    pub const DEFAULT: SimpleModelState = SimpleModelState {
        x: Quadrant::R0,
        y: Quadrant::R0,
        z: Quadrant::R0,
        uv_lock: false,
    };

    pub fn as_model_state(&self) -> &'static dyn ModelState {
        let rotation = BlockModelRotation::get(Quadrant::from_xyz_angles(self.x, self.y, self.z));
        if self.uv_lock {
            rotation.with_uv_lock()
        } else {
            rotation
        }
    }

    fn parse(object: &JsonObject) -> Result<SimpleModelState, String> {
        let quadrant = |name: &str| match object.get(name) {
            None => Ok(Quadrant::R0),
            Some(value) => Quadrant::parse_json(parse_int(value)?),
        };
        Ok(SimpleModelState {
            x: quadrant("x")?,
            y: quadrant("y")?,
            z: quadrant("z")?,
            uv_lock: match object.get("uvlock") {
                None => false,
                Some(value) => parse_bool(value)?,
            },
        })
    }
}

/// `Codec.INT` through `JsonOps`: a number's `intValue()`, or a boolean as 1 or 0.
pub(crate) fn parse_int(value: &JsonElement) -> Result<i32, String> {
    match value.as_json_primitive() {
        Some(primitive) if primitive.is_number() => Ok(primitive.get_as_int()),
        Some(primitive) if primitive.is_boolean() => Ok(i32::from(primitive.get_as_boolean())),
        _ => Err(format!("Not a number: {value}")),
    }
}

/// `Codec.BOOL` through `JsonOps`: a boolean, or a number as whether its byte isn't 0.
pub(crate) fn parse_bool(value: &JsonElement) -> Result<bool, String> {
    match value.as_json_primitive() {
        Some(primitive) if primitive.is_boolean() => Ok(primitive.get_as_boolean()),
        Some(primitive) if primitive.is_number() => Ok(primitive.get_as_int() as i8 != 0),
        _ => Err(format!("Not a boolean: {value}")),
    }
}

/// `Codec.STRING` through `JsonOps`: only a JSON string.
pub(crate) fn parse_string(value: &JsonElement) -> Result<String, String> {
    match value.as_json_primitive() {
        Some(primitive) if primitive.is_string() => Ok(primitive.get_as_string()),
        _ => Err(format!("Not a string: {value}")),
    }
}

/// `Identifier.CODEC`.
pub(crate) fn parse_identifier(value: &JsonElement) -> Result<Identifier, String> {
    let text = parse_string(value)?;
    Identifier::try_parse(&text).ok_or_else(|| format!("Not a valid resource location: {text}"))
}
