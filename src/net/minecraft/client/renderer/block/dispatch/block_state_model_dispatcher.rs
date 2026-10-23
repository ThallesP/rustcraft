//! net/minecraft/client/renderer/block/dispatch/BlockStateModelDispatcher.java
//!
//! `CODEC` and the codecs it's made of (DataFixerUpper's) are decoded by hand from Gson's tree
//! (`parse`), the way they read it: `variants`, a non-empty map of selectors to models, and
//! `multipart`, a non-empty list of selectors, at least one of them; a model is a non-empty list
//! of weighted variants, else one variant. The maps keep the file's order, like Gson's.

use std::collections::HashMap;
use std::sync::Arc;

use crate::com::google::gson::json_element::{JsonElement, JsonObject};
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model::{
    self, UnbakedRoot,
};
use crate::net::minecraft::client::renderer::block::dispatch::multipart::multi_part_model::{
    self, MultiPartModel,
};
use crate::net::minecraft::client::renderer::block::dispatch::multipart::selector::Selector;
use crate::net::minecraft::client::renderer::block::dispatch::single_variant;
use crate::net::minecraft::client::renderer::block::dispatch::variant::{Variant, parse_int};
use crate::net::minecraft::client::renderer::block::dispatch::variant_selector::VariantSelector;
use crate::net::minecraft::client::renderer::block::dispatch::weighted_variants;
use crate::net::minecraft::util::random::weighted::Weighted;
use crate::net::minecraft::util::random::weighted_list::WeightedList;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;

pub struct BlockStateModelDispatcher {
    pub simple_models: Option<SimpleModelSelectors>,
    pub multi_part: Option<MultiPartDefinition>,
}

/// `BlockStateModelDispatcher.MultiPartDefinition`.
pub struct MultiPartDefinition {
    pub selectors: Vec<Selector>,
}

/// `BlockStateModelDispatcher.SimpleModelSelectors`: each `variants` key and its model.
pub struct SimpleModelSelectors {
    pub models: Vec<(String, Arc<dyn block_state_model::Unbaked>)>,
}

impl BlockStateModelDispatcher {
    /// `CODEC`; the `Err` is the `JsonParseException` `getOrThrow` throws.
    pub fn parse(json: &JsonElement) -> Result<BlockStateModelDispatcher, String> {
        let object = json
            .as_json_object()
            .ok_or_else(|| format!("Not a JSON object: {}", json.to_json_string()))?;
        let simple_models = object
            .get("variants")
            .map(SimpleModelSelectors::parse)
            .transpose()?;
        let multi_part = object
            .get("multipart")
            .map(MultiPartDefinition::parse)
            .transpose()?;
        if simple_models.is_none() && multi_part.is_none() {
            return Err("Neither 'variants' nor 'multipart' found".to_owned());
        }
        Ok(BlockStateModelDispatcher {
            simple_models,
            multi_part,
        })
    }

    /// Each state's model: the variants' first, then the multipart's for the states they leave.
    /// The `Err` is what Java throws instantiating the multipart's conditions.
    // A state hashes its definition's address and its index, which never change: the definition's
    // cell is its registration, set once.
    #[allow(clippy::mutable_key_type)]
    pub fn instantiate(
        &self,
        state_definition: &'static StateDefinition,
        source: &dyn Fn() -> String,
    ) -> Result<HashMap<BlockState, Arc<dyn UnbakedRoot>>, String> {
        let mut matched_states: HashMap<BlockState, Arc<dyn UnbakedRoot>> = HashMap::new();
        if let Some(simple_models) = &self.simple_models {
            simple_models.instantiate(state_definition, source, &mut |state, model| {
                match matched_states.insert(state, model) {
                    Some(_) => Err(format!("Overlapping definition on state: {state}")),
                    None => Ok(()),
                }
            });
        }
        if let Some(multi_part) = &self.multi_part {
            let model: Arc<dyn UnbakedRoot> = Arc::new(multi_part.instantiate(state_definition)?);
            for state in state_definition.get_possible_states() {
                matched_states.entry(state).or_insert_with(|| model.clone());
            }
        }
        Ok(matched_states)
    }
}

impl MultiPartDefinition {
    fn parse(json: &JsonElement) -> Result<MultiPartDefinition, String> {
        let list = json
            .as_json_array()
            .ok_or_else(|| format!("Not a json array: {}", json.to_json_string()))?;
        if list.is_empty() {
            return Err("List must have contents".to_owned());
        }
        Ok(MultiPartDefinition {
            selectors: list.iter().map(Selector::parse).collect::<Result<_, _>>()?,
        })
    }

    pub fn instantiate(
        &self,
        state_definition: &StateDefinition,
    ) -> Result<multi_part_model::Unbaked, String> {
        let mut instantiated_selectors = Vec::with_capacity(self.selectors.len());
        for selector in &self.selectors {
            instantiated_selectors.push(multi_part_model::Selector {
                condition: Arc::new(selector.instantiate(state_definition)?),
                model: selector.variant.clone(),
            });
        }
        Ok(MultiPartModel::unbaked(instantiated_selectors))
    }
}

impl SimpleModelSelectors {
    fn parse(json: &JsonElement) -> Result<SimpleModelSelectors, String> {
        let object = json
            .as_json_object()
            .ok_or_else(|| format!("Not a JSON object: {}", json.to_json_string()))?;
        if object.is_empty() {
            return Err("Map must not be empty".to_owned());
        }
        Ok(SimpleModelSelectors {
            models: object
                .entry_set()
                .map(|(key, value)| Ok((key.clone(), parse_unbaked(value)?)))
                .collect::<Result<_, String>>()?,
        })
    }

    /// Each selector's states get its model, in the file's order. Java logs a selector that names
    /// a property or value the block hasn't, or overlaps an earlier one, and goes on with the
    /// next: an overlap stops the selector after the state it overlapped on was given the model.
    pub fn instantiate(
        &self,
        state_definition: &'static StateDefinition,
        source: &dyn Fn() -> String,
        output: &mut dyn FnMut(BlockState, Arc<dyn UnbakedRoot>) -> Result<(), String>,
    ) {
        for (selector_string, model) in &self.models {
            let result = (|| {
                let selector = VariantSelector::predicate(state_definition, selector_string)?;
                let wrapper = block_state_model::as_root(model.clone());
                for state in state_definition.get_possible_states() {
                    if selector(state) {
                        output(state, wrapper.clone())?;
                    }
                }
                Ok::<(), String>(())
            })();
            if result.is_err() {
                eprintln!(
                    "Exception loading blockstate definition: '{}' for variant: '{selector_string}'",
                    source()
                );
            }
        }
    }
}

/// `BlockStateModel.Unbaked.CODEC`: a non-empty list of `{weight, …variant}`, else a variant.
pub fn parse_unbaked(json: &JsonElement) -> Result<Arc<dyn block_state_model::Unbaked>, String> {
    if let Some(list) = json.as_json_array() {
        let entries = (|| {
            if list.is_empty() {
                return Err("List must have contents".to_owned());
            }
            list.iter()
                .map(|element| {
                    let object = element.as_json_object().ok_or_else(|| {
                        format!("Not a JSON object: {}", element.to_json_string())
                    })?;
                    let variant = Variant::parse(object)?;
                    let weight = parse_weight(object)?;
                    let model: Arc<dyn block_state_model::Unbaked> =
                        Arc::new(single_variant::Unbaked { variant });
                    Ok(Weighted::new(model, weight))
                })
                .collect::<Result<Vec<_>, String>>()
        })();
        if let Ok(entries) = entries {
            return Ok(Arc::new(weighted_variants::Unbaked {
                entries: WeightedList::of(entries),
            }));
        }
    }
    let object: &JsonObject = json
        .as_json_object()
        .ok_or_else(|| format!("Not a JSON object: {}", json.to_json_string()))?;
    Ok(Arc::new(single_variant::Unbaked {
        variant: Variant::parse(object)?,
    }))
}

// `ExtraCodecs.POSITIVE_INT.optionalFieldOf("weight", 1)`.
fn parse_weight(object: &JsonObject) -> Result<i32, String> {
    match object.get("weight") {
        None => Ok(1),
        Some(value) => {
            let weight = parse_int(value)?;
            if weight >= 1 {
                Ok(weight)
            } else {
                Err(format!("Value must be positive: {weight}"))
            }
        }
    }
}
