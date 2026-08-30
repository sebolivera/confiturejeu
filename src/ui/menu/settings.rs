use crate::ui::menu::MenuButtonAction;
use crate::ui::{NORMAL_BUTTON, TEXT_COLOR};
use bevy::color::palettes::css::CRIMSON;
use bevy::prelude::{
    AlignItems, BackgroundColor, Button, Children, Commands, Component, FlexDirection, FontSize,
    JustifyContent, Node, SpawnIter, SpawnRelated, Text, TextColor, TextFont, UiRect, children,
    default, percent, px,
};

/// Tag component used to tag entities added on the settings menu screen
#[derive(Component)]
pub struct OnSettingsMenuScreen;

/// Settings menu for the settings screen.
pub fn settings_menu_setup(mut commands: Commands) {
    let button_node = Node {
        width: px(200),
        height: px(65),
        margin: UiRect::all(px(20)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font_size: FontSize::Px(33.0),
            ..default()
        },
        TextColor(TEXT_COLOR),
    );

    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnSettingsMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(CRIMSON.into()),
            Children::spawn(SpawnIter(
                [
                    (MenuButtonAction::SettingsDisplay, "Display"),
                    (MenuButtonAction::SettingsSound, "Sound"),
                    (MenuButtonAction::BackToMainMenu, "Back"),
                ]
                .into_iter()
                .map(move |(action, text)| {
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        action,
                        children![(Text::new(text), button_text_style.clone())],
                    )
                })
            ))
        )],
    ));
}
