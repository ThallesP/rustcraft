//! net/minecraft/client/ComponentCollector.java
//!
//! Pieces of text gathered into one: here styled strings, which is all its callers append, made
//! into a `Composite` (Java hands a single piece back as itself, which visits the same).

use crate::net::minecraft::network::chat::formatted_text::{Composite, OfStyled};

#[derive(Default)]
pub struct ComponentCollector {
    parts: Vec<OfStyled>,
}

impl ComponentCollector {
    pub fn new() -> ComponentCollector {
        ComponentCollector::default()
    }

    pub fn append(&mut self, component: OfStyled) {
        self.parts.push(component);
    }

    /// `getResult`: none when nothing was appended.
    pub fn get_result(&self) -> Option<Composite> {
        if self.parts.is_empty() {
            None
        } else {
            Some(Composite(self.parts.clone()))
        }
    }

    pub fn get_result_or_empty(&self) -> Composite {
        self.get_result().unwrap_or(Composite::EMPTY)
    }

    pub fn reset(&mut self) {
        self.parts.clear();
    }
}
