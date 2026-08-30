//! Handles the player camera look.

use crate::player::components::PlayerCamera;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::{
    Component, Deref, DerefMut, EulerRot, Quat, Res, Single, Transform, Vec2, With,
};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use std::f32::consts::FRAC_PI_2;

/// Maximum pitch (up or down), just shy of straight vertical to avoid gimbal flip.
const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

/// Camera sensitivity.
#[derive(Debug, Component, Deref, DerefMut, Copy, Clone)]
pub struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.0003, 0.0002))
    }
}

/// Computes the new look rotation from a mouse motion delta.
///
/// Yaw is unbounded, pitch is clamped to [`PITCH_LIMIT`], and roll is always
/// forced to zero.
#[must_use]
pub fn look_rotation(current: Quat, mouse_delta: Vec2, sensitivity: Vec2) -> Quat {
    let delta_yaw = -mouse_delta.x * sensitivity.x;
    let delta_pitch = -mouse_delta.y * sensitivity.y;

    let (yaw, pitch, _roll) = current.to_euler(EulerRot::YXZ);
    let yaw = yaw + delta_yaw;
    let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

    Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0)
}

/// Moves the player's camera based on mouse input.
///
/// The rotation is applied to the camera, not to the player body: the body is a
/// dynamic rigid body whose `Transform` is owned by the physics solver.
pub fn mouse_look(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    camera: Single<(&mut Transform, &CameraSensitivity), With<PlayerCamera>>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
) {
    if cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }
    let (mut transform, sensitivity) = camera.into_inner();
    let delta = accumulated_mouse_motion.delta;

    if delta != Vec2::ZERO {
        transform.rotation = look_rotation(transform.rotation, delta, **sensitivity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENSITIVITY: Vec2 = Vec2::new(0.01, 0.01);
    const EPSILON: f32 = 1e-4;

    fn euler(rotation: Quat) -> (f32, f32, f32) {
        rotation.to_euler(EulerRot::YXZ)
    }

    #[test]
    fn no_motion_keeps_orientation() {
        let rotation = look_rotation(Quat::IDENTITY, Vec2::ZERO, SENSITIVITY);
        let (yaw, pitch, roll) = euler(rotation);
        assert!(yaw.abs() < EPSILON && pitch.abs() < EPSILON && roll.abs() < EPSILON);
    }

    #[test]
    fn moving_mouse_right_yaws_right() {
        let rotation = look_rotation(Quat::IDENTITY, Vec2::new(100.0, 0.0), SENSITIVITY);
        let (yaw, _, _) = euler(rotation);
        // Rightward mouse motion turns clockwise, i.e. negative yaw.
        assert!((yaw - (-1.0)).abs() < EPSILON);
    }

    #[test]
    fn moving_mouse_up_pitches_up() {
        let rotation = look_rotation(Quat::IDENTITY, Vec2::new(0.0, -50.0), SENSITIVITY);
        let (_, pitch, _) = euler(rotation);
        assert!((pitch - 0.5).abs() < EPSILON);
    }

    #[test]
    fn pitch_is_clamped() {
        let mut rotation = Quat::IDENTITY;
        // Sweep far past the vertical limit.
        for _ in 0..100 {
            rotation = look_rotation(rotation, Vec2::new(0.0, -1000.0), SENSITIVITY);
        }
        let (_, pitch, _) = euler(rotation);
        assert!(pitch <= PITCH_LIMIT + EPSILON);
        assert!(pitch > PITCH_LIMIT - 0.1);
    }

    #[test]
    fn roll_stays_zero() {
        let mut rotation = Quat::IDENTITY;
        for _ in 0..50 {
            rotation = look_rotation(rotation, Vec2::new(37.0, -23.0), SENSITIVITY);
        }
        let (_, _, roll) = euler(rotation);
        assert!(roll.abs() < EPSILON);
    }

    #[test]
    fn sensitivity_scales_rotation() {
        let slow = look_rotation(Quat::IDENTITY, Vec2::new(10.0, 0.0), Vec2::splat(0.001));
        let fast = look_rotation(Quat::IDENTITY, Vec2::new(10.0, 0.0), Vec2::splat(0.002));
        let (slow_yaw, _, _) = euler(slow);
        let (fast_yaw, _, _) = euler(fast);
        assert!(slow_yaw.mul_add(-2.0, fast_yaw).abs() < EPSILON);
    }
}
