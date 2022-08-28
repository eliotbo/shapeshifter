use bevy::prelude::*;

use shapeshifter_level_maker::util::RemainingCuts;

use super::TEXT_COLOR;

use crate::game::{GameButtonAction, WholeGameCuts, WonTheGame};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub struct OptionButton;

#[derive(Component)]
pub struct NextButtonParent;

#[derive(Component)]
pub struct Instruction;

#[derive(Component)]
pub struct RemainingCutsComponent;

pub struct SpawnNextLevelButton;
pub struct TogglePauseMenu;
pub struct SpawnInstruction {
    pub text: String,
}

pub fn spawn_won_screen(
    mut commands: Commands,
    whole_game_cut: Res<WholeGameCuts>,
    asset_server: Res<AssetServer>,
    mut won_the_game_event_reader: EventReader<WonTheGame>,
) {
    //
    if let Some(_) = won_the_game_event_reader.iter().next() {
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");

        let text = format!("You won the game with {} cuts!", whole_game_cut.cuts);
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 30.0,
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
                        // left: Val::Px(50.0),
                        top: Val::Px(50.0),
                        ..default()
                    },
                    ..default()
                },
                color: Color::rgba(0., 0., 0., 0.).into(),
                ..default()
            })
            .insert(Instruction)
            .with_children(|parent| {
                // Display the game name
                parent.spawn_bundle(
                    TextBundle::from_section(text, text_style).with_style(Style {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    }),
                );
            });
    }
}

pub fn spawn_instruction(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_instruction_event_reader: EventReader<SpawnInstruction>,
) {
    if let Some(instruction) = spawn_instruction_event_reader.iter().next() {
        //
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 30.0,
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
                        left: Val::Px(50.0),
                        bottom: Val::Px(50.0),
                        ..default()
                    },
                    ..default()
                },
                color: Color::rgba(0., 0., 0., 0.).into(),
                ..default()
            })
            .insert(Instruction)
            .with_children(|parent| {
                // Display the game name
                parent.spawn_bundle(
                    TextBundle::from_section(&instruction.text, text_style).with_style(Style {
                        margin: UiRect::all(Val::Px(5.0)),
                        ..default()
                    }),
                );
            });
    }
}

pub fn spawn_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pause_menu_query: Query<Entity, With<PauseMenu>>,
    mut toggle_pause_menu_event_reader: EventReader<TogglePauseMenu>,
) {
    for _ in toggle_pause_menu_event_reader.iter() {
        if let Some(entity) = pause_menu_query.iter().next() {
            commands.entity(entity).despawn_recursive();
            info!("Despawning pause menu");
        } else {
            info!("spawning pause menu");
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
                            "Options",
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
                    top: Val::Px(130.0),
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

pub fn spawn_options_button(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut spawn_next_level_button_event_reader: EventReader<SpawnNextLevelButton>,
) {
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
                    left: Val::Px(50.0),
                    top: Val::Px(25.0),
                    ..default()
                },
                ..default()
            },
            color: Color::PURPLE.into(),
            // visibility: Visibility { is_visible: false },
            // computed_visibility: ComputedVisibility::not_visible(),
            ..default()
        })
        .insert(OptionButton)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    visibility: Visibility { is_visible: true },
                    ..default()
                })
                .insert(GameButtonAction::OptionsMenu)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section(
                        "Options",
                        button_text_style.clone(),
                    ));
                });
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
                        right: Val::Px(50.0),
                        top: Val::Px(25.0),
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
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            "Next Level",
                            button_text_style.clone(),
                        ));
                    });
            });
    }
}
