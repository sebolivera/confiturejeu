use crate::input_utils;
use bevy::prelude::{App, Component, Plugin, Update};

/// Player resources.
pub mod components;
/// Camera look.
pub mod look;
/// Movement
pub mod movement;

/// Player representation.
#[derive(Debug, Component, Default, Copy, Clone)]
#[require(components::RunSpeed)]
pub struct Player;

/// Player plugin.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                movement::move_player_from_keyboard,
                look::mouse_look,
                input_utils::cursor::toggle_cursor,
            ),
        );
    }
}
