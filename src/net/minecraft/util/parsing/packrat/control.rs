//! net/minecraft/util/parsing/packrat/Control.java
//!
//! What a cut term tells the alternative it's in: to try none of the alternatives after the one
//! it's in. Java's interface has two classes, `UNBOUND`'s, which forgets the cut (what a rule's
//! term gets), and the parse state's `SimpleControl`, which it hands out to each alternative and
//! reuses. Here that's one struct, holding the flag the parse state's pool shares, or none.

use std::cell::Cell;
use std::rc::Rc;

/// A cut's flag, or none to forget it.
#[derive(Clone, Debug)]
pub struct Control(Option<Rc<Cell<bool>>>);

impl Control {
    pub const UNBOUND: Control = Control(None);

    /// The parse state's `SimpleControl` over its flag.
    pub fn simple(has_cut: Rc<Cell<bool>>) -> Control {
        Control(Some(has_cut))
    }

    pub fn cut(&self) {
        if let Some(has_cut) = &self.0 {
            has_cut.set(true);
        }
    }

    pub fn has_cut(&self) -> bool {
        self.0.as_ref().is_some_and(|has_cut| has_cut.get())
    }
}
