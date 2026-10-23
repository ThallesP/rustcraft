//! net/minecraft/client/renderer/block/dispatch/multipart/Selector.java
//!
//! `CODEC` (`when`, an optional condition, and `apply`, a model) is decoded by hand
//! (`Selector::parse`).

use std::sync::Arc;

use crate::com::google::gson::json_element::JsonElement;
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model;
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_dispatcher::parse_unbaked;
use crate::net::minecraft::client::renderer::block::dispatch::multipart::condition::Condition;
use crate::net::minecraft::client::renderer::block::dispatch::variant_selector::Predicate;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;

pub struct Selector {
    pub condition: Option<Condition>,
    pub variant: Arc<dyn block_state_model::Unbaked>,
}

impl Selector {
    pub fn instantiate(&self, definition: &StateDefinition) -> Result<Predicate, String> {
        match &self.condition {
            Some(condition) => condition.instantiate(definition),
            None => Ok(Box::new(|_: BlockState| true)),
        }
    }

    pub fn parse(value: &JsonElement) -> Result<Selector, String> {
        let object = value
            .as_json_object()
            .ok_or_else(|| format!("Not a JSON object: {}", value.to_json_string()))?;
        let condition = object.get("when").map(Condition::parse).transpose()?;
        let variant = parse_unbaked(object.get("apply").ok_or("No key apply in MapLike")?)?;
        Ok(Selector { condition, variant })
    }
}
