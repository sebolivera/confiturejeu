use bevy::prelude::{
    AlignItems, AssetServer, BackgroundColor, Color, Commands, Component, Deref, DerefMut,
    ImageNode, JustifyContent, Node, Res, Resource, Timer, TimerMode, children, default, percent,
    px,
};

/// Plugin for the splash screen
pub mod plugin;

/// Tag component used to tag entities added on the splash screen
#[derive(Component)]
pub struct OnSplashScreen;

/// Newtype to use a `Timer` for this screen as a resource
#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

/// Sets up the splash screen with a Bevy logo and a timer
pub fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load("branding/icon.png");
    commands.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        OnSplashScreen,
        children![(
            ImageNode::new(icon),
            Node {
                width: px(200),
                ..default()
            },
        )],
    ));
    commands.insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)));
}
