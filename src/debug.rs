use crate::player::Player;
use bevy::prelude::*;

/// Wrapper for player stats to display in the UI.
#[derive(Component)]
pub struct PlayerStatsText;

/// Sets up the UI for debug display info.
pub fn setup_stats_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("Loading stats..."),
        PlayerStatsText,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            ..default()
        },
    ));
}

/// Display the stats in the UI.
/// Currently displays:
/// * Player transform
pub fn update_stats_ui(
    player: Single<&Transform, With<Player>>,
    mut text: Single<&mut Text, With<PlayerStatsText>>,
    mut last_pos: Local<Vec3>,
    time: Res<Time>,
) {
    let pos = player.translation;

    let distance_moved = pos - *last_pos;

    let velocity = distance_moved / time.delta_secs().max(0.0001);

    *last_pos = pos;

    **text = format!(
        "Position\nX: {:>7.2}\nY: {:>7.2}\nZ: {:>7.2}\n\nSpeed: {:>7.2} units/sec",
        pos.x,
        pos.y,
        pos.z,
        velocity.length()
    )
    .into();
}
