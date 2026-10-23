//! net/minecraft/world/level/entity/EntityTickList.java
//!
//! The entities a level ticks, by id, in the order they were added (Java's
//! `Int2ObjectLinkedOpenHashMap`: adding an id it has keeps its place). Java iterates its map
//! while ticking and copies it when one is added or removed meanwhile, so an iteration sees the
//! entities as they were when it began: here `for_each` iterates a copy of the ids.

/// The ticking entities' ids, in order.
#[derive(Clone, Debug, Default)]
pub struct EntityTickList {
    active: Vec<i32>,
}

impl EntityTickList {
    pub fn new() -> EntityTickList {
        EntityTickList::default()
    }

    pub fn add(&mut self, id: i32) {
        if !self.contains(id) {
            self.active.push(id);
        }
    }

    pub fn remove(&mut self, id: i32) {
        self.active.retain(|active| *active != id);
    }

    pub fn contains(&self, id: i32) -> bool {
        self.active.contains(&id)
    }

    /// The ids as they are now, which an iteration goes through whatever it adds or removes.
    pub fn for_each(&self) -> Vec<i32> {
        self.active.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::EntityTickList;

    #[test]
    fn keeps_the_order_entities_came_in() {
        let mut list = EntityTickList::new();
        list.add(5);
        list.add(2);
        list.add(9);
        list.add(5);
        assert_eq!(list.for_each(), [5, 2, 9]);
        let ticking = list.for_each();
        list.remove(2);
        list.add(7);
        assert_eq!(ticking, [5, 2, 9]);
        assert_eq!(list.for_each(), [5, 9, 7]);
        assert!(list.contains(7) && !list.contains(2));
    }
}
