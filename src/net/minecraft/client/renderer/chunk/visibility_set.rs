//! net/minecraft/client/renderer/chunk/VisibilitySet.java
//!
//! Java's `BitSet` of 36 bits is one word, whose 64 bits `setAll` sets, so it's a `u64`.

use std::fmt;

use crate::net::minecraft::core::direction::Direction;

/// Which pairs of a section's faces see each other through it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisibilitySet {
    data: u64,
}

impl VisibilitySet {
    const FACINGS: u32 = 6;

    pub fn new() -> VisibilitySet {
        VisibilitySet::default()
    }

    /// Each pair of the directions, both ways, `directions` in `Direction`'s order like an
    /// `EnumSet`.
    pub fn add(&mut self, directions: &[Direction]) {
        for &direction1 in directions {
            for &direction2 in directions {
                self.set(direction1, direction2, true);
            }
        }
    }

    pub fn set(&mut self, direction1: Direction, direction2: Direction, value: bool) {
        let bit1 = direction1 as u32 + direction2 as u32 * VisibilitySet::FACINGS;
        let bit2 = direction2 as u32 + direction1 as u32 * VisibilitySet::FACINGS;
        for bit in [bit1, bit2] {
            if value {
                self.data |= 1 << bit;
            } else {
                self.data &= !(1 << bit);
            }
        }
    }

    pub fn set_all(&mut self, visible: bool) {
        self.data = if visible { u64::MAX } else { 0 };
    }

    pub fn visibility_between(&self, direction1: Direction, direction2: Direction) -> bool {
        self.data >> (direction1 as u32 + direction2 as u32 * VisibilitySet::FACINGS) & 1 != 0
    }
}

impl fmt::Display for VisibilitySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let initial = |direction: Direction| {
            direction
                .get_name()
                .chars()
                .next()
                .expect("a name")
                .to_ascii_uppercase()
        };
        write!(f, " ")?;
        for direction in Direction::VALUES {
            write!(f, " {}", initial(direction))?;
        }
        writeln!(f)?;
        for direction1 in Direction::VALUES {
            write!(f, "{}", initial(direction1))?;
            for direction2 in Direction::VALUES {
                if direction1 == direction2 {
                    write!(f, "  ")?;
                } else {
                    let ok = self.visibility_between(direction1, direction2);
                    write!(f, " {}", if ok { 'Y' } else { 'n' })?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
