//! net/minecraft/world/entity/MoverType.java

/// What moves an entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MoverType {
    SELF,
    PLAYER,
    PISTON,
    SHULKER_BOX,
    SHULKER,
}

impl MoverType {
    /// Whether both the server and the client move entities this way: all but the entity itself.
    pub fn is_server_and_client_simulated(&self) -> bool {
        *self != MoverType::SELF
    }
}
