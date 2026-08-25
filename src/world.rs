use crate::player::Player;
use crate::player::look::CameraSensitivity;
use bevy::prelude::{
    Camera3d, Children, Color, Cuboid, Mesh3d, MeshMaterial3d, Plane3d, PointLight, SceneList,
    StandardMaterial, Transform, Vec2, Vec3, asset_value, bsn_list,
};

/// Default scene for the game.
pub fn scene() -> impl SceneList {
    bsn_list! [
        (
            #Plane
            Mesh3d(asset_value(Plane3d::new(Vec3::Y, Vec2::splat(50.0))))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::WHITE))
        ),
        (
            #RedCube
            Mesh3d(asset_value(Cuboid::new(1.0, 2.0, 1.0)))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.8, 0.2, 0.2)))
            Transform::from_xyz(3.0, 1.0, -5.0)
        ),
        (
            #GreenCube
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.2, 0.8, 0.2)))
            Transform::from_xyz(-4.0, 0.5, -3.0)
        ),
        (
            #Cube
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
            Transform::from_xyz(0.0, 0.5, 0.0)
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
            CameraSensitivity::default()
            Transform::from_xyz(0.0, 1.7, 0.0)

            Children [
                (
                    #MainCamera
                    Camera3d
                )
            ]
        )
    ]
}
