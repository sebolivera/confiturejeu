use crate::settings::Volume;
use crate::ui::menu::MenuButtonAction;
use crate::ui::menu::interaction::{SelectedOption, Setting};
use crate::ui::{NORMAL_BUTTON, TEXT_COLOR};
use bevy::color::palettes::css::CRIMSON;
use bevy::prelude::{
    AlignItems, BackgroundColor, Bundle, Button, ChildSpawner, Children, Commands, Component,
    FlexDirection, FontSize, JustifyContent, Node, Res, Spawn, SpawnRelated, SpawnWith, Text,
    TextColor, TextFont, UiRect, children, default, percent, px,
};

/// Tag component used to tag entities added on the sound settings menu screen
#[derive(Component)]
pub struct OnSoundSettingsMenuScreen;

/// Sets up the sound settings menu
pub fn sound_settings_menu_setup(mut commands: Commands, volume: Res<Volume>) {
    fn button_node() -> Node {
        Node {
            width: px(200),
            height: px(65),
            margin: UiRect::all(px(20)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }
    }
    fn button_text_style() -> impl Bundle {
        (
            TextFont {
                font_size: FontSize::Px(33.0),
                ..default()
            },
            TextColor(TEXT_COLOR),
        )
    }

    let volume = *volume;
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnSoundSettingsMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(CRIMSON.into()),
            children![
                (
                    Node {
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(CRIMSON.into()),
                    Children::spawn((
                        Spawn((Text::new("Volume"), button_text_style())),
                        SpawnWith(move |parent: &mut ChildSpawner| {
                            for volume_setting in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
                                let mut entity = parent.spawn((
                                    Button,
                                    Node {
                                        width: px(30),
                                        height: px(65),
                                        ..button_node()
                                    },
                                    BackgroundColor(NORMAL_BUTTON),
                                    Setting(Volume(volume_setting)),
                                ));
                                if volume == Volume(volume_setting) {
                                    entity.insert(SelectedOption);
                                }
                            }
                        })
                    ))
                ),
                (
                    Button,
                    button_node(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::BackToSettings,
                    children![(Text::new("Back"), button_text_style())]
                )
            ]
        )],
    ));
}
