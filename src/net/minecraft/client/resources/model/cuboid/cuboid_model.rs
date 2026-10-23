//! net/minecraft/client/resources/model/cuboid/CuboidModel.java
//!
//! Not ported yet: `transforms`, the `display` section's `ItemTransforms`, which come with item
//! models: a model's `display` is read past, so one that wouldn't decode isn't refused.

use std::sync::Arc;

use crate::com::google::gson::json_element::JsonObject;
use crate::com::google::gson::json_parser::JsonParser;
use crate::net::minecraft::client::resources::model::cuboid::cuboid_model_element::CuboidModelElement;
use crate::net::minecraft::client::resources::model::cuboid::unbaked_cuboid_geometry::UnbakedCuboidGeometry;
use crate::net::minecraft::client::resources::model::geometry::unbaked_geometry::UnbakedGeometry;
use crate::net::minecraft::client::resources::model::sprite::texture_slots::{Data, TextureSlots};
use crate::net::minecraft::client::resources::model::unbaked_model::{GuiLight, UnbakedModel};
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::gson_helper::GsonHelper;

/// A model file: its elements, if it has any, its texture slots, and its parent.
pub struct CuboidModel {
    pub geometry: Option<Arc<dyn UnbakedGeometry>>,
    pub gui_light: Option<GuiLight>,
    pub ambient_occlusion: Option<bool>,
    pub texture_slots: Data,
    pub parent: Option<Identifier>,
}

impl CuboidModel {
    /// `fromStream`: the file's JSON, read strictly (`GsonHelper.fromJson`), then
    /// `CuboidModel.Deserializer`. The `Err` is Java's `JsonParseException`.
    pub fn from_stream(text: &str) -> Result<CuboidModel, String> {
        let json = JsonParser::parse_strict(text)?;
        let object = json
            .as_json_object()
            .ok_or_else(|| format!("Not a JSON Object: {}", json.to_json_string()))?;
        CuboidModel::deserialize(object)
    }

    fn deserialize(object: &JsonObject) -> Result<CuboidModel, String> {
        let elements = CuboidModel::get_elements(object)?;
        let parent_name = GsonHelper::get_as_string_or(object, "parent", "")?;
        let texture_map = CuboidModel::get_texture_map(object)?;
        let has_ambient_occlusion = CuboidModel::get_ambient_occlusion(object)?;
        if object.has("display") {
            GsonHelper::get_as_json_object(object, "display")?;
        }

        let gui_light = if object.has("gui_light") {
            Some(GuiLight::get_by_name(&GsonHelper::get_as_string(
                object,
                "gui_light",
            )?)?)
        } else {
            None
        };

        let parent_location =
            if parent_name.is_empty() {
                None
            } else {
                Some(Identifier::try_parse(&parent_name).ok_or_else(|| {
                    format!("Non [a-z0-9_.-] character in identifier: {parent_name}")
                })?)
            };
        Ok(CuboidModel {
            geometry: elements,
            gui_light,
            ambient_occlusion: has_ambient_occlusion,
            texture_slots: texture_map,
            parent: parent_location,
        })
    }

    fn get_texture_map(object: &JsonObject) -> Result<Data, String> {
        if object.has("textures") {
            TextureSlots::parse_texture_map(GsonHelper::get_as_json_object(object, "textures")?)
        } else {
            Ok(Data::empty())
        }
    }

    fn get_ambient_occlusion(object: &JsonObject) -> Result<Option<bool>, String> {
        if object.has("ambientocclusion") {
            Ok(Some(GsonHelper::get_as_boolean(
                object,
                "ambientocclusion",
            )?))
        } else {
            Ok(None)
        }
    }

    fn get_elements(object: &JsonObject) -> Result<Option<Arc<dyn UnbakedGeometry>>, String> {
        if !object.has("elements") {
            return Ok(None);
        }

        let elements = GsonHelper::get_as_json_array(object, "elements")?
            .iter()
            .map(CuboidModelElement::deserialize)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(Arc::new(UnbakedCuboidGeometry::new(elements))))
    }
}

impl UnbakedModel for CuboidModel {
    fn ambient_occlusion(&self) -> Option<bool> {
        self.ambient_occlusion
    }

    fn gui_light(&self) -> Option<GuiLight> {
        self.gui_light
    }

    fn texture_slots(&self) -> &Data {
        &self.texture_slots
    }

    fn geometry(&self) -> Option<Arc<dyn UnbakedGeometry>> {
        self.geometry.clone()
    }

    fn parent(&self) -> Option<&Identifier> {
        self.parent.as_ref()
    }
}
