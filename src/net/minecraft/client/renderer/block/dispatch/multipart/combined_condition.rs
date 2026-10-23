//! net/minecraft/client/renderer/block/dispatch/multipart/CombinedCondition.java
//!
//! Not ported yet: `Operation.CODEC`, which `Condition::parse` stands in for.

use crate::net::minecraft::client::renderer::block::dispatch::multipart::condition::Condition;
use crate::net::minecraft::client::renderer::block::dispatch::variant_selector::Predicate;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;

#[derive(Clone, Debug, PartialEq)]
pub struct CombinedCondition {
    pub operation: Operation,
    pub terms: Vec<Condition>,
}

/// `CombinedCondition.Operation`: every term, or any.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    AND,
    OR,
}

impl Operation {
    /// By serialized name, which is the constant's.
    pub fn by_name(name: &str) -> Option<Operation> {
        match name {
            "AND" => Some(Operation::AND),
            "OR" => Some(Operation::OR),
            _ => None,
        }
    }

    /// `Util.allOf` or `Util.anyOf` of the terms.
    pub fn apply(&self, terms: Vec<Predicate>) -> Predicate {
        match self {
            Operation::AND => {
                Box::new(move |state: BlockState| terms.iter().all(|term| term(state)))
            }
            Operation::OR => {
                Box::new(move |state: BlockState| terms.iter().any(|term| term(state)))
            }
        }
    }
}

impl CombinedCondition {
    pub fn instantiate(&self, definition: &StateDefinition) -> Result<Predicate, String> {
        let terms = self
            .terms
            .iter()
            .map(|condition| condition.instantiate(definition))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self.operation.apply(terms))
    }
}
