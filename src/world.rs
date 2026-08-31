use crate::player::Player;
use crate::player::components::CameraRig;
use crate::player::components::PlayerCamera;
use crate::player::look::CameraSensitivity;
use avian3d::math::Vector;
use avian3d::prelude::{CoefficientCombine, Collider, Friction, LockedAxes, RigidBody};
use bevy::prelude::{
    Assets, Camera3d, Children, Color, Commands, Cuboid, Mesh, Mesh3d, MeshMaterial3d, Name,
    Plane3d, PointLight, Quat, ResMut, SceneList, StandardMaterial, Transform, Vec2, Vec3,
    Visibility, asset_value, bsn_list,
};

/// Gravity applied to every dynamic body, in units per second squared.
pub(crate) const GRAVITY: Vec3 = Vec3::new(0.0, -20.0, 0.0);

/// Number of steps in the test staircase.
const STAIR_STEPS: u8 = 10;
/// Vertical rise of a single step.
const STAIR_RISE: f32 = 0.2;
/// Horizontal depth of a single step.
const STAIR_RUN: f32 = 0.5;
/// Where the foot of the staircase sits. The stairs climb towards -Z.
const STAIR_ORIGIN: Vec3 = Vec3::new(8.0, 0.0, -4.0);

/// Horizontal distance the ramp covers. The ramp climbs towards -Z.
const RAMP_RUN: f32 = 8.0;
/// Height the ramp gains over [`RAMP_RUN`].
const RAMP_RISE: f32 = 2.0;
/// Thickness of the ramp slab.
const RAMP_THICKNESS: f32 = 0.4;
/// Where the foot of the ramp sits.
const RAMP_ORIGIN: Vec3 = Vec3::new(-8.0, 0.0, -4.0);

/// Width of both the staircase and the ramp.
const WALKWAY_WIDTH: f32 = 3.0;

/// Default scene for the game.
pub fn scene() -> impl SceneList {
    bsn_list! [
        (
            #Plane
            Mesh3d(asset_value(Plane3d::new(Vec3::Y, Vec2::splat(50.0))))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::WHITE))
            RigidBody::from(RigidBody::Static)
            Collider::half_space(Vector::new(0.0, 1.0, 0.0))
        ),
        (
            #RedCube
            Mesh3d(asset_value(Cuboid::new(1.0, 2.0, 1.0)))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.8, 0.2, 0.2)))
            Transform::from_xyz(3.0, 1.0, -5.0)
            RigidBody::from(RigidBody::Static)
            Collider::cuboid(1.0, 2.0, 1.0)
        ),
        (
            #GreenCube
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.2, 0.8, 0.2)))
            Transform::from_xyz(-4.0, 0.5, -3.0)
            RigidBody::from(RigidBody::Static)
            Collider::cuboid(1.0, 1.0, 1.0)
        ),
        (
            #Cube
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
            Transform::from_xyz(0.0, 0.5, 0.0)
            RigidBody::from(RigidBody::Static)
            Collider::cuboid(1.0, 1.0, 1.0)
        ),
        (
            #Sun
            PointLight {
                shadow_maps_enabled: true,
            }
            Transform::from_xyz(4.0, 8.0, 4.0)
        ),
        (
            #Player
            Player
            Transform::from_xyz(0.0, 0.85, 4.0)
            Visibility::default()
            RigidBody::from(RigidBody::Dynamic)
            Collider::capsule(0.3, 1.1)
            LockedAxes::from(LockedAxes::ROTATION_LOCKED)
            Friction {
                dynamic_coefficient: 0.0,
                static_coefficient: 0.0,
                combine_rule: CoefficientCombine::Min,
            }

            Children [
                (
                    #MainCamera
                    Camera3d
                    PlayerCamera
                    CameraRig::default()
                    CameraSensitivity::default()
                    Transform::from_xyz(0.0, 0.85, 0.0)
                )
            ]
        )
    ]
}

/// Spawns the traversal test geometry: a staircase and a ramp.
pub fn spawn_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let stair_material = materials.add(Color::srgb(0.75, 0.6, 0.35));

    for step in 0..STAIR_STEPS {
        let (height, depth) = stair_block(step);

        commands.spawn((
            Name::new(format!("Step{step}")),
            Mesh3d(meshes.add(Cuboid::new(WALKWAY_WIDTH, height, STAIR_RUN))),
            MeshMaterial3d(stair_material.clone()),
            Transform::from_xyz(STAIR_ORIGIN.x, height / 2.0, depth),
            RigidBody::Static,
            Collider::cuboid(WALKWAY_WIDTH, height, STAIR_RUN),
        ));
    }

    let (length, angle, centre_y) = ramp_slab();

    commands.spawn((
        Name::new("Ramp"),
        Mesh3d(meshes.add(Cuboid::new(WALKWAY_WIDTH, RAMP_THICKNESS, length))),
        MeshMaterial3d(materials.add(Color::srgb(0.35, 0.5, 0.7))),
        Transform::from_xyz(RAMP_ORIGIN.x, centre_y, RAMP_ORIGIN.z - RAMP_RUN / 2.0)
            .with_rotation(Quat::from_rotation_x(angle)),
        RigidBody::Static,
        Collider::cuboid(WALKWAY_WIDTH, RAMP_THICKNESS, length),
    ));
}

/// Returns the height and Z centre of one staircase step.
#[must_use]
fn stair_block(step: u8) -> (f32, f32) {
    let height = f32::from(step + 1) * STAIR_RISE;
    let depth = STAIR_RUN.mul_add(-f32::from(step), STAIR_ORIGIN.z);

    (height, depth)
}

/// Returns the length, tilt and centre height of the ramp slab.
#[must_use]
fn ramp_slab() -> (f32, f32, f32) {
    let angle = RAMP_RISE.atan2(RAMP_RUN);
    let length = RAMP_RUN.hypot(RAMP_RISE);
    let centre_y = (RAMP_THICKNESS / 2.0).mul_add(-angle.cos(), RAMP_RISE / 2.0);

    (length, angle, centre_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::movement::{MAX_STEP_HEIGHT, MIN_WALKABLE_NORMAL_Y};

    const EPSILON: f32 = 1e-5;

    fn ramp_surface(end: f32) -> f32 {
        let (length, angle, centre_y) = ramp_slab();
        let tilted = (length / 2.0).mul_add(-(end * angle.sin()), centre_y);

        (RAMP_THICKNESS / 2.0).mul_add(angle.cos(), tilted)
    }

    #[test]
    fn first_step_is_one_rise_tall() {
        let (height, _) = stair_block(0);
        assert!((height - STAIR_RISE).abs() < EPSILON);
    }

    #[test]
    fn steps_gain_one_rise_each() {
        for step in 1..STAIR_STEPS {
            let (height, _) = stair_block(step);
            let (previous, _) = stair_block(step - 1);
            assert!((height - previous - STAIR_RISE).abs() < EPSILON);
        }
    }

    #[test]
    fn steps_recede_by_one_run_each() {
        for step in 1..STAIR_STEPS {
            let (_, depth) = stair_block(step);
            let (_, previous) = stair_block(step - 1);
            assert!((previous - depth - STAIR_RUN).abs() < EPSILON);
        }
    }

    #[test]
    fn staircase_reaches_its_full_height() {
        let (top, _) = stair_block(STAIR_STEPS - 1);
        assert!(f32::from(STAIR_STEPS).mul_add(-STAIR_RISE, top).abs() < EPSILON);
    }

    #[test]
    fn steps_are_low_enough_for_the_player_to_climb() {
        const { assert!(STAIR_RISE < MAX_STEP_HEIGHT) }
    }

    #[test]
    fn ramp_low_end_sits_flush_with_the_ground() {
        assert!(ramp_surface(1.0).abs() < EPSILON);
    }

    #[test]
    fn ramp_high_end_reaches_the_full_rise() {
        assert!((ramp_surface(-1.0) - RAMP_RISE).abs() < EPSILON);
    }

    #[test]
    fn ramp_is_long_enough_to_span_its_run() {
        let (length, ..) = ramp_slab();
        assert!(length >= RAMP_RUN);
    }

    #[test]
    fn ramp_counts_as_walkable_not_as_a_step() {
        let (_, angle, _) = ramp_slab();
        assert!(angle.cos() > MIN_WALKABLE_NORMAL_Y);
    }
}
