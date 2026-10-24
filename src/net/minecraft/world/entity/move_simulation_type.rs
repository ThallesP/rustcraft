//! net/minecraft/world/entity/MoveSimulationType.java
//!
//! Which sides move an entity in their own ticks (`Entity.getMoveSimulationType`,
//! `canSimulateMovement`): both (items, falling blocks), the side that decides where it goes
//! (most entities: the server), or that side and the server (players).

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveSimulationType {
    SERVER_AND_CLIENT,
    AUTHORITATIVE_SIDE,
    AUTHORITATIVE_SIDE_AND_SERVER,
}
