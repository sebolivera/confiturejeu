//! Handles player movement.

use crate::player::Player;
use crate::player::components::RunSpeed;
use bevy::prelude::{ButtonInput, KeyCode, Quat, Res, Single, Time, Transform, Vec2, Vec3, With};

/// Base speed for player movement, in units per second.
const BASE_SPEED: f32 = 10.0;

/// The player's movement intent for one frame, decoupled from any input device.
#[derive(Debug, Default, Clone, Copy)]
pub struct MoveIntent {
    /// 2d direction of the player
    pub axis: Vec2,
    /// Sprint (applies the run speed multiplier).
    pub sprint: bool,
}

impl MoveIntent {
    /// Reads the movement intent from the current keyboard state.
    fn from_keyboard(keys: &ButtonInput<KeyCode>) -> Self {
        let mut axis = Vec2::ZERO;

        if keys.pressed(KeyCode::KeyW) {
            axis.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            axis.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            axis.x += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            axis.x -= 1.0;
        }

        Self {
            axis: axis.normalize_or_zero(),
            sprint: keys.pressed(KeyCode::ShiftLeft),
        }
    }
}

/// Computes the translation to apply for one frame of movement.
///
/// Movement is constrained to the horizontal plane: the facing direction is
/// flattened before use, so looking up or down never changes movement speed.
/// Diagonal input is normalized so it is no faster than cardinal input.
#[must_use]
pub fn movement_delta(
    rotation: Quat,
    intent: MoveIntent,
    sprint_multiplier: f32,
    delta_secs: f32,
) -> Vec3 {
    if intent.axis == Vec2::ZERO {
        return Vec3::ZERO;
    }

    let forward = (rotation * Vec3::NEG_Z).with_y(0.0).normalize_or_zero();
    let right = (rotation * Vec3::X).with_y(0.0).normalize_or_zero();
    // Clamp rather than normalize: guards against over-unit input while
    // preserving sub-unit magnitudes (e.g. a half-tilted analog stick).
    let mut direction = (forward * intent.axis.y) + (right * intent.axis.x);
    direction = direction.clamp_length_max(1.0);

    if intent.sprint {
        direction *= sprint_multiplier;
    }
    direction * BASE_SPEED * delta_secs
}

/// Handles player movement through keyboard input.
pub(crate) fn move_player_from_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut player: Single<(&mut Transform, &RunSpeed), With<Player>>,
    time: Res<Time>,
) {
    if keys.just_pressed(KeyCode::Space) {
        // We'll see about jumps later.
    }
    if keys.just_released(KeyCode::ControlLeft) {
        // Probably crouch?
    }
    let intent = MoveIntent::from_keyboard(&keys);
    let rotation = player.0.rotation;
    let multiplier = player.1.multiplier;
    player.0.translation += movement_delta(rotation, intent, multiplier, time.delta_secs());
}

#[cfg(test)]
mod tests {
    // Honestly, none of these is really needed at this point, though it looks pretty when I run
    // `cargo test`...
    use super::*;
    use std::f32::consts::FRAC_PI_4;

    const DT: f32 = 1.0;
    const SPRINT: f32 = 1.5;
    const EPSILON: f32 = 1e-5;

    fn intent(axis: Vec2) -> MoveIntent {
        MoveIntent {
            axis,
            sprint: false,
        }
    }

    #[test]
    fn opposing_keys_cancel_out() {
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyS);
        keys.press(KeyCode::KeyA);
        keys.press(KeyCode::KeyD);
        assert_eq!(MoveIntent::from_keyboard(&keys).axis, Vec2::ZERO);
    }

    #[test]
    fn keyboard_diagonal_axis_is_normalized() {
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyD);
        let axis = MoveIntent::from_keyboard(&keys).axis;
        assert!((axis.length() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn no_input_no_movement() {
        let delta = movement_delta(Quat::IDENTITY, MoveIntent::default(), SPRINT, DT);
        assert_eq!(delta, Vec3::ZERO);
    }

    #[test]
    fn forward_moves_at_base_speed() {
        let delta = movement_delta(Quat::IDENTITY, intent(Vec2::Y), SPRINT, DT);
        assert!((delta - Vec3::NEG_Z * BASE_SPEED).length() < EPSILON);
    }

    #[test]
    fn diagonal_speed_equals_cardinal_speed() {
        let cardinal = movement_delta(Quat::IDENTITY, intent(Vec2::Y), SPRINT, DT);
        let diagonal = movement_delta(Quat::IDENTITY, intent(Vec2::new(1.0, 1.0)), SPRINT, DT);
        assert!((diagonal.length() - cardinal.length()).abs() < EPSILON);
    }

    #[test]
    fn sprint_scales_speed_by_multiplier() {
        let walk = movement_delta(Quat::IDENTITY, intent(Vec2::Y), SPRINT, DT);
        let sprint = movement_delta(
            Quat::IDENTITY,
            MoveIntent {
                sprint: true,
                ..intent(Vec2::Y)
            },
            SPRINT,
            DT,
        );
        assert!(walk.length().mul_add(-SPRINT, sprint.length()).abs() < EPSILON);
    }

    #[test]
    fn movement_follows_yaw() {
        let rotation = Quat::from_rotation_y(FRAC_PI_4);
        let delta = movement_delta(rotation, intent(Vec2::Y), SPRINT, DT);
        let expected = Vec3::new(-1.0, 0.0, -1.0).normalize() * BASE_SPEED;
        assert!((delta - expected).length() < EPSILON);
    }

    #[test]
    fn pitch_does_not_slow_movement() {
        // Looking down at 45 degrees must not reduce horizontal speed.
        let rotation = Quat::from_rotation_x(-FRAC_PI_4);
        let delta = movement_delta(rotation, intent(Vec2::Y), SPRINT, DT);
        assert!((delta.length() - BASE_SPEED).abs() < EPSILON);
        assert!(delta.y.abs() < EPSILON);
    }

    #[test]
    fn movement_scales_with_delta_time() {
        let full = movement_delta(Quat::IDENTITY, intent(Vec2::Y), SPRINT, 1.0);
        let half = movement_delta(Quat::IDENTITY, intent(Vec2::Y), SPRINT, 0.5);
        assert!((half.length() - full.length() / 2.0).abs() < EPSILON);
    }
}
