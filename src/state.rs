use bevy::prelude::*;

/// Game state.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    /// Splash screen
    Splash,
    /// Escape menu
    Menu,
    /// Game
    Game,
}
