use crate::state::GameState;
use crate::ui::despawn_screen;
use crate::ui::splash::{OnSplashScreen, SplashTimer, splash_setup};
use bevy::prelude::{
    Alpha, App, Children, ImageNode, IntoScheduleConfigs, NextState, OnEnter, OnExit, Query, Res,
    ResMut, Single, Time, Update, With, in_state,
};

const FADE_SECONDS: f32 = 0.3;

/// This plugin will display a splash screen with Bevy logo for 1 second before switching to the menu
pub fn splash_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Splash), splash_setup)
        .add_systems(
            Update,
            (countdown, fade)
                .chain()
                .run_if(in_state(GameState::Splash)),
        )
        .add_systems(OnExit(GameState::Splash), despawn_screen::<OnSplashScreen>);
}

/// Tick the timer, and change state when finished
fn countdown(
    mut game_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if timer.tick(time.delta()).is_finished() {
        game_state.set(GameState::Menu);
    }
}

/// Fades the logo in and out over the splash duration
fn fade(
    timer: Res<SplashTimer>,
    root: Single<&Children, With<OnSplashScreen>>,
    mut images: Query<&mut ImageNode>,
) {
    let elapsed = timer.elapsed_secs();
    let remaining = timer.duration().as_secs_f32() - elapsed;
    let alpha = (elapsed / FADE_SECONDS)
        .min(remaining / FADE_SECONDS)
        .clamp(0.0, 1.0);
    for &child in *root {
        if let Ok(mut image) = images.get_mut(child) {
            image.color.set_alpha(alpha);
        }
    }
}
