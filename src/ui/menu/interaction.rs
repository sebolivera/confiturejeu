use crate::ui::{HOVERED_BUTTON, HOVERED_PRESSED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use bevy::ecs::component::Mutable;
use bevy::prelude::{
    BackgroundColor, Button, Changed, Commands, Component, Entity, Interaction, Query, ResMut,
    Resource, Single, With,
};

/// Tag component used to mark which setting is currently selected
#[derive(Component)]
pub struct SelectedOption;

/// Setting component.
#[derive(Component)]
pub struct Setting<T>(pub T);

/// This system handles changing all buttons color based on mouse interaction
pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

/// This system updates the settings when a new value for a setting is selected, and marks
/// the button as the one currently selected
pub fn setting_button<T: Resource<Mutability = Mutable> + Component + PartialEq + Copy>(
    interaction_query: Query<
        (&Interaction, &Setting<T>, Entity),
        (Changed<Interaction>, With<Button>),
    >,
    selected_query: Single<(Entity, &mut BackgroundColor), With<SelectedOption>>,
    mut commands: Commands,
    mut setting: ResMut<T>,
) {
    let (previous_button, mut previous_button_color) = selected_query.into_inner();
    for (interaction, button_setting, entity) in &interaction_query {
        if *interaction == Interaction::Pressed && *setting != button_setting.0 {
            *previous_button_color = NORMAL_BUTTON.into();
            commands.entity(previous_button).remove::<SelectedOption>();
            commands.entity(entity).insert(SelectedOption);
            *setting = button_setting.0;
        }
    }
}
