use crate::levels::*;

use bevy::audio::AudioSink;
use bevy::prelude::*;

use shapeshifter_level_maker::util::{
    HasWonLevelEvent, Polygon, RemainingCuts, SpawnLevel, Target,
};

use super::TEXT_COLOR;

use crate::game::GameButtonAction;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub struct NextButtonParent;

#[derive(Component)]
pub struct RemainingCutsComponent;

pub struct SpawnNextLevelButton;
pub struct SpawnPauseMenu;

pub fn spawn_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_pause_menu_event_reader: EventReader<SpawnPauseMenu>,
) {
    for _ in spawn_pause_menu_event_reader.iter() {
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        // Common style for all buttons on the screen
        let button_style = Style {
            size: Size::new(Val::Px(250.0), Val::Px(65.0)),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };

        let button_text_style = TextStyle {
            font: font.clone(),
            font_size: 40.0,
            color: TEXT_COLOR,
        };
        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::PURPLE.into(),
                ..default()
            })
            .insert(PauseMenu)
            .with_children(|parent| {
                // Display the game name
                parent.spawn_bundle(
                    TextBundle::from_section(
                        "Menu",
                        TextStyle {
                            font: font.clone(),
                            font_size: 80.0,
                            color: TEXT_COLOR,
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    }),
                );

                // Display three buttons for each action available from the main menu:
                // - new game
                // - settings
                // - quit
                parent
                    .spawn_bundle(ButtonBundle {
                        style: button_style.clone(),
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    })
                    .insert(GameButtonAction::GoBack)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            "Previous level",
                            button_text_style.clone(),
                        ));
                    });
                parent
                    .spawn_bundle(ButtonBundle {
                        style: button_style.clone(),
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    })
                    .insert(GameButtonAction::Restart)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            "Restart level",
                            button_text_style.clone(),
                        ));
                    });
                parent
                    .spawn_bundle(ButtonBundle {
                        style: button_style.clone(),
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    })
                    .insert(GameButtonAction::ToMenu)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            "Back to menu",
                            button_text_style.clone(),
                        ));
                    });
            });
    }
}

// }

pub fn spawn_remaining_cuts_label(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    remaining_cuts: ResMut<RemainingCuts>,
) {
    //
    //
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let label = format!("Cuts: {}", remaining_cuts.remaining);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(50.0),
                    top: Val::Px(50.0),
                    ..default()
                },
                ..default()
            },
            color: Color::rgba(0.0, 0.0, 0.0, 0.0).into(),
            // visibility: Visibility { is_visible: false },
            // computed_visibility: ComputedVisibility::not_visible(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section(
                    label,
                    TextStyle {
                        font: font.clone(),
                        font_size: 40.0,
                        color: TEXT_COLOR,
                    },
                ))
                .insert(RemainingCutsComponent);
        });
}

pub fn spawn_next_level_button(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_next_level_button_event_reader: EventReader<SpawnNextLevelButton>,
) {
    //

    // for (entity, mut vis) in go_next_button_query.iter_mut() {
    //     vis.is_visible = true;
    //     commands.entity(entity).remove::<super::menu::Inactive>();
    // }

    if let Some(_) = spawn_next_level_button_event_reader.iter().next() {
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");

        let button_style = Style {
            size: Size::new(Val::Px(150.0), Val::Px(65.0)),
            margin: UiRect::all(Val::Px(3.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };

        let button_text_style = TextStyle {
            font: font.clone(),
            font_size: 32.0,
            color: TEXT_COLOR,
        };

        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        right: Val::Px(150.0),
                        top: Val::Px(50.0),
                        ..default()
                    },
                    ..default()
                },
                color: Color::PURPLE.into(),
                // visibility: Visibility { is_visible: false },
                // computed_visibility: ComputedVisibility::not_visible(),
                ..default()
            })
            .insert(NextButtonParent)
            .with_children(|parent| {
                parent
                    .spawn_bundle(ButtonBundle {
                        style: button_style.clone(),
                        color: NORMAL_BUTTON.into(),
                        visibility: Visibility { is_visible: true },
                        ..default()
                    })
                    .insert(GameButtonAction::GoNext)
                    .insert(crate::menu::Inactive)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            "Next Level",
                            button_text_style.clone(),
                        ));
                    });
            });
    }
}
