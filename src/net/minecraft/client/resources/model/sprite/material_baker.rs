//! net/minecraft/client/resources/model/sprite/MaterialBaker.java
//!
//! Models' materials found in the item atlas, else the block atlas, cached by material like Java's
//! `ConcurrentHashMap` (a material neither atlas has isn't cached, like `computeIfAbsent`'s null).
//! The bake runs on one thread here, so the caches and the missing-texture lists are cells.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::net::minecraft::client::renderer::texture::missing_texture_atlas_sprite::MissingTextureAtlasSprite;
use crate::net::minecraft::client::renderer::texture::sprite_loader::Preparations;
use crate::net::minecraft::client::resources::model::model_debug_name::ModelDebugName;
use crate::net::minecraft::client::resources::model::sprite::material::{Baked, Material};
use crate::net::minecraft::client::resources::model::sprite::texture_slots::TextureSlots;
use crate::net::minecraft::resources::identifier::Identifier;

pub struct MaterialBaker<'a> {
    block_atlas: &'a Preparations,
    item_atlas: &'a Preparations,
    missing_sprite: Baked,
    missing_sprite_force_translucent: Baked,
    // Java's `HashMultimap`s, only ever logged sorted.
    missing_sprites: RefCell<BTreeMap<String, BTreeSet<Identifier>>>,
    missing_references: RefCell<BTreeMap<String, BTreeSet<String>>>,
    baked_materials: RefCell<HashMap<Material, Baked>>,
}

impl<'a> MaterialBaker<'a> {
    /// Java's `blockAtlas.missing()` is null for an atlas without the missing sprite, which no
    /// atlas is; this panics there.
    pub fn new(block_atlas: &'a Preparations, item_atlas: &'a Preparations) -> MaterialBaker<'a> {
        let missing = block_atlas
            .missing()
            .expect("the block atlas has the missing sprite")
            .clone();
        MaterialBaker {
            block_atlas,
            item_atlas,
            missing_sprite: Baked {
                sprite: missing.clone(),
                force_translucent: false,
            },
            missing_sprite_force_translucent: Baked {
                sprite: missing,
                force_translucent: true,
            },
            missing_sprites: RefCell::default(),
            missing_references: RefCell::default(),
            baked_materials: RefCell::default(),
        }
    }

    fn replacement_for_missing_material(&self, material: &Material) -> Baked {
        if material.force_translucent {
            self.missing_sprite_force_translucent.clone()
        } else {
            self.missing_sprite.clone()
        }
    }

    pub fn get(&self, material: &Material, name: &dyn ModelDebugName) -> Baked {
        if material.sprite == MissingTextureAtlasSprite::get_location() {
            return self.replacement_for_missing_material(material);
        }

        if let Some(baked) = self.baked_materials.borrow().get(material) {
            return baked.clone();
        }
        match self.bake(material) {
            Some(baked) => {
                self.baked_materials
                    .borrow_mut()
                    .insert(material.clone(), baked.clone());
                baked
            }
            None => {
                self.missing_sprites
                    .borrow_mut()
                    .entry(name.debug_name())
                    .or_default()
                    .insert(material.sprite.clone());
                self.replacement_for_missing_material(material)
            }
        }
    }

    fn bake(&self, material: &Material) -> Option<Baked> {
        MaterialBaker::bake_for_atlas(material, self.item_atlas)
            .or_else(|| MaterialBaker::bake_for_atlas(material, self.block_atlas))
    }

    fn bake_for_atlas(material: &Material, atlas: &Preparations) -> Option<Baked> {
        atlas.get_sprite(&material.sprite).map(|sprite| Baked {
            sprite: sprite.clone(),
            force_translucent: material.force_translucent,
        })
    }

    pub fn resolve_slot(&self, slots: &TextureSlots, id: &str, name: &dyn ModelDebugName) -> Baked {
        match slots.get_material(id) {
            Some(resolved_material) => self.get(resolved_material, name),
            None => self.report_missing_reference(id, name),
        }
    }

    pub fn report_missing_reference(
        &self,
        reference: &str,
        responsible_model: &dyn ModelDebugName,
    ) -> Baked {
        self.missing_references
            .borrow_mut()
            .entry(responsible_model.debug_name())
            .or_default()
            .insert(reference.to_owned());
        self.missing_sprite.clone()
    }

    /// The warnings Java logs, a model at a time, its textures sorted. Java's multimaps are
    /// hash maps, so the models come in another order there.
    pub fn log_missing_textures(&self) {
        for (location, sprites) in self.missing_sprites.borrow().iter() {
            let sprites: Vec<String> = sprites
                .iter()
                .map(|sprite| format!("    {sprite}"))
                .collect();
            eprintln!(
                "Missing textures in model {location}:\n{}",
                sprites.join("\n")
            );
        }
        for (location, references) in self.missing_references.borrow().iter() {
            let references: Vec<String> = references
                .iter()
                .map(|reference| format!("    {reference}"))
                .collect();
            eprintln!(
                "Missing texture references in model {location}:\n{}",
                references.join("\n")
            );
        }
    }
}
