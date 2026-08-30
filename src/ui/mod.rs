use bevy::prelude::{Color, Commands, Component, Entity, Query, With};

/// Menu module
pub mod menu;
/// Splash screen module
pub mod splash;

/// Default text color
pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
/// Normal button color
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
/// Hovered button color
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
/// Hovered and pressed button color
pub const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
/// Pressed button color
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

/// Despawns every entity tagged with `T`.
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
