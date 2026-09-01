//! Handles player movement.

use crate::player::Player;
use crate::player::components::{CameraRig, PlayerCamera, PlayerDimensions, PlayerDynamics};
use avian3d::prelude::{
    Collider, LinearVelocity, Position, RayHitData, ShapeCastConfig, ShapeHitData, SpatialQuery,
    SpatialQueryFilter,
};
use bevy::prelude::{
    ButtonInput, Dir3, Entity, KeyCode, Quat, Res, Single, Time, Transform, Vec2, Vec3, With,
};

/// Base speed for player movement, in units per second.
const BASE_SPEED: f32 = 10.0;
/// Speed of jump.
const JUMP_SPEED: f32 = 8.0;
/// Extra ray length below the capsule used to detect the floor just beneath us.
const GROUND_PROBE: f32 = 0.1;
/// Maximum time the player can jump after leaving the ground.
const COYOTE_TIME: f32 = 0.1;
/// Tallest ledge the player can walk onto without jumping.
pub(crate) const MAX_STEP_HEIGHT: f32 = 0.4;
/// How far ahead of the capsule axis the step probes reach.
const STEP_PROBE_DISTANCE: f32 = 0.5;
/// Small offset that keeps probes from starting exactly on a surface.
const STEP_SKIN: f32 = 0.02;
/// Smallest upward normal component still considered walkable ground.
pub(crate) const MIN_WALKABLE_NORMAL_Y: f32 = 0.7;
/// How fast the camera catches up after the body is lifted onto a step.
const STEP_SMOOTH_SPEED: f32 = 8.0;
/// Highest upward speed still treated as resting on the ground. Contact
/// resolution can leave a tiny upward velocity, which must not break the
/// ground snap; a real jump is far above this.
const MAX_GROUNDED_RISE_SPEED: f32 = 0.5;

/// The player's movement intent for one frame, decoupled from any input device.
#[derive(Debug, Default, Clone, Copy)]
pub struct MoveIntent {
    /// 2d direction of the player
    pub axis: Vec2,
    /// Sprint (applies the run speed multiplier).
    pub sprint: bool,
    /// Jump
    pub jump: bool,
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
            jump: keys.just_pressed(KeyCode::Space),
        }
    }
}

/// Computes the translation to apply for one frame of movement.
#[must_use]
pub fn movement_velocity(rotation: Quat, intent: MoveIntent, sprint_multiplier: f32) -> Vec3 {
    if intent.axis == Vec2::ZERO {
        return Vec3::ZERO;
    }

    let forward = (rotation * Vec3::NEG_Z).with_y(0.0).normalize_or_zero();
    let right = (rotation * Vec3::X).with_y(0.0).normalize_or_zero();

    let mut direction = (forward * intent.axis.y) + (right * intent.axis.x);
    direction = direction.clamp_length_max(1.0);

    if intent.sprint {
        direction *= sprint_multiplier;
    }
    direction * BASE_SPEED
}

/// Handles player movement through keyboard input.
pub(crate) fn move_player_from_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player: Single<
        (
            Entity,
            &Position,
            &Collider,
            &mut LinearVelocity,
            &mut PlayerDynamics,
            &PlayerDimensions,
        ),
        With<Player>,
    >,
    camera: Single<&Transform, With<PlayerCamera>>,
    spatial_query: SpatialQuery,
) {
    let (entity, position, collider, mut linear_velocity, mut dynamics, dimensions) =
        player.into_inner();
    let mut intent = MoveIntent::from_keyboard(&keys);
    intent.jump |= core::mem::take(&mut dynamics.jump_buffered);
    let target_velocity = movement_velocity(camera.rotation, intent, dynamics.multiplier);
    let origin = position.0;

    linear_velocity.x = target_velocity.x;
    linear_velocity.z = target_velocity.z;

    let contact = ground_hit(&spatial_query, entity, origin, *dimensions, GROUND_PROBE);
    let snap = if contact.is_none()
        && dynamics.grounded
        && !intent.jump
        && linear_velocity.y < MAX_GROUNDED_RISE_SPEED
    {
        snap_to_ground(&spatial_query, entity, collider, origin)
    } else {
        None
    };
    let grounded = contact.is_some() || snap.is_some();

    if grounded && linear_velocity.y < MAX_GROUNDED_RISE_SPEED {
        dynamics.coyote_timer = 0.0;
    } else {
        dynamics.coyote_timer += time.delta_secs();
    }

    if let Some(hit) = contact {
        linear_velocity.y = linear_velocity
            .y
            .min(max_ground_rise(hit.normal, target_velocity.length()));
    }

    if grounded {
        let rise = step_rise(&spatial_query, entity, origin, *dimensions, target_velocity);

        dynamics.pending_step = match (rise, snap) {
            (Some(rise), _) => rise,
            (None, Some(hit)) => {
                linear_velocity.y = 0.0;
                -hit.distance
            }
            (None, None) => 0.0,
        };
    }

    dynamics.grounded = grounded;

    if intent.jump && dynamics.coyote_timer < COYOTE_TIME {
        linear_velocity.y = JUMP_SPEED;
        dynamics.coyote_timer = COYOTE_TIME;
    }
}

/// Records jump presses at render rate so the fixed-timestep movement system
/// never misses or duplicates one.
pub(crate) fn buffer_jump(
    keys: Res<ButtonInput<KeyCode>>,
    mut dynamics: Single<&mut PlayerDynamics, With<Player>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        dynamics.jump_buffered = true;
    }
}

/// Applies the pending vertical adjustment (step-up lift or ground snap) and
/// makes the camera trail the jump smoothly.
pub(crate) fn apply_step_up(
    player: Single<(&mut Position, &mut PlayerDynamics), With<Player>>,
    rig: Single<&mut CameraRig, With<PlayerCamera>>,
) {
    let (mut position, mut dynamics) = player.into_inner();
    let rise = core::mem::take(&mut dynamics.pending_step);

    if rise != 0.0 {
        position.y += rise;
        rig.into_inner().step_offset -= rise;
    }
}

/// Eases the camera back to its resting height after a step-up.
pub(crate) fn smooth_step_camera(
    time: Res<Time>,
    camera: Single<(&mut Transform, &mut CameraRig), With<PlayerCamera>>,
) {
    let (mut transform, mut rig) = camera.into_inner();

    if rig.step_offset != 0.0 {
        rig.step_offset = decayed_offset(rig.step_offset, time.delta_secs());
    }

    transform.translation.y = rig.base_height + rig.step_offset;
}

/// Fastest ground contact may lift the player, given its normal and their speed.
#[must_use]
pub fn max_ground_rise(ground_normal: Vec3, speed: f32) -> f32 {
    let slope_tangent = ground_normal.with_y(0.0).length() / ground_normal.y.max(f32::EPSILON);
    speed * slope_tangent
}

/// Moves a camera step offset one frame's worth closer to zero.
#[must_use]
pub fn decayed_offset(offset: f32, delta_seconds: f32) -> f32 {
    let decay = STEP_SMOOTH_SPEED * delta_seconds;

    if offset < 0.0 {
        (offset + decay).min(0.0)
    } else {
        (offset - decay).max(0.0)
    }
}

/// Returns how far up the player must be lifted to stand on the ledge ahead.
fn step_rise(
    spatial_query: &SpatialQuery,
    entity: Entity,
    origin: Vec3,
    dimensions: PlayerDimensions,
    horizontal_velocity: Vec3,
) -> Option<f32> {
    let direction = Dir3::new(horizontal_velocity.with_y(0.0)).ok()?;
    let filter = SpatialQueryFilter::from_excluded_entities([entity]);
    let feet = origin.y - dimensions.height / 2.0;

    let ankle = origin.with_y(feet + STEP_SKIN);
    let riser = spatial_query.cast_ray(ankle, direction, STEP_PROBE_DISTANCE, true, &filter)?;

    if riser.normal.y >= MIN_WALKABLE_NORMAL_Y {
        return None;
    }

    let knee = origin.with_y(feet + MAX_STEP_HEIGHT + STEP_SKIN);
    if spatial_query
        .cast_ray(knee, direction, STEP_PROBE_DISTANCE, true, &filter)
        .is_some()
    {
        return None;
    }

    let over_the_lip = knee + direction * STEP_PROBE_DISTANCE;
    let tread = spatial_query.cast_ray(
        over_the_lip,
        Dir3::NEG_Y,
        MAX_STEP_HEIGHT + STEP_SKIN,
        true,
        &filter,
    )?;

    if tread.normal.y < MIN_WALKABLE_NORMAL_Y {
        return None;
    }

    let rise = over_the_lip.y - tread.distance - feet;

    (rise > STEP_SKIN).then_some(rise + STEP_SKIN)
}

/// Returns the ground hit under the player within `probe` below the feet, or
/// [`None`] if airborne.
fn ground_hit(
    spatial_query: &SpatialQuery,
    entity: Entity,
    origin: Vec3,
    dimensions: PlayerDimensions,
    probe: f32,
) -> Option<RayHitData> {
    let cast_length = dimensions.height.mul_add(0.5, probe);

    spatial_query.cast_ray(
        origin,
        Dir3::NEG_Y,
        cast_length,
        true,
        &SpatialQueryFilter::from_excluded_entities([entity]),
    )
}

/// Casts the player capsule downwards and returns the ground it would land on
/// within [`MAX_STEP_HEIGHT`], or [`None`] if it is further below.
fn snap_to_ground(
    spatial_query: &SpatialQuery,
    entity: Entity,
    collider: &Collider,
    origin: Vec3,
) -> Option<ShapeHitData> {
    spatial_query.cast_shape(
        collider,
        origin,
        Quat::IDENTITY,
        Dir3::NEG_Y,
        &ShapeCastConfig {
            max_distance: MAX_STEP_HEIGHT,
            ..ShapeCastConfig::default()
        },
        &SpatialQueryFilter::from_excluded_entities([entity]),
    )
}

#[cfg(test)]
mod tests {
    // Honestly, none of these is really needed at this point, though it looks pretty when I run
    // `cargo test`...
    use super::*;
    use std::f32::consts::FRAC_PI_4;

    const SPRINT: f32 = 1.5;
    const EPSILON: f32 = 1e-5;

    fn intent(axis: Vec2) -> MoveIntent {
        MoveIntent {
            axis,
            sprint: false,
            jump: false,
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
        let velocity = movement_velocity(Quat::IDENTITY, MoveIntent::default(), SPRINT);
        assert_eq!(velocity, Vec3::ZERO);
    }

    #[test]
    fn forward_moves_at_base_speed() {
        let velocity = movement_velocity(Quat::IDENTITY, intent(Vec2::Y), SPRINT);
        assert!((velocity - Vec3::NEG_Z * BASE_SPEED).length() < EPSILON);
    }

    #[test]
    fn diagonal_speed_equals_cardinal_speed() {
        let cardinal = movement_velocity(Quat::IDENTITY, intent(Vec2::Y), SPRINT);
        let diagonal = movement_velocity(Quat::IDENTITY, intent(Vec2::new(1.0, 1.0)), SPRINT);
        assert!((diagonal.length() - cardinal.length()).abs() < EPSILON);
    }

    #[test]
    fn sprint_scales_speed_by_multiplier() {
        let walk = movement_velocity(Quat::IDENTITY, intent(Vec2::Y), SPRINT);
        let sprint = movement_velocity(
            Quat::IDENTITY,
            MoveIntent {
                sprint: true,
                ..intent(Vec2::Y)
            },
            SPRINT,
        );
        assert!(walk.length().mul_add(-SPRINT, sprint.length()).abs() < EPSILON);
    }

    #[test]
    fn movement_follows_yaw() {
        let rotation = Quat::from_rotation_y(FRAC_PI_4);
        let velocity = movement_velocity(rotation, intent(Vec2::Y), SPRINT);
        let expected = Vec3::new(-1.0, 0.0, -1.0).normalize() * BASE_SPEED;
        assert!((velocity - expected).length() < EPSILON);
    }

    #[test]
    fn pitch_does_not_slow_movement() {
        // Looking down at 45 degrees must not reduce horizontal speed.
        let rotation = Quat::from_rotation_x(-FRAC_PI_4);
        let velocity = movement_velocity(rotation, intent(Vec2::Y), SPRINT);
        assert!((velocity.length() - BASE_SPEED).abs() < EPSILON);
        assert!(velocity.y.abs() < EPSILON);
    }

    #[test]
    fn movement_is_frame_rate_independent() {
        // The dynamic body integrates velocity itself, so the returned value is a
        // velocity, not a per-frame delta: it must not depend on any timestep.
        let once = movement_velocity(Quat::IDENTITY, intent(Vec2::Y), SPRINT);
        let twice = movement_velocity(Quat::IDENTITY, intent(Vec2::Y), SPRINT);
        assert!((once - twice).length() < EPSILON);
        assert!((once.length() - BASE_SPEED).abs() < EPSILON);
    }

    #[test]
    fn movement_never_touches_vertical_velocity() {
        // Gravity and jumping own the Y axis; horizontal movement must leave it alone.
        for rotation in [
            Quat::IDENTITY,
            Quat::from_rotation_x(-FRAC_PI_4),
            Quat::from_rotation_y(FRAC_PI_4) * Quat::from_rotation_x(FRAC_PI_4),
        ] {
            for axis in [Vec2::Y, Vec2::NEG_Y, Vec2::X, Vec2::new(1.0, 1.0)] {
                let velocity = movement_velocity(rotation, intent(axis), SPRINT);
                assert!(velocity.y.abs() < EPSILON);
            }
        }
    }

    #[test]
    fn jump_speed_overcomes_gravity() {
        const { assert!(JUMP_SPEED > 0.0) }
        const { assert!(crate::world::GRAVITY.y < 0.0) }
    }

    #[test]
    fn ground_probe_reaches_past_the_bottom_of_the_capsule() {
        let dimensions = PlayerDimensions::default();
        let cast_length = dimensions.height.mul_add(0.5, GROUND_PROBE);
        assert!(cast_length > dimensions.height / 2.0);
    }

    #[test]
    fn flat_ground_allows_no_rise() {
        assert!(max_ground_rise(Vec3::Y, BASE_SPEED).abs() < EPSILON);
    }

    #[test]
    fn slope_allows_exactly_its_geometric_climb_rate() {
        // A 45 degree slope rises one unit per unit travelled.
        let normal = Vec3::new(1.0, 1.0, 0.0).normalize();
        assert!((max_ground_rise(normal, BASE_SPEED) - BASE_SPEED).abs() < EPSILON);
    }

    #[test]
    fn shallower_slopes_allow_less_rise() {
        let shallow = Vec3::new(0.2, 1.0, 0.0).normalize();
        let steep = Vec3::new(0.8, 1.0, 0.0).normalize();
        assert!(max_ground_rise(shallow, BASE_SPEED) < max_ground_rise(steep, BASE_SPEED));
    }

    #[test]
    fn standing_still_allows_no_rise() {
        let normal = Vec3::new(1.0, 1.0, 0.0).normalize();
        assert!(max_ground_rise(normal, 0.0).abs() < EPSILON);
    }

    #[test]
    fn vertical_normal_does_not_produce_infinity() {
        assert!(max_ground_rise(Vec3::X, BASE_SPEED).is_finite());
    }

    #[test]
    fn step_offset_decays_towards_zero() {
        let offset = decayed_offset(-0.2, 1.0 / 60.0);
        assert!(offset > -0.2 && offset < 0.0);
    }

    #[test]
    fn step_offset_decay_does_not_overshoot() {
        assert!(decayed_offset(-0.2, 1.0).abs() < EPSILON);
        assert!(decayed_offset(0.2, 1.0).abs() < EPSILON);
    }

    #[test]
    fn step_offset_decay_is_symmetric() {
        let delta = 1.0 / 60.0;
        assert!((decayed_offset(-0.2, delta) + decayed_offset(0.2, delta)).abs() < EPSILON);
    }

    #[test]
    fn settled_camera_stays_settled() {
        assert!(decayed_offset(0.0, 1.0 / 60.0).abs() < EPSILON);
    }

    #[test]
    fn step_offset_reaches_zero_in_finite_time() {
        let mut offset = -MAX_STEP_HEIGHT;
        for _ in 0..600 {
            offset = decayed_offset(offset, 1.0 / 60.0);
        }
        assert!(offset.abs() < EPSILON);
    }
}
