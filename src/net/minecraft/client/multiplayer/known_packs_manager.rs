//! net/minecraft/client/multiplayer/KnownPacksManager.java
//!
//! Not ported yet: the pack repository it selects from (`ServerPacksSource`'s vanilla trusted
//! repository) and `createResourceManager`, which come with resource loading. The packs that
//! repository knows are the game's own: the `vanilla` pack, which is `KnownPack.vanilla("core")`,
//! and each feature pack under `data/minecraft/datapacks/` (`minecart_improvements`,
//! `redstone_experiments` and `trade_rebalance` in 26.3), which is `KnownPack.vanilla` of its id.

use crate::net::minecraft::server::packs::repository::known_pack::KnownPack;

/// The packs the client already has, which the server can leave out of the configuration.
#[derive(Debug)]
pub struct KnownPacksManager {
    /// Each known pack, with its id in the repository.
    known_pack_to_id: Vec<(KnownPack, String)>,
    selected: Vec<String>,
}

impl Default for KnownPacksManager {
    fn default() -> KnownPacksManager {
        KnownPacksManager::new()
    }
}

impl KnownPacksManager {
    /// The feature packs of the game's version, by id.
    const FEATURE_PACKS: [&'static str; 3] = [
        "minecart_improvements",
        "redstone_experiments",
        "trade_rebalance",
    ];

    pub fn new() -> KnownPacksManager {
        let mut known_pack_to_id = vec![(KnownPack::vanilla("core"), "vanilla".to_string())];
        for id in KnownPacksManager::FEATURE_PACKS {
            known_pack_to_id.push((KnownPack::vanilla(id), id.to_string()));
        }

        KnownPacksManager {
            known_pack_to_id,
            selected: Vec::new(),
        }
    }

    /// The packs the server offered that the client has, in the server's order, which the client
    /// selects.
    pub fn try_selecting_packs(&mut self, packs_to_select: &[KnownPack]) -> Vec<KnownPack> {
        let mut response = Vec::with_capacity(packs_to_select.len());
        let mut selected_packs = Vec::with_capacity(packs_to_select.len());

        for known_pack in packs_to_select {
            let known_pack_id = self
                .known_pack_to_id
                .iter()
                .find(|(pack, _)| pack == known_pack)
                .map(|(_, id)| id.clone());
            if let Some(known_pack_id) = known_pack_id {
                selected_packs.push(known_pack_id);
                response.push(known_pack.clone());
            }
        }

        self.selected = selected_packs;
        response
    }

    /// The repository's selected packs' ids.
    pub fn selected(&self) -> &[String] {
        &self.selected
    }
}
