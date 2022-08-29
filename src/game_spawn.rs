use bevy::prelude::*;

use bevy_easings::*;
use shapeshifter_level_maker::util::{Polygon, RemainingCuts, SpawnLevel, Target};

use super::TEXT_COLOR;

use crate::game::{GameButtonAction, WholeGameCuts, WonTheGame};
use crate::levels::*;
use crate::menu::FontHandles;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const CITY_TITLE_MILLIS: u64 = 3000;

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
    // asset_server: Res<AssetServer>,
    mut won_the_game_event_reader: EventReader<WonTheGame>,
    fonts: Res<FontHandles>,
    sound_map: Res<crate::menu::SoundMap>,
    audio: Res<Audio>,
) {
    //
    if let Some(_) = won_the_game_event_reader.iter().next() {
        // let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let font = fonts.font.clone();

        sound_map.play("final_victory", &audio);

        let text = format!(
            "You solved all 
the puzzles!
# cuts: {}",
            whole_game_cut.cuts
        );
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 80.0,
            color: Color::TEAL,
        };

        let start_style = Style {
            margin: UiRect::all(Val::Auto),
            flex_direction: FlexDirection::RowReverse,
            // align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            position: UiRect {
                right: Val::Px(50.0),
                top: Val::Px(-25.0),
                ..default()
            },
            ..default()
        };

        commands
            .spawn_bundle(NodeBundle {
                style: start_style,
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
    // asset_server: Res<AssetServer>,
    mut spawn_instruction_event_reader: EventReader<SpawnInstruction>,
    fonts: Res<FontHandles>,
) {
    if let Some(instruction) = spawn_instruction_event_reader.iter().next() {
        //
        // let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let font = fonts.font.clone();
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 40.0,
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

#[derive(Default)]
pub struct CityTitleTimer {
    pub maybe_timer: Option<Timer>,
}

#[derive(Component)]
pub struct CityTitle;

pub fn despawn_city(
    mut commands: Commands,
    // mut spawn_level_event_writer: EventWriter<SpawnLevel>,
    mut city_title_timer: ResMut<CityTitleTimer>,
    // game_levels: ResMut<GameLevels>,
    time: Res<Time>,
    query: Query<Entity, With<CityTitle>>,
    mut game_state: ResMut<State<crate::GameState>>,
) {
    //
    if let Some(ref mut timer) = city_title_timer.maybe_timer {
        if timer.tick(time.delta()).just_finished() {
            game_state.set(crate::GameState::Game).unwrap();

            for entity in query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub fn spawn_city_title(
    mut commands: Commands,
    fonts: Res<FontHandles>,
    current_level: Res<CurrentLevel>,
    // pause_menu_query: Query<Entity, With<PauseMenu>>,
    // mut city_title_timer: ResMut<CityTitleTimer>,
    // delete_query: Query<Entity, Or<(With<Polygon>, With<Target>, With<NextButtonParent>)>>,
    // mut spawn_city_title_event_reader: EventReader<SpawnCityTitle>,
) {
    let font = fonts.font.clone();

    commands.insert_resource(CityTitleTimer {
        maybe_timer: Some(Timer::new(
            bevy::utils::Duration::from_millis(CITY_TITLE_MILLIS),
            false,
        )),
    });

    let city_str = match current_level.level {
        Level::Tutorial(_) => "TUTORIAL",
        Level::Simplicity(_) => "SIMPLICITY",
        // Level::Convexity(_) => "CONVEXITY",
        Level::Perplexity(_) => "PERPLEXITY",
        Level::Complexity(_) => "COMPLEXITY",
        _ => "",
    };

    let label = format!("{}", city_str);

    let ease_function = bevy_easings::EaseFunction::ExponentialInOut;

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 100.0,
        color: super::TEXT_COLOR,
    };

    let start_style = Style {
        margin: UiRect::all(Val::Auto),
        flex_direction: FlexDirection::ColumnReverse,
        align_items: AlignItems::Center,
        ..default()
    };

    let sc = 0.3;

    let easing_scale = Transform::from_scale(Vec3::new(sc, sc, 1.0)).ease_to(
        Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
        ease_function,
        bevy_easings::EasingType::Once {
            duration: std::time::Duration::from_millis(CITY_TITLE_MILLIS),
        },
    );

    commands
        .spawn_bundle(NodeBundle {
            style: start_style,
            color: Color::rgba(0.0, 0.0, 0.0, 0.0).into(),
            transform: Transform::from_scale(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        })
        .insert(easing_scale)
        .insert(CityTitle)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section(label, text_style))
                .insert(RemainingCutsComponent);
        });
}

pub fn spawn_pause_menu(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    pause_menu_query: Query<Entity, With<PauseMenu>>,
    mut toggle_pause_menu_event_reader: EventReader<TogglePauseMenu>,
    fonts: Res<FontHandles>,
) {
    for _ in toggle_pause_menu_event_reader.iter() {
        if let Some(entity) = pause_menu_query.iter().next() {
            commands.entity(entity).despawn_recursive();
            info!("Despawning pause menu");
        } else {
            info!("spawning pause menu");
            // let font = asset_server.load("fonts/FiraSans-Bold.ttf");
            let font = fonts.font.clone();
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
                font_size: 50.0,
                color: TEXT_COLOR,
            };
            commands
                .spawn_bundle(NodeBundle {
                    style: Style {
                        margin: UiRect::all(Val::Auto),
                        flex_direction: FlexDirection::ColumnReverse,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            bottom: Val::Px(150.0),
                            left: Val::Px(470.0),
                            ..default()
                        },
                        // align_self: AlignSelf::Center,
                        ..default()
                    },
                    color: Color::PURPLE.into(),
                    // transform: Transform::from_translation(Vec3::new(0.0, 0.0, 111.0)),
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
                                font_size: 110.0,
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

#[derive(Component)]
pub struct LevelInt;

pub fn spawn_current_level(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    game_levels: Res<GameLevels>,
    current_level: Res<crate::levels::CurrentLevel>,
    fonts: Res<FontHandles>,
) {
    // let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font = fonts.font.clone();

    let level_int = game_levels.to_int(&current_level.level.clone()) + 1;
    let label = format!("Level {} / {}", level_int, game_levels.get_total_levels());

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                // align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Relative,
                // align_items: AlignItems::Center,
                position: UiRect {
                    bottom: Val::Percent(-45.0),
                    right: Val::Percent(-35.0),
                    ..default()
                },
                ..default()
            },
            color: Color::rgba(0.0, 0.0, 0.0, 0.0).into(),
            // visibility: Visibility { is_visible: false },
            // computed_visibility: ComputedVisibility::not_visible(),
            ..default()
        })
        .insert(LevelInt)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle::from_section(
                    label,
                    TextStyle {
                        font: font.clone(),
                        font_size: 60.0,
                        color: TEXT_COLOR,
                    },
                ))
                .insert(LevelInt);
        });
}

// }

pub fn spawn_remaining_cuts_label(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    remaining_cuts: ResMut<RemainingCuts>,
    fonts: Res<FontHandles>,
) {
    //
    //
    // let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font = fonts.font.clone();

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
                        font_size: 60.0,
                        color: TEXT_COLOR,
                    },
                ))
                .insert(RemainingCutsComponent);
        });
}

pub fn spawn_options_button(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    fonts: Res<FontHandles>,
    // mut spawn_next_level_button_event_reader: EventReader<SpawnNextLevelButton>,
) {
    // let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font = fonts.font.clone();

    let button_style = Style {
        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(3.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 50.0,
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
    mut spawn_next_level_button_event_reader: EventReader<SpawnNextLevelButton>,
    fonts: Res<FontHandles>,
) {
    //

    // for (entity, mut vis) in go_next_button_query.iter_mut() {
    //     vis.is_visible = true;
    //     commands.entity(entity).remove::<super::menu::Inactive>();
    // }

    if let Some(_) = spawn_next_level_button_event_reader.iter().next() {
        // let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let font = fonts.font.clone();

        let button_style = Style {
            size: Size::new(Val::Px(150.0), Val::Px(65.0)),
            margin: UiRect::all(Val::Px(3.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };

        let button_text_style = TextStyle {
            font: font.clone(),
            font_size: 45.0,
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
                        top: Val::Px(25.0),
                        ..default()
                    },
                    ..default()
                },
                color: Color::PURPLE.into(),
                // transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
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
