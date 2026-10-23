//! net/minecraft/client/resources/model/UnbakedModel.java
//!
//! Not ported yet: `transforms()`, the item transforms a model's `display` gives, which come with
//! item models.

use std::sync::{Arc, LazyLock};

use crate::net::minecraft::client::resources::model::geometry::unbaked_geometry::UnbakedGeometry;
use crate::net::minecraft::client::resources::model::sprite::texture_slots::Data;
use crate::net::minecraft::resources::identifier::Identifier;

pub const PARTICLE_TEXTURE_REFERENCE: &str = "particle";

static EMPTY_TEXTURE_SLOTS: LazyLock<Data> = LazyLock::new(Data::empty);

/// A model as its file says it, each part `None` where the file leaves it to its parent.
pub trait UnbakedModel: Send + Sync {
    fn ambient_occlusion(&self) -> Option<bool> {
        None
    }

    fn gui_light(&self) -> Option<GuiLight> {
        None
    }

    fn texture_slots(&self) -> &Data {
        &EMPTY_TEXTURE_SLOTS
    }

    fn geometry(&self) -> Option<Arc<dyn UnbakedGeometry>> {
        None
    }

    fn parent(&self) -> Option<&Identifier> {
        None
    }
}

/// `UnbakedModel.GuiLight`: how an item drawn from the model is lit in a GUI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GuiLight {
    FRONT,
    SIDE,
}

impl GuiLight {
    /// Java throws `IllegalArgumentException` for another name, which is the `Err`.
    pub fn get_by_name(name: &str) -> Result<GuiLight, String> {
        match name {
            "front" => Ok(GuiLight::FRONT),
            "side" => Ok(GuiLight::SIDE),
            _ => Err(format!("Invalid gui light: {name}")),
        }
    }

    pub fn light_like_block(&self) -> bool {
        *self == GuiLight::SIDE
    }
}
