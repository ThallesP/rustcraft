//! net/minecraft/client/renderer/texture/Stitcher.java
//!
//! Packs sprites into an atlas: tallest first (then widest, then by name), each into the first
//! region with room, growing the atlas a power of two at a time along its shorter side. Each
//! sprite is padded by `1 << mipLevel` (more with anisotropy) and rounded up to the mip level's
//! texel size. Java's generic `T extends Entry` is any type with a name and a size; the regions
//! hold an entry's index into the registered list.

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::mth::Mth;
use std::cmp::Ordering;

/// `Stitcher.Entry`.
pub trait Entry {
    fn width(&self) -> i32;

    fn height(&self) -> i32;

    fn name(&self) -> &Identifier;
}

pub struct Stitcher<T: Entry> {
    mip_level: i32,
    textures_to_be_stitched: Vec<Holder>,
    entries: Vec<T>,
    storage: Vec<Region>,
    storage_x: i32,
    storage_y: i32,
    max_width: i32,
    max_height: i32,
    padding: i32,
}

/// `Stitcher.Holder`: an entry's index and its padded size.
#[derive(Clone, Copy, Debug)]
struct Holder {
    entry: usize,
    width: i32,
    height: i32,
}

/// `StitcherException`: the sprite that didn't fit, by name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StitcherException {
    pub sprite: Identifier,
}

impl<T: Entry> Stitcher<T> {
    pub fn new(
        max_width: i32,
        max_height: i32,
        mip_level: i32,
        anisotropy_bit: i32,
    ) -> Stitcher<T> {
        Stitcher {
            mip_level,
            textures_to_be_stitched: Vec::new(),
            entries: Vec::new(),
            storage: Vec::new(),
            storage_x: 0,
            storage_y: 0,
            max_width,
            max_height,
            padding: 1 << mip_level << Mth::clamp_i32(anisotropy_bit - 1, 0, 4),
        }
    }

    pub fn get_width(&self) -> i32 {
        self.storage_x
    }

    pub fn get_height(&self) -> i32 {
        self.storage_y
    }

    pub fn register_sprite(&mut self, entry: T) {
        let holder = Holder {
            entry: self.entries.len(),
            width: Stitcher::<T>::smallest_fitting_min_texel(
                entry.width() + self.padding * 2,
                self.mip_level,
            ),
            height: Stitcher::<T>::smallest_fitting_min_texel(
                entry.height() + self.padding * 2,
                self.mip_level,
            ),
        };
        self.entries.push(entry);
        self.textures_to_be_stitched.push(holder);
    }

    /// `HOLDER_COMPARATOR`.
    fn compare(&self, a: &Holder, b: &Holder) -> Ordering {
        (-a.height)
            .cmp(&-b.height)
            .then((-a.width).cmp(&-b.width))
            .then_with(|| {
                self.entries[a.entry]
                    .name()
                    .cmp(self.entries[b.entry].name())
            })
    }

    /// The `Err` is Java's `StitcherException`.
    pub fn stitch(&mut self) -> Result<(), StitcherException> {
        let mut holders = self.textures_to_be_stitched.clone();
        holders.sort_by(|a, b| self.compare(a, b));
        for holder in holders {
            if !self.add_to_storage(holder) {
                return Err(StitcherException {
                    sprite: self.entries[holder.entry].name().clone(),
                });
            }
        }
        Ok(())
    }

    /// Every placed entry with its position and the padding, in the regions' order, handing the
    /// entries over (Java's `SpriteLoader` callback).
    pub fn gather_sprites(self, mut loader: impl FnMut(T, i32, i32, i32)) {
        let mut placed = Vec::new();
        for top_region in &self.storage {
            top_region.walk(&mut placed);
        }
        let mut entries: Vec<Option<T>> = self.entries.into_iter().map(Some).collect();
        for (entry, x, y) in placed {
            let entry = entries[entry].take().expect("each entry is placed once");
            loader(entry, x, y, self.padding);
        }
    }

    fn smallest_fitting_min_texel(input: i32, max_mip_level: i32) -> i32 {
        ((input >> max_mip_level)
            + if input & ((1 << max_mip_level) - 1) == 0 {
                0
            } else {
                1
            })
            << max_mip_level
    }

    fn add_to_storage(&mut self, holder: Holder) -> bool {
        for region in &mut self.storage {
            if region.add(holder) {
                return true;
            }
        }
        self.expand(holder)
    }

    fn expand(&mut self, holder: Holder) -> bool {
        let x_current_size = Mth::smallest_encompassing_power_of_two(self.storage_x);
        let y_current_size = Mth::smallest_encompassing_power_of_two(self.storage_y);
        let x_new_size = Mth::smallest_encompassing_power_of_two(self.storage_x + holder.width);
        let y_new_size = Mth::smallest_encompassing_power_of_two(self.storage_y + holder.height);
        let x_can_grow = x_new_size <= self.max_width;
        let y_can_grow = y_new_size <= self.max_height;
        if !x_can_grow && !y_can_grow {
            return false;
        }

        let x_will_grow = x_can_grow && x_current_size != x_new_size;
        let y_will_grow = y_can_grow && y_current_size != y_new_size;
        let grow_on_x = if x_will_grow ^ y_will_grow {
            x_will_grow
        } else {
            x_can_grow && x_current_size <= y_current_size
        };

        let mut slot = if grow_on_x {
            if self.storage_y == 0 {
                self.storage_y = y_new_size;
            }
            let slot = Region::new(
                self.storage_x,
                0,
                x_new_size - self.storage_x,
                self.storage_y,
            );
            self.storage_x = x_new_size;
            slot
        } else {
            let slot = Region::new(
                0,
                self.storage_y,
                self.storage_x,
                y_new_size - self.storage_y,
            );
            self.storage_y = y_new_size;
            slot
        };
        slot.add(holder);
        self.storage.push(slot);
        true
    }
}

/// `Stitcher.Region`.
#[derive(Debug)]
struct Region {
    origin_x: i32,
    origin_y: i32,
    width: i32,
    height: i32,
    sub_slots: Option<Vec<Region>>,
    holder: Option<Holder>,
}

impl Region {
    fn new(origin_x: i32, origin_y: i32, width: i32, height: i32) -> Region {
        Region {
            origin_x,
            origin_y,
            width,
            height,
            sub_slots: None,
            holder: None,
        }
    }

    fn add(&mut self, holder: Holder) -> bool {
        if self.holder.is_some() {
            return false;
        }
        let texture_width = holder.width;
        let texture_height = holder.height;
        if texture_width > self.width || texture_height > self.height {
            return false;
        }
        if texture_width == self.width && texture_height == self.height {
            self.holder = Some(holder);
            return true;
        }

        let (origin_x, origin_y, width, height) =
            (self.origin_x, self.origin_y, self.width, self.height);
        let sub_slots = self.sub_slots.get_or_insert_with(|| {
            let mut sub_slots = vec![Region::new(
                origin_x,
                origin_y,
                texture_width,
                texture_height,
            )];
            let spare_width = width - texture_width;
            let spare_height = height - texture_height;
            if spare_height > 0 && spare_width > 0 {
                let right = height.max(spare_width);
                let bottom = width.max(spare_height);
                if right >= bottom {
                    sub_slots.push(Region::new(
                        origin_x,
                        origin_y + texture_height,
                        texture_width,
                        spare_height,
                    ));
                    sub_slots.push(Region::new(
                        origin_x + texture_width,
                        origin_y,
                        spare_width,
                        height,
                    ));
                } else {
                    sub_slots.push(Region::new(
                        origin_x + texture_width,
                        origin_y,
                        spare_width,
                        texture_height,
                    ));
                    sub_slots.push(Region::new(
                        origin_x,
                        origin_y + texture_height,
                        width,
                        spare_height,
                    ));
                }
            } else if spare_width == 0 {
                sub_slots.push(Region::new(
                    origin_x,
                    origin_y + texture_height,
                    texture_width,
                    spare_height,
                ));
            } else if spare_height == 0 {
                sub_slots.push(Region::new(
                    origin_x + texture_width,
                    origin_y,
                    spare_width,
                    texture_height,
                ));
            }
            sub_slots
        });
        sub_slots.iter_mut().any(|sub_slot| sub_slot.add(holder))
    }

    fn walk(&self, output: &mut Vec<(usize, i32, i32)>) {
        if let Some(holder) = &self.holder {
            output.push((holder.entry, self.origin_x, self.origin_y));
        } else if let Some(sub_slots) = &self.sub_slots {
            for sub_slot in sub_slots {
                sub_slot.walk(output);
            }
        }
    }
}
