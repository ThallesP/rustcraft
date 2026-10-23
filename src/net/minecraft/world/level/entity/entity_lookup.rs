//! net/minecraft/world/level/entity/EntityLookup.java
//!
//! A level's entities by id, in the order they were added (Java's `Int2ObjectLinkedOpenHashMap`),
//! and by UUID: one with a UUID it already has isn't added, with a warning, like Java's.

use std::collections::HashMap;

use crate::java::util::uuid::UUID;

/// What the lookup asks an entity (`EntityAccess`).
pub trait EntityAccess {
    fn get_id(&self) -> i32;

    fn get_uuid(&self) -> UUID;
}

/// The entities by id, in order, and by UUID.
#[derive(Debug)]
pub struct EntityLookup<T> {
    by_id: Vec<(i32, T)>,
    by_uuid: HashMap<UUID, i32>,
}

impl<T> Default for EntityLookup<T> {
    fn default() -> EntityLookup<T> {
        EntityLookup {
            by_id: Vec::new(),
            by_uuid: HashMap::new(),
        }
    }
}

impl<T: EntityAccess> EntityLookup<T> {
    pub fn new() -> EntityLookup<T> {
        EntityLookup::default()
    }

    /// `getAllEntities()`, in the order they were added.
    pub fn get_all_entities(&self) -> impl Iterator<Item = &T> {
        self.by_id.iter().map(|(_, entity)| entity)
    }

    pub fn add(&mut self, entity: T) {
        let uuid = entity.get_uuid();
        if self.by_uuid.contains_key(&uuid) {
            eprintln!("Duplicate entity UUID {uuid}: entity {}", entity.get_id());
            return;
        }
        let id = entity.get_id();
        self.by_uuid.insert(uuid, id);
        match self.by_id.iter_mut().find(|(known, _)| *known == id) {
            // `put` of a key the map has keeps its place.
            Some(slot) => slot.1 = entity,
            None => self.by_id.push((id, entity)),
        }
    }

    /// `remove(T)`, by the entity's id and UUID.
    pub fn remove(&mut self, id: i32, uuid: UUID) {
        self.by_uuid.remove(&uuid);
        self.by_id.retain(|(known, _)| *known != id);
    }

    pub fn get_entity(&self, id: i32) -> Option<&T> {
        self.by_id
            .iter()
            .find(|(known, _)| *known == id)
            .map(|(_, entity)| entity)
    }

    pub fn get_entity_mut(&mut self, id: i32) -> Option<&mut T> {
        self.by_id
            .iter_mut()
            .find(|(known, _)| *known == id)
            .map(|(_, entity)| entity)
    }

    /// The entity taken out for a while, with its place, which `put_back` gives it again: Java's
    /// entity is in its level while its code runs, which Rust's borrows don't let it be.
    pub fn take(&mut self, id: i32) -> Option<(usize, T)> {
        let index = self.by_id.iter().position(|(known, _)| *known == id)?;
        Some((index, self.by_id.remove(index).1))
    }

    pub fn put_back(&mut self, index: usize, entity: T) {
        let id = entity.get_id();
        self.by_id.insert(index, (id, entity));
    }

    pub fn count(&self) -> usize {
        self.by_uuid.len()
    }
}
