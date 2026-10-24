//! net/minecraft/commands/arguments/selector/options/InvertableSetOptionState.java

use std::collections::HashSet;

use crate::net::minecraft::resources::identifier::Identifier;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Limitation {
    #[default]
    None,
    Single,
    Multiple,
}

/// An option a selector takes once as itself, or any number of times inverted (and as tags).
#[derive(Clone, Debug, Default)]
pub struct InvertableSetOptionState {
    state: Limitation,
    tags: HashSet<Identifier>,
}

impl InvertableSetOptionState {
    fn can_limit_to_single(&self) -> bool {
        self.state == Limitation::None
    }

    fn can_limit_to_multiple(&self) -> bool {
        self.state != Limitation::Single
    }

    pub fn can_parse_positive_element(&self) -> bool {
        self.can_limit_to_single()
    }

    pub fn can_parse_negative_element(&self) -> bool {
        self.can_limit_to_multiple()
    }

    pub fn can_parse_any_tag(&self) -> bool {
        self.can_limit_to_multiple()
    }

    pub fn can_parse_tag(&self, tag: &Identifier) -> bool {
        self.can_parse_any_tag() && !self.tags.contains(tag)
    }

    pub fn can_parse_any(&self) -> bool {
        self.state != Limitation::Single
    }

    pub fn can_parse_element(&self, inverted: bool) -> bool {
        if inverted {
            self.can_parse_negative_element()
        } else {
            self.can_parse_positive_element()
        }
    }

    pub fn mark_parsed_positive_element(&mut self) {
        self.state = Limitation::Single;
    }

    pub fn mark_parsed_negative_element(&mut self) {
        self.state = Limitation::Multiple;
    }

    pub fn mark_parsed_tag(&mut self, tag: Identifier) {
        self.state = Limitation::Multiple;
        self.tags.insert(tag);
    }

    pub fn mark_parsed_element(&mut self, inverted: bool) {
        if inverted {
            self.mark_parsed_negative_element();
        } else {
            self.mark_parsed_positive_element();
        }
    }
}
