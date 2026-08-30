use crate::settings::{DisplayQuality, Volume};
use crate::state::GameState;
use crate::ui::despawn_screen;
use crate::ui::menu::display::{OnDisplaySettingsMenuScreen, display_settings_menu_setup};
use crate::ui::menu::interaction::{button_system, setting_button};
use crate::ui::menu::main::{OnMainMenuScreen, main_menu_setup, menu_setup};
use crate::ui::menu::settings::{OnSettingsMenuScreen, settings_menu_setup};
use crate::ui::menu::sound::{OnSoundSettingsMenuScreen, sound_settings_menu_setup};
use bevy::app::App;
use bevy::prelude::{
    AppExit, AppExtStates, Button, ButtonInput, Changed, Component, Interaction,
    IntoScheduleConfigs, KeyCode, MessageWriter, NextState, OnEnter, OnExit, Query, Res, ResMut,
    Single, State, States, Update, With, in_state,
};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

/// Display settings menu.
pub mod display;
/// Button interaction systems.
pub mod interaction;
/// Main menu.
pub mod main;
/// Settings menu.
pub mod settings;
/// Sound settings menu.
pub mod sound;

/// Possible menu states.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum MenuState {
    /// Main menu.
    Main,
    /// Settings menu.
    Settings,
    /// Display settings.
    SettingsDisplay,
    /// Sound settings.
    SettingsSound,
    #[default]
    /// No menu.
    Disabled,
}

/// All actions that can be triggered from a button click
#[derive(Component)]
pub enum MenuButtonAction {
    /// Play the game
    Play,
    /// Open the settings menu
    Settings,
    /// Open the display settings menu
    SettingsDisplay,
    /// Open the sound settings menu
    SettingsSound,
    /// Return to the main menu
    BackToMainMenu,
    /// Return to the settings menu
    BackToSettings,
    /// Quit the game
    Quit,
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_writer: MessageWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_writer.write(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    game_state.set(GameState::Game);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                MenuButtonAction::SettingsDisplay => {
                    menu_state.set(MenuState::SettingsDisplay);
                }
                MenuButtonAction::SettingsSound => {
                    menu_state.set(MenuState::SettingsSound);
                }
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                MenuButtonAction::BackToSettings => {
                    menu_state.set(MenuState::Settings);
                }
            }
        }
    }
}

fn toggle_menu_on_escape(
    input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !input.just_pressed(KeyCode::Escape) {
        return;
    }
    match current_state.get() {
        GameState::Game => {
            game_state.set(GameState::Menu);
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
        GameState::Menu => {
            game_state.set(GameState::Game);
            menu_state.set(MenuState::Disabled);
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
        GameState::Splash => {}
    }
}

/// Menu plugin.
pub fn menu_plugin(app: &mut App) {
    app.init_state::<MenuState>()
        .add_systems(OnEnter(GameState::Menu), menu_setup)
        .add_systems(OnEnter(MenuState::Main), main_menu_setup)
        .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
        .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
        .add_systems(
            OnExit(MenuState::Settings),
            despawn_screen::<OnSettingsMenuScreen>,
        )
        .add_systems(
            OnEnter(MenuState::SettingsDisplay),
            display_settings_menu_setup,
        )
        .add_systems(
            OnExit(MenuState::SettingsDisplay),
            despawn_screen::<OnDisplaySettingsMenuScreen>,
        )
        .add_systems(
            Update,
            (setting_button::<DisplayQuality>.run_if(in_state(MenuState::SettingsDisplay)),),
        )
        .add_systems(OnEnter(MenuState::SettingsSound), sound_settings_menu_setup)
        .add_systems(
            OnExit(MenuState::SettingsSound),
            despawn_screen::<OnSoundSettingsMenuScreen>,
        )
        .add_systems(
            Update,
            setting_button::<Volume>.run_if(in_state(MenuState::SettingsSound)),
        )
        .add_systems(
            Update,
            (menu_action, button_system).run_if(in_state(GameState::Menu)),
        )
        .add_systems(Update, toggle_menu_on_escape);
}
