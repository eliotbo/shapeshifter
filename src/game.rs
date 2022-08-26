use crate::levels::*;

use bevy::prelude::*;
use shapeshifter_level_maker::util::{HasWonLevelEvent, SpawnLevel};

use super::{DisplayQuality, GameState, TEXT_COLOR};

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameLevels::default())
            .insert_resource(CurrentLevel::Simplicity(0))
            .add_event::<NextLevel>()
            .add_event::<PreviousLevel>()
            .add_event::<WonTheGame>()
            .add_system_set(SystemSet::on_exit(GameState::Game).with_system(delete_game_entities))
            .add_system_set(
                SystemSet::on_enter(GameState::Game)
                    .with_system(game_setup)
                    .with_system(spawn_next_level_button),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(next_level)
                    .with_system(previous_level)
                    .with_system(next_button_action)
                    .with_system(force_next_level)
                    .with_system(activate_next_level_button),
            );

        // .add_system_set(
        //     SystemSet::on_exit(GameState::Game).with_system(despawn_screen::<OnGameScreen>),
        // );
    }
}

// #[derive(Deref, DerefMut)]
// struct GameTimer(Timer);

pub enum CurrentLevel {
    Simplicity(usize),
    Convexity(usize),
    Perplexity(usize),
    Complexity(usize),
}

impl CurrentLevel {
    pub fn simplicity(&mut self, x: usize) {
        *self = CurrentLevel::Simplicity(x);
    }
    pub fn convexity(&mut self, x: usize) {
        *self = CurrentLevel::Convexity(x);
    }
    pub fn perplexity(&mut self, x: usize) {
        *self = CurrentLevel::Perplexity(x);
    }
    pub fn complexity(&mut self, x: usize) {
        *self = CurrentLevel::Complexity(x);
    }
}

pub struct NextLevel;
pub struct PreviousLevel;
pub struct WonTheGame;

#[derive(Component)]
pub struct PauseMenu;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);

#[derive(Component)]
enum GameButtonAction {
    GoNext,
    Revert,
    GoBack,
}

fn delete_game_entities(
    mut commands: Commands,
    query: Query<Entity>,
    mut current_level: ResMut<CurrentLevel>,
) {
    current_level.simplicity(0);
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    // Common style for all buttons on the screen
    let button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_icon_style = Style {
        size: Size::new(Val::Px(30.0), Val::Auto),
        // This takes the icons out of the flexbox flow, to be positioned exactly
        position_type: PositionType::Absolute,
        // The icon will be close to the left border of the button
        position: UiRect {
            left: Val::Px(10.0),
            right: Val::Auto,
            top: Val::Auto,
            bottom: Val::Auto,
        },
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
                        "Previous",
                        button_text_style.clone(),
                    ));
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(GameButtonAction::Revert)
                .with_children(|parent| {
                    let icon = asset_server.load("textures/Game Icons/right.png");
                    parent.spawn_bundle(ImageBundle {
                        style: button_icon_style.clone(),
                        image: UiImage(icon),
                        ..default()
                    });
                    parent.spawn_bundle(TextBundle::from_section(
                        "Go to city",
                        button_text_style.clone(),
                    ));
                });
        });
}

fn show_pause_menu() {}

fn go_back_to_menu(mut commands: Commands, mut game_state: ResMut<State<GameState>>) {
    game_state.set(GameState::Menu).unwrap();
}

fn next_level(
    // mut commands: Commands,
    game_levels: ResMut<GameLevels>,
    mut next_level_event_reader: EventReader<NextLevel>,
    mut current_level: ResMut<CurrentLevel>,
    mut won_the_game_event_writer: EventWriter<WonTheGame>,
    mut spawn_level_event_writer: EventWriter<SpawnLevel>,
) {
    if let Some(_) = next_level_event_reader.iter().next() {
        match *current_level {
            CurrentLevel::Simplicity(level) => {
                if level < game_levels.simplicity.len() - 1 {
                    current_level.simplicity(level + 1);
                    spawn_level_event_writer.send(game_levels.simplicity[level + 1].clone());
                } else {
                    current_level.convexity(0);
                    spawn_level_event_writer.send(game_levels.convexity[0].clone());
                }
            }
            CurrentLevel::Convexity(level) => {
                if level < game_levels.convexity.len() - 1 {
                    current_level.convexity(level + 1);
                    spawn_level_event_writer.send(game_levels.convexity[level + 1].clone());
                } else {
                    current_level.perplexity(0);
                    spawn_level_event_writer.send(game_levels.perplexity[0].clone());
                }
                spawn_level_event_writer.send(game_levels.simplicity[level].clone());
            }
            CurrentLevel::Perplexity(level) => {
                if level < game_levels.perplexity.len() - 1 {
                    current_level.perplexity(level + 1);
                    spawn_level_event_writer.send(game_levels.perplexity[level + 1].clone());
                } else {
                    current_level.complexity(0);
                    spawn_level_event_writer.send(game_levels.complexity[0].clone());
                }
            }
            CurrentLevel::Complexity(level) => {
                if level < game_levels.complexity.len() - 1 {
                    current_level.complexity(level + 1);
                    spawn_level_event_writer.send(game_levels.complexity[level + 1].clone());
                } else {
                    won_the_game_event_writer.send(WonTheGame);
                }
            }
        }
    }
}

fn previous_level(
    // mut commands: Commands,
    game_levels: ResMut<GameLevels>,
    mut previous_level_event_reader: EventReader<PreviousLevel>,
    mut current_level: ResMut<CurrentLevel>,
    mut spawn_level_event_writer: EventWriter<SpawnLevel>,
) {
    if let Some(_) = previous_level_event_reader.iter().next() {
        match *current_level {
            CurrentLevel::Simplicity(level) => {
                if level > 0 {
                    current_level.simplicity(level - 1);
                    spawn_level_event_writer.send(game_levels.simplicity[level - 1].clone());
                } // do nothing if we're at the first level
            }
            CurrentLevel::Convexity(level) => {
                if level > 0 {
                    current_level.convexity(level - 1);
                    spawn_level_event_writer.send(game_levels.convexity[level - 1].clone());
                } else {
                    current_level.simplicity(game_levels.simplicity.len() - 1);
                    spawn_level_event_writer
                        .send(game_levels.simplicity[game_levels.simplicity.len() - 1].clone());
                }
            }
            CurrentLevel::Perplexity(level) => {
                if level > 0 {
                    current_level.perplexity(level - 1);
                    spawn_level_event_writer.send(game_levels.perplexity[level - 1].clone());
                } else {
                    current_level.convexity(game_levels.convexity.len() - 1);
                    spawn_level_event_writer
                        .send(game_levels.convexity[game_levels.convexity.len() - 1].clone());
                }
            }
            CurrentLevel::Complexity(level) => {
                if level > 0 {
                    current_level.complexity(level - 1);
                    spawn_level_event_writer.send(game_levels.complexity[level - 1].clone());
                } else {
                    current_level.perplexity(game_levels.perplexity.len() - 1);
                    spawn_level_event_writer
                        .send(game_levels.perplexity[game_levels.perplexity.len() - 1].clone());
                }
            }
        }
    }
}

fn force_next_level(
    keyboard_input: Res<Input<KeyCode>>,
    mut next_level_event_writer: EventWriter<NextLevel>,
    mut previous_level_event_writer: EventWriter<PreviousLevel>,
) {
    if keyboard_input.just_pressed(KeyCode::Right) {
        next_level_event_writer.send(NextLevel);
    }
    if keyboard_input.just_pressed(KeyCode::Left) {}
}

fn next_button_action(
    mut commands: Commands,
    mut interaction_query: Query<
        (Entity, &mut Visibility, &Interaction, &GameButtonAction),
        (Changed<Interaction>, With<Button>),
    >,

    mut next_level_event_writer: EventWriter<NextLevel>,
) {
    for (entity, mut vis, interaction, menu_button_action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                // MenuButtonAction::Quit => app_exit_events.send(AppExit),
                GameButtonAction::GoNext => {
                    next_level_event_writer.send(NextLevel);
                    commands.entity(entity).insert(super::menu::Inactive);
                    vis.is_visible = false;
                }
                _ => {}
            }
        }
    }
}

fn activate_next_level_button(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut has_won_event_reader: EventReader<HasWonLevelEvent>,
    mut go_next_button_query: Query<(Entity, &mut Visibility), (With<Button>)>,
) {
    //
    for _ in has_won_event_reader.iter() {
        for (entity, mut vis) in go_next_button_query.iter_mut() {
            vis.is_visible = true;
            commands.entity(entity).remove::<super::menu::Inactive>();
        }
    }
}

fn spawn_next_level_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    //

    // for (entity, mut vis) in go_next_button_query.iter_mut() {
    //     vis.is_visible = true;
    //     commands.entity(entity).remove::<super::menu::Inactive>();
    // }

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
            color: Color::rgba(0.0, 0.0, 0.0, 0.0).into(),
            // visibility: Visibility { is_visible: false },
            computed_visibility: ComputedVisibility::not_visible(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    visibility: Visibility { is_visible: false },
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

fn game_setup(
    // mut spawn_poly_event_writer: EventWriter<SpawnPoly>,
    // mut spawn_target_event_writer: EventWriter<SpawnTarget>,
    mut spawn_level_event_writer: EventWriter<SpawnLevel>,
    game_levels: ResMut<GameLevels>,
) {
    // let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    // spawn_level_event_writer.send(SpawnLevel {
    //     polygon: "004_simplicity_square_parallel".to_string(),
    //     polygon_multiplier: 1.0,
    //     target: "004_simplicity_square_parallel".to_string(),
    //     target_multiplier: 1.1,
    // });

    spawn_level_event_writer.send(game_levels.simplicity[0].clone());

    // spawn_poly_event_writer.send(SpawnPoly {
    //     polygon: "004_simplicity_square_parallel".to_string(),
    //     polygon_multiplier: 1.0,
    // });

    // spawn_target_event_writer.send(SpawnTarget {
    //     target: "004_simplicity_square_parallel".to_string(),
    //     target_multiplier: 1.1,
    // });

    // commands
    //     // First create a `NodeBundle` for centering what we want to display
    //     .spawn_bundle(NodeBundle {
    //         style: Style {
    //             // This will center the current node
    //             margin: UiRect::all(Val::Auto),
    //             // This will display its children in a column, from top to bottom. Unlike
    //             // in Flexbox, Bevy origin is on bottom left, so the vertical axis is reversed
    //             flex_direction: FlexDirection::ColumnReverse,
    //             // `align_items` will align children on the cross axis. Here the main axis is
    //             // vertical (column), so the cross axis is horizontal. This will center the
    //             // children
    //             align_items: AlignItems::Center,
    //             ..default()
    //         },
    //         color: Color::BLACK.into(),
    //         ..default()
    //     })

    //     .with_children(|parent| {
    //         // Display two lines of text, the second one with the current settings
    //         parent.spawn_bundle(
    //             TextBundle::from_section(
    //                 "Will be back to the menu shortly...",
    //                 TextStyle {
    //                     font: font.clone(),
    //                     font_size: 80.0,
    //                     color: TEXT_COLOR,
    //                 },
    //             )
    //             .with_style(Style {
    //                 margin: UiRect::all(Val::Px(50.0)),
    //                 ..default()
    //             }),
    //         );
    //         parent.spawn_bundle(
    //             TextBundle::from_sections([
    //                 TextSection::new(
    //                     format!("quality: {:?}", *display_quality),
    //                     TextStyle {
    //                         font: font.clone(),
    //                         font_size: 60.0,
    //                         color: Color::BLUE,
    //                     },
    //                 ),
    //                 TextSection::new(
    //                     " - ",
    //                     TextStyle {
    //                         font: font.clone(),
    //                         font_size: 60.0,
    //                         color: TEXT_COLOR,
    //                     },
    //                 ),
    //             ])
    //             .with_style(Style {
    //                 margin: UiRect::all(Val::Px(50.0)),
    //                 ..default()
    //             }),
    //         );
    //     });
}
