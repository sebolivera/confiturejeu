use crate::settings::DisplayQuality;
use crate::ui::menu::MenuButtonAction;
use crate::ui::menu::interaction::{SelectedOption, Setting};
use crate::ui::{NORMAL_BUTTON, TEXT_COLOR};
use bevy::color::palettes::css::CRIMSON;
use bevy::prelude::{
    AlignItems, BackgroundColor, Bundle, Button, ChildSpawner, Children, Commands, Component,
    FlexDirection, FontSize, JustifyContent, Node, Res, Spawn, SpawnRelated, SpawnWith, Text,
    TextColor, TextFont, UiRect, children, default, percent, px,
};

/// Tag component used to tag entities added on the display settings menu screen
#[derive(Component)]
pub struct OnDisplaySettingsMenuScreen;

/// Display the settings menu for the display settings.
pub fn display_settings_menu_setup(mut commands: Commands, display_quality: Res<DisplayQuality>) {
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

    let display_quality = *display_quality;
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnDisplaySettingsMenuScreen,
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
                        Spawn((Text::new("Display Quality"), button_text_style())),
                        SpawnWith(move |parent: &mut ChildSpawner| {
                            for quality_setting in [
                                DisplayQuality::Low,
                                DisplayQuality::Medium,
                                DisplayQuality::High,
                            ] {
                                let mut entity = parent.spawn((
                                    Button,
                                    Node {
                                        width: px(150),
                                        height: px(65),
                                        ..button_node()
                                    },
                                    BackgroundColor(NORMAL_BUTTON),
                                    Setting(quality_setting),
                                    children![(
                                        Text::new(format!("{quality_setting:?}")),
                                        button_text_style(),
                                    )],
                                ));
                                if display_quality == quality_setting {
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
