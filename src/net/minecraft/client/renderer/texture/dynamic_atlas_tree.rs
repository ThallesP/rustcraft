//! net/minecraft/client/renderer/texture/DynamicAtlasTree.java (with DynamicAtlasTreeSlot.java)
//!
//! A rectangle split in two as slots are put in it: the slot at the top left, the rest to its
//! right or below (whichever's left over is wider), `spacing` pixels apart. A font texture's
//! glyphs go where its tree puts them, in the order they're first drawn.

pub struct DynamicAtlasTree {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    children: Option<Box<(DynamicAtlasTree, DynamicAtlasTree)>>,
    occupied: bool,
}

/// `DynamicAtlasTreeSlot`: where a slot went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicAtlasTreeSlot {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl DynamicAtlasTree {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> DynamicAtlasTree {
        DynamicAtlasTree {
            x,
            y,
            width,
            height,
            children: None,
            occupied: false,
        }
    }

    pub fn insert(
        &mut self,
        slot_width: i32,
        slot_height: i32,
        spacing: i32,
    ) -> Option<DynamicAtlasTreeSlot> {
        self.insert_inner(slot_width, slot_height, spacing)
    }

    fn insert_inner(
        &mut self,
        slot_width: i32,
        slot_height: i32,
        spacing: i32,
    ) -> Option<DynamicAtlasTreeSlot> {
        if let Some(children) = &mut self.children {
            let (left, right) = &mut **children;
            return left
                .insert_inner(slot_width, slot_height, spacing)
                .or_else(|| right.insert_inner(slot_width, slot_height, spacing));
        }
        if self.occupied {
            return None;
        }
        if slot_width > self.width || slot_height > self.height {
            return None;
        }
        if slot_width == self.width && slot_height == self.height {
            self.occupied = true;
            return Some(self.slot());
        }
        let delta_width = self.width - slot_width;
        let delta_height = self.height - slot_height;
        let children = if delta_width > delta_height {
            (
                DynamicAtlasTree::new(self.x, self.y, slot_width, self.height),
                DynamicAtlasTree::new(
                    self.x + slot_width + spacing,
                    self.y,
                    self.width - slot_width - spacing,
                    self.height,
                ),
            )
        } else {
            (
                DynamicAtlasTree::new(self.x, self.y, self.width, slot_height),
                DynamicAtlasTree::new(
                    self.x,
                    self.y + slot_height + spacing,
                    self.width,
                    self.height - slot_height - spacing,
                ),
            )
        };
        let children = self.children.insert(Box::new(children));
        children.0.insert_inner(slot_width, slot_height, spacing)
    }

    fn slot(&self) -> DynamicAtlasTreeSlot {
        DynamicAtlasTreeSlot {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }
}
