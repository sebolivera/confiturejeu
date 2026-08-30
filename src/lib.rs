//! First-person prototype: a player walking around a flat test scene.

/// Debug utilities.
pub mod debug;
pub mod input_utils;
/// Player.
pub mod player;
/// Game settings.
pub mod settings;
/// Game state.
pub mod state;
/// User Interface.
pub mod ui;
/// World data.
pub mod world;

use crate::player::PlayerPlugin;
use crate::settings::{DisplayQuality, Volume};
use crate::state::GameState;
use crate::world::{GRAVITY, scene, spawn_obstacles};
use avian3d::PhysicsPlugins;
use avian3d::prelude::Gravity;
use bevy::prelude::*;

/// Builds the app and runs the game until the window is closed.
pub fn run() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PlayerPlugin,
            PhysicsPlugins::default(),
            ui::splash::plugin::splash_plugin,
            ui::menu::menu_plugin,
        ))
        .init_state::<GameState>()
        .insert_resource(DisplayQuality::Medium)
        .insert_resource(Volume(7))
        .insert_resource(Gravity(GRAVITY))
        .add_systems(
            Startup,
            (scene.spawn(), spawn_obstacles, debug::setup_stats_ui),
        )
        .add_systems(
            Update,
            debug::update_stats_ui.after(player::movement::move_player_from_keyboard),
        )
        .run();
}
