//! net/minecraft/util/parsing/packrat/Scope.java
//!
//! The values terms have parsed, in frames: a rule's own (`pushFrame`), and an alternative's
//! copy of its rule's keys, which it merges back when an alternative succeeds or drops when none
//! does. Lookups only see the top frame. Java keeps keys and values in one array, two slots an
//! entry; here an entry is one slot of a `Vec`, so every index is Java's halved. A key whose value
//! was cleared (an alternative that failed after storing it) stays with no value, which getters
//! hand out as `None`, Java's `null`: a key there without a value is a different answer from no
//! key at all.
//!
//! Values are `Rc<dyn Any>`, cloned out as the atom's type. Not ported: `validateStructure`, which
//! only Java's assertions run, and `lastFrame`, which only Mojang's tests call. `toString` (what
//! `parseTopRule`'s `IllegalStateException` prints) shows the atoms, but a value only as `?`.

use std::any::Any;
use std::fmt;
use std::rc::Rc;

use crate::net::minecraft::util::parsing::packrat::atom::Atom;

#[derive(Clone)]
enum Slot {
    // `FRAME_START_MARKER`, with the index of the frame's marker before it (the root's is 0).
    Frame(usize),
    // An atom (its id and name) and its value, if any.
    Value(usize, &'static str, Option<Rc<dyn Any>>),
}

/// A stack of frames of atoms' values.
pub struct Scope {
    stack: Vec<Slot>,
    top_entry_key_index: usize,
    top_marker_key_index: usize,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            stack: vec![Slot::Frame(0)],
            top_entry_key_index: 0,
            top_marker_key_index: 0,
        }
    }

    fn key(&self, index: usize) -> Option<usize> {
        match self.stack[index] {
            Slot::Value(id, _, _) => Some(id),
            Slot::Frame(_) => None,
        }
    }

    // The slot of the top frame's entry for the atom.
    fn value_index(&self, atom: usize) -> Option<usize> {
        (self.top_marker_key_index + 1..=self.top_entry_key_index)
            .rev()
            .find(|&i| self.key(i) == Some(atom))
    }

    // The slot of the top frame's latest entry for any of the atoms.
    fn value_index_for_any(&self, atoms: &[usize]) -> Option<usize> {
        (self.top_marker_key_index + 1..=self.top_entry_key_index)
            .rev()
            .find(|&i| self.key(i).is_some_and(|key| atoms.contains(&key)))
    }

    // Java's array grows to fit; slots past the top are stale.
    fn write(&mut self, index: usize, slot: Slot) {
        if index < self.stack.len() {
            self.stack[index] = slot;
        } else {
            self.stack.push(slot);
        }
    }

    fn setup_new_frame(&mut self) {
        self.top_entry_key_index += 1;
        self.write(
            self.top_entry_key_index,
            Slot::Frame(self.top_marker_key_index),
        );
        self.top_marker_key_index = self.top_entry_key_index;
    }

    pub fn push_frame(&mut self) {
        self.setup_new_frame();
    }

    fn get_previous_marker_index(&self, marker_key_index: usize) -> usize {
        match self.stack[marker_key_index] {
            Slot::Frame(previous) => previous,
            Slot::Value(..) => panic!("java.lang.ClassCastException: not a frame marker"),
        }
    }

    pub fn pop_frame(&mut self) {
        debug_assert!(self.top_marker_key_index != 0);
        self.top_entry_key_index = self.top_marker_key_index - 1;
        self.top_marker_key_index = self.get_previous_marker_index(self.top_marker_key_index);
    }

    /// A new frame with the top frame's atoms, without their values.
    pub fn split_frame(&mut self) {
        let current_frame_marker_index = self.top_marker_key_index;
        let non_marker_entries_in_frame = self.top_entry_key_index - self.top_marker_key_index;
        self.setup_new_frame();
        let source_cursor = current_frame_marker_index + 1;
        let mut target_cursor = self.top_entry_key_index;
        for source in source_cursor..source_cursor + non_marker_entries_in_frame {
            target_cursor += 1;
            let Slot::Value(key, name, _) = self.stack[source] else {
                panic!("a frame marker inside a frame");
            };
            self.write(target_cursor, Slot::Value(key, name, None));
        }
        self.top_entry_key_index = target_cursor;
    }

    pub fn clear_frame_values(&mut self) {
        for i in self.top_marker_key_index + 1..=self.top_entry_key_index {
            if let Slot::Value(_, _, value) = &mut self.stack[i] {
                *value = None;
            }
        }
    }

    /// Merges the top frame into the one before: its atoms go where the frame before has the
    /// same atom at the same place (its values where it has one), the rest over what's there.
    pub fn merge_frame(&mut self) {
        let previous_marker_index = self.get_previous_marker_index(self.top_marker_key_index);
        let mut previous_frame_cursor = previous_marker_index;
        let mut current_frame_cursor = self.top_marker_key_index;
        while current_frame_cursor < self.top_entry_key_index {
            previous_frame_cursor += 1;
            current_frame_cursor += 1;
            let Slot::Value(new_key, name, new_value) = self.stack[current_frame_cursor].clone()
            else {
                panic!("a frame marker inside a frame");
            };
            // Another atom there (or the marker) is overwritten; the same atom keeps its value
            // unless the frame has one.
            let old_key = self.key(previous_frame_cursor);
            if old_key != Some(new_key) || new_value.is_some() {
                self.stack[previous_frame_cursor] = Slot::Value(new_key, name, new_value);
            }
        }
        self.top_entry_key_index = previous_frame_cursor;
        self.top_marker_key_index = previous_marker_index;
    }

    pub fn put<T: 'static>(&mut self, name: &Atom<T>, value: T) {
        let value: Option<Rc<dyn Any>> = Some(Rc::new(value));
        match self.value_index(name.id()) {
            Some(value_index) => {
                self.stack[value_index] = Slot::Value(name.id(), name.name(), value);
            }
            None => {
                self.top_entry_key_index += 1;
                self.write(
                    self.top_entry_key_index,
                    Slot::Value(name.id(), name.name(), value),
                );
            }
        }
    }

    fn value_at<T: Clone + 'static>(&self, value_index: usize) -> Option<T> {
        match &self.stack[value_index] {
            Slot::Value(_, name, value) => value.as_ref().map(|value| {
                value
                    .downcast_ref::<T>()
                    .unwrap_or_else(|| panic!("java.lang.ClassCastException: <{name}>'s value"))
                    .clone()
            }),
            Slot::Frame(_) => panic!("a frame marker inside a frame"),
        }
    }

    /// The top frame's value of the atom: `None` when it has none, or no value.
    pub fn get<T: Clone + 'static>(&self, name: &Atom<T>) -> Option<T> {
        self.value_index(name.id())
            .and_then(|value_index| self.value_at(value_index))
    }

    /// The top frame's value of the atom, `None` when its value was cleared; panics without the
    /// atom, where Java throws `IllegalArgumentException`.
    pub fn get_or_throw<T: Clone + 'static>(&self, name: &Atom<T>) -> Option<T> {
        match self.value_index(name.id()) {
            Some(value_index) => self.value_at(value_index),
            None => panic!("java.lang.IllegalArgumentException: No value for atom {name}"),
        }
    }

    /// The top frame's value of the atom, the fallback without the atom, `None` when its value
    /// was cleared.
    pub fn get_or_default<T: Clone + 'static>(&self, name: &Atom<T>, fallback: T) -> Option<T> {
        match self.value_index(name.id()) {
            Some(value_index) => self.value_at(value_index),
            None => Some(fallback),
        }
    }

    /// The value of the atom of these the top frame got last, `None` when it has none or no value.
    pub fn get_any<T: Clone + 'static>(&self, names: &[&Atom<T>]) -> Option<T> {
        let ids: Vec<usize> = names.iter().map(|name| name.id()).collect();
        self.value_index_for_any(&ids)
            .and_then(|value_index| self.value_at(value_index))
    }

    /// `get_any`, panicking without any of the atoms, where Java throws
    /// `IllegalArgumentException`.
    pub fn get_any_or_throw<T: Clone + 'static>(&self, names: &[&Atom<T>]) -> Option<T> {
        let ids: Vec<usize> = names.iter().map(|name| name.id()).collect();
        match self.value_index_for_any(&ids) {
            Some(value_index) => self.value_at(value_index),
            None => {
                let names: Vec<String> = names.iter().map(ToString::to_string).collect();
                panic!(
                    "java.lang.IllegalArgumentException: No value for atoms [{}]",
                    names.join(", ")
                )
            }
        }
    }

    pub fn has_only_single_frame(&self) -> bool {
        if (1..=self.top_entry_key_index).any(|i| matches!(self.stack[i], Slot::Frame(_))) {
            return false;
        }
        assert!(
            matches!(self.stack[0], Slot::Frame(_)),
            "java.lang.IllegalStateException: Corrupted stack"
        );
        true
    }
}

impl Default for Scope {
    fn default() -> Self {
        Scope::new()
    }
}

/// `|` before each frame, then its atoms, `<name>:value`, the value as `?` or `null`.
impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut after_frame = true;
        for slot in &self.stack[..=self.top_entry_key_index] {
            match slot {
                Slot::Frame(_) => {
                    f.write_str("|")?;
                    after_frame = true;
                }
                Slot::Value(_, name, value) => {
                    if !after_frame {
                        f.write_str(",")?;
                    }
                    after_frame = false;
                    let value = if value.is_some() { "?" } else { "null" };
                    write!(f, "<{name}>:{value}")?;
                }
            }
        }
        Ok(())
    }
}
