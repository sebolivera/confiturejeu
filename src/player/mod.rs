use crate::input_utils;
use crate::state::GameState;
use avian3d::prelude::LinearVelocity;
use bevy::prelude::{
    App, Component, IntoScheduleConfigs, OnExit, Plugin, Single, Update, With, in_state,
};

/// Player resources.
pub mod components;
/// Camera look.
pub mod look;
/// Movement
pub mod movement;

/// Player representation.
#[derive(Debug, Component, Default, Copy, Clone)]
#[require(components::PlayerDynamics, components::PlayerDimensions)]
pub struct Player;

/// Player plugin.
pub struct PlayerPlugin;

/// Clears horizontal velocity so the player doesn't keep moving after a pause.
fn halt_player(mut linear_velocity: Single<&mut LinearVelocity, With<Player>>) {
    linear_velocity.x = 0.0;
    linear_velocity.z = 0.0;
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                movement::move_player_from_keyboard,
                movement::apply_step_up,
                movement::smooth_step_camera,
            )
                .chain()
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(
            Update,
            (look::mouse_look, input_utils::cursor::toggle_cursor)
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(OnExit(GameState::Game), halt_player);
    }
}
