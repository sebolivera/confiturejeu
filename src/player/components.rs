use bevy::prelude::Component;

/// Player run speed.
#[derive(Component, Clone, Copy, Debug)]
pub struct RunSpeed {
    /// Multiplier for player run speed.
    pub multiplier: f32,
}

impl Default for RunSpeed {
    fn default() -> Self {
        Self { multiplier: 1.5 }
    }
}
