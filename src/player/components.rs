use bevy::prelude::Component;

/// Marks the first-person camera carrying the player's look orientation.
#[derive(Debug, Component, Default, Copy, Clone)]
pub struct PlayerCamera;

/// Resting height of the camera plus the transient offset used to smooth steps.
#[derive(Component, Clone, Copy, Debug)]
pub struct CameraRig {
    /// Local height of the camera above the player origin.
    pub base_height: f32,
    /// Current visual offset from [`Self::base_height`]. Decays to zero.
    pub step_offset: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            base_height: 0.85,
            step_offset: 0.0,
        }
    }
}

/// Player run speed.
#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerDynamics {
    /// Multiplier for player run speed.
    pub multiplier: f32,
    /// Time since the player was last grounded.
    pub coyote_timer: f32,
    /// Signed vertical adjustment to apply: positive lifts the player onto a
    /// step, negative snaps them down onto the ground.
    pub pending_step: f32,
    /// Whether the player was standing on the ground last frame.
    pub grounded: bool,
    /// Jump press recorded at render rate, waiting for the next physics tick.
    pub jump_buffered: bool,
}

impl Default for PlayerDynamics {
    fn default() -> Self {
        Self {
            multiplier: 1.5,
            coyote_timer: 0.0,
            pending_step: 0.0,
            grounded: false,
            jump_buffered: false,
        }
    }
}

/// Height and width of the player's capsule
#[derive(Component, Clone, Copy, Debug)]
pub struct PlayerDimensions {
    /// Height of the player.
    pub height: f32,
}

impl Default for PlayerDimensions {
    fn default() -> Self {
        Self { height: 1.7 }
    }
}
