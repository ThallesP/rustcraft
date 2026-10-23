//! net/minecraft/client/renderer/state/level/CameraEntityRenderState.java
//!
//! What the frame reads of the entity the camera is: its hurt and death, which tilt the view
//! (`bobHurt`), and whether it sleeps. The walk's bobbing (`bob`,
//! `backwardsInterpolatedWalkDistance`), which the oracle turns off, and the effects that block
//! the sky come with what reads them.

#[derive(Clone, Debug, Default)]
pub struct CameraEntityRenderState {
    pub hurt_time: f32,
    pub hurt_duration: i32,
    pub death_time: f32,
    pub is_sleeping: bool,
    pub is_living: bool,
    pub is_player: bool,
    pub is_dead_or_dying: bool,
    pub hurt_dir: f32,
}
