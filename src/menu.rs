// use crate::game;
use crate::levels;

use bevy::audio::AudioSink;
use bevy::prelude::*;
use shapeshifter_level_maker::util::{PerformedCut, Polygon, SpawnLevel, Target};

use super::{despawn_screen, GameState, TEXT_COLOR};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Current screen in the menu is handled by an independent state from `GameState`
            .add_state(MenuState::Disabled)
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(menu_setup))
            // Systems to handle the main menu screen
            .add_system_set(SystemSet::on_enter(MenuState::Main).with_system(main_menu_setup))
            .add_system_set(
                SystemSet::on_exit(MenuState::Main).with_system(despawn_screen::<OnMainMenuScreen>),
            )
            // Systems to handle the settings menu screen
            .add_system_set(
                SystemSet::on_enter(MenuState::Settings).with_system(settings_menu_setup),
            )
            .add_system_set(
                SystemSet::on_exit(MenuState::Settings)
                    .with_system(despawn_screen::<OnSettingsMenuScreen>),
            )
            // Common systems to all screens that handles buttons behaviour
            .add_system_set(
                SystemSet::on_update(GameState::Menu)
                    .with_system(menu_action)
                    .with_system(play_cut_sound)
                    .with_system(button_system),
            )
            .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(remove_poly_target));
    }
}

fn remove_poly_target(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Target>, With<Polygon>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct FontHandles {
    pub font: Handle<Font>,
}

// State used for the current menu screen
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MenuState {
    Main,
    // Design,
    Settings,
    Disabled,
}
// Tag component used to tag buttons that cannot be interacted with
#[derive(Component)]
pub struct Inactive;

// Tag component used to tag entities added on the main menu screen
#[derive(Component)]
struct OnMainMenuScreen;

// Tag component used to tag entities added on the settings menu screen
#[derive(Component)]
struct OnSettingsMenuScreen;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// Tag component used to mark wich setting is currently selected
#[derive(Component)]
pub struct SelectedOption;

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    Play,
    GoToCity,
    Tutorial,
    Simplicity,
    // Convexity,
    Perplexity,
    Complexity,
    // Design,
    BackToMainMenu,
}

// This system handles changing all buttons color based on mouse interaction
pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>, Without<Inactive>),
    >,
    sound_map: Res<crate::menu::SoundMap>,
    audio: Res<Audio>,
) {
    for (interaction, mut color, selected) in &mut interaction_query {
        *color = match (*interaction, selected) {
            (Interaction::Clicked, _) | (Interaction::None, Some(_)) => {
                sound_map.play("bip", &audio);
                PRESSED_BUTTON.into()
            }
            (Interaction::Hovered, Some(_)) => {
                sound_map.play("click", &audio);
                HOVERED_PRESSED_BUTTON.into()
            }
            (Interaction::Hovered, None) => {
                sound_map.play("click", &audio);
                HOVERED_BUTTON.into()
            }
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

// // This system updates the settings when a new value for a setting is selected, and marks
// // the button as the one currently selected
// fn setting_button<T: Component + PartialEq + Copy>(
//     interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
//     mut selected_query: Query<(Entity, &mut UiColor), With<SelectedOption>>,
//     mut commands: Commands,
//     mut setting: ResMut<T>,
// ) {
//     for (interaction, button_setting, entity) in &interaction_query {
//         if *interaction == Interaction::Clicked && *setting != *button_setting {
//             let (previous_button, mut previous_color) = selected_query.single_mut();
//             *previous_color = NORMAL_BUTTON.into();
//             commands.entity(previous_button).remove::<SelectedOption>();
//             commands.entity(entity).insert(SelectedOption);
//             *setting = *button_setting;
//         }
//     }
// }

pub struct MusicController(pub Handle<AudioSink>);

use std::collections::HashMap;
pub struct SoundMap {
    pub map: HashMap<String, Vec<Handle<AudioSource>>>,
}

use rand::Rng;
// use rand::Thread;

impl SoundMap {
    //
    //
    // plays a random sound in the sound group
    pub fn play(&self, sound_group: &str, audio: &Audio) {
        let mut rng = rand::thread_rng();
        let sounds = self.map.get(sound_group).unwrap();
        // let random_index = rng.gen_range(0..sounds.len());
        let random_index = rng.gen_range(0..sounds.len()).clone();
        let sound = &sounds[random_index];
        audio.play(sound.clone());
    }
}

fn menu_setup(
    mut commands: Commands,
    mut menu_state: ResMut<State<MenuState>>,
    asset_server: Res<AssetServer>,
    // mut menu_music: ResMut<MenuMusic>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    //
    //
    // music settings
    let menu_music_handle = asset_server.load("music/charles_menu.ogg");
    let handle = audio_sinks.get_handle(
        audio.play_with_settings(menu_music_handle, bevy::audio::PlaybackSettings::LOOP),
    );
    commands.insert_resource(MusicController(handle));

    //
    //
    // initialize menu state
    let _ = menu_state.set(MenuState::Main);

    //
    //
    // sounds
    let mut sound_map = HashMap::new();

    // let click1 = asset_server.load("sounds/Menu/Menu Click 1.ogg"); // too different
    let click2 = asset_server.load("sounds/Menu/Menu Click 2.ogg");
    let click3 = asset_server.load("sounds/Menu/Menu Click 3.ogg");
    let click4 = asset_server.load("sounds/Menu/Menu Click 4.ogg");
    let click5 = asset_server.load("sounds/Menu/Menu Click 5.ogg");
    let click7 = asset_server.load("sounds/Menu/Menu Click 7.ogg");
    let bip = asset_server.load("sounds/Menu/Menu Bip.ogg");

    // let scissor1 = asset_server.load("sounds/Scissor Cut/Scissor paper cut 1.ogg"); // too long
    let scissor2 = asset_server.load("sounds/Scissor Cut/Scissor paper cut 2.ogg");
    let scissor3 = asset_server.load("sounds/Scissor Cut/Scissor paper cut 3.ogg");
    // let scissor4 = asset_server.load("sounds/Scissor Cut/Scissor paper cut 4.ogg"); // too long
    let scissor5 = asset_server.load("sounds/Scissor Cut/Scissor paper cut 5.ogg");
    let scissor6 = asset_server.load("sounds/Scissor Cut/Scissor paper cut 6.ogg");

    let place1 = asset_server.load("sounds/Placement Ding/Placement Ding 1.ogg");
    let place2 = asset_server.load("sounds/Placement Ding/Placement Ding 2.ogg");
    let place3 = asset_server.load("sounds/Placement Ding/Placement Ding 3.ogg");
    let place4 = asset_server.load("sounds/Placement Ding/Placement Ding 4.ogg");

    // let victoy1 = asset_server.load("sounds/Victory Sounds/Victory Cartoon 1.ogg");
    // let victoy2 = asset_server.load("sounds/Victory Sounds/Victory Cartoon 2.ogg");
    let victoy3 = asset_server.load("sounds/Victory Sounds/Victory Cartoon 3.ogg");
    // let victory_chord1 = asset_server.load("sounds/Victory Sounds/Victory Chord 1.ogg");
    // let victory_chord2 = asset_server.load("sounds/Victory Sounds/Victory Chord 2.ogg");
    // let victory_chord3 = asset_server.load("sounds/Victory Sounds/Victory Chord 3.ogg");
    // let victory_chord4 = asset_server.load("sounds/Victory Sounds/Victory Chord 4.ogg");
    // let victory_ukulele = asset_server.load("sounds/Victory Sounds/Victory Ukulele.ogg");

    let final_victory = asset_server.load("sounds/Victory Finale.ogg");

    sound_map.insert(
        "click".to_string(),
        vec![click2, click3, click4, click5, click7],
    );
    sound_map.insert("bip".to_string(), vec![bip]);

    sound_map.insert(
        "cut".to_string(),
        vec![scissor2, scissor3, scissor5, scissor6],
    );

    sound_map.insert("target".to_string(), vec![place1, place2, place3, place4]);

    sound_map.insert(
        "victory".to_string(),
        vec![
            // victoy1,
            // victoy2,
            victoy3,
            // victory_chord1,
            // victory_chord2,
            // victory_chord3,
            // victory_chord4,
            // victory_ukulele,
        ],
    );

    sound_map.insert("final_victory".to_string(), vec![final_victory]);

    commands.insert_resource(SoundMap { map: sound_map });
}

fn play_cut_sound(
    mut performed_cut_event_reader: EventReader<PerformedCut>,

    sound_map: Res<crate::menu::SoundMap>,
    audio: Res<Audio>,
) {
    for _ in performed_cut_event_reader.iter() {
        sound_map.play("cut", &audio);
    }
}

fn main_menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_level_event_writer: EventWriter<SpawnLevel>,
    current_level: Res<crate::levels::CurrentLevel>,
) {
    let new_game_label = if current_level.level == crate::levels::Level::Tutorial(0) {
        "New Game"
    } else {
        "Continue"
    };

    spawn_level_event_writer.send(SpawnLevel {
        polygon: "cat2".to_string(),
        target: "shark1".to_string(),
        target_multiplier: 1.1,
        number_of_cuts: 1000,
    });

    // let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font = asset_server.load("fonts/poly.ttf");

    let fonts = FontHandles { font: font.clone() };
    commands.insert_resource(fonts);

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
        font_size: 60.0,
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
        .insert(OnMainMenuScreen)
        .with_children(|parent| {
            // Display the game name
            parent.spawn_bundle(
                TextBundle::from_section(
                    "Menu",
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
                .insert(MenuButtonAction::Play)
                .with_children(|parent| {
                    // let icon = asset_server.load("textures/Game Icons/right.png");
                    // parent.spawn_bundle(ImageBundle {
                    //     style: button_icon_style.clone(),
                    //     image: UiImage(icon),
                    //     ..default()
                    // });
                    parent.spawn_bundle(TextBundle::from_section(
                        new_game_label,
                        button_text_style.clone(),
                    ));
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(MenuButtonAction::GoToCity)
                .with_children(|parent| {
                    // let icon = asset_server.load("textures/Game Icons/right.png");
                    // parent.spawn_bundle(ImageBundle {
                    //     style: button_icon_style.clone(),
                    //     image: UiImage(icon),
                    //     ..default()
                    // });
                    parent.spawn_bundle(TextBundle::from_section(
                        "Go to city",
                        button_text_style.clone(),
                    ));
                });
        });
}

// #[derive(Component)]
// pub struct LockIcon;

fn settings_menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    unlocked_cities: Res<levels::UnlockedCities>,
) {
    let button_style = Style {
        size: Size::new(Val::Px(300.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextStyle {
        // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        font: asset_server.load("fonts/poly.ttf"),
        font_size: 60.0,
        color: TEXT_COLOR,
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

    let icon = asset_server.load("textures/Game Icons/lock.png");

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
        .insert(OnSettingsMenuScreen)
        .with_children(|parent| {
            for (action, text) in [
                (MenuButtonAction::Tutorial, "Tutorial"),
                (MenuButtonAction::Simplicity, "Simplicity"),
                (MenuButtonAction::Perplexity, "Perplexity"),
                (MenuButtonAction::Complexity, "Complexity"),
            ] {
                let button_bundle = ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..default()
                };

                let mut buttons_spawner = parent.spawn_bundle(button_bundle);
                buttons_spawner.insert(action);

                let mut is_active = true;
                match text {
                    "Tutorial" => {
                        if !unlocked_cities.cities.contains(&levels::City::Tutorial) {
                            buttons_spawner.insert(Inactive);
                            is_active = false;
                        }
                    }
                    "Simplicity" => {
                        if !unlocked_cities.cities.contains(&levels::City::Simplicity) {
                            buttons_spawner.insert(Inactive);
                            is_active = false;
                        }
                    }
                    "Perplexity" => {
                        if !unlocked_cities.cities.contains(&levels::City::Perplexity) {
                            buttons_spawner.insert(Inactive);
                            is_active = false;
                        }
                    }
                    "Complexity" => {
                        if !unlocked_cities.cities.contains(&levels::City::Complexity) {
                            buttons_spawner.insert(Inactive);
                            is_active = false;
                        }
                    }
                    _ => {}
                };

                buttons_spawner.with_children(|parent2| {
                    if !is_active {
                        parent2.spawn_bundle(ImageBundle {
                            style: button_icon_style.clone(),
                            image: UiImage(icon.clone()),
                            ..default()
                        });
                    }

                    parent2.spawn_bundle(TextBundle::from_section(text, button_text_style.clone()));
                });
            }

            // parent
            //     .spawn_bundle(ButtonBundle {
            //         style: button_style.clone(),
            //         color: NORMAL_BUTTON.into(),
            //         ..default()
            //     })
            //     .insert(MenuButtonAction::Design)
            //     .with_children(|parent2| {
            //         parent2.spawn_bundle(TextBundle::from_section(
            //             "Design level",
            //             button_text_style.clone(),
            //         ));
            //     });

            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(MenuButtonAction::BackToMainMenu)
                .with_children(|parent2| {
                    parent2
                        .spawn_bundle(TextBundle::from_section("Back", button_text_style.clone()));
                });
        });
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>, Without<Inactive>),
    >,
    // mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<State<MenuState>>,
    mut game_state: ResMut<State<GameState>>,
    music_controller: Res<MusicController>,
    audio_sinks: Res<Assets<AudioSink>>,
    mut current_level: ResMut<crate::levels::CurrentLevel>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                // MenuButtonAction::Quit => app_exit_events.send(AppExit),
                MenuButtonAction::Play => {
                    // game_state.set(GameState::Game).unwrap();
                    game_state.set(GameState::CityTitle).unwrap();
                    menu_state.set(MenuState::Disabled).unwrap();
                    if let Some(sink) = audio_sinks.get(&music_controller.0) {
                        sink.stop();
                    }
                }
                MenuButtonAction::GoToCity => menu_state.set(MenuState::Settings).unwrap(),

                MenuButtonAction::Tutorial => {
                    current_level.level = crate::levels::Level::Tutorial(0);
                    // game_state.set(GameState::Game).unwrap();
                    game_state.set(GameState::CityTitle).unwrap();
                    menu_state.set(MenuState::Disabled).unwrap();
                    if let Some(sink) = audio_sinks.get(&music_controller.0) {
                        sink.stop();
                    }
                }

                MenuButtonAction::Simplicity => {
                    current_level.level = crate::levels::Level::Simplicity(0);
                    // game_state.set(GameState::Game).unwrap();
                    game_state.set(GameState::CityTitle).unwrap();
                    menu_state.set(MenuState::Disabled).unwrap();
                    if let Some(sink) = audio_sinks.get(&music_controller.0) {
                        sink.stop();
                    }
                }

                // MenuButtonAction::Convexity => {
                //     current_level.level = crate::levels::Level::Convexity(0);
                //     // game_state.set(GameState::Game).unwrap();
                //     game_state.set(GameState::CityTitle).unwrap();
                //     menu_state.set(MenuState::Disabled).unwrap();
                //     if let Some(sink) = audio_sinks.get(&music_controller.0) {
                //         sink.stop();
                //     }
                // }
                MenuButtonAction::Perplexity => {
                    current_level.level = crate::levels::Level::Perplexity(0);
                    game_state.set(GameState::CityTitle).unwrap();
                    menu_state.set(MenuState::Disabled).unwrap();
                    if let Some(sink) = audio_sinks.get(&music_controller.0) {
                        sink.stop();
                    }
                }

                MenuButtonAction::Complexity => {
                    current_level.level = crate::levels::Level::Complexity(0);
                    game_state.set(GameState::CityTitle).unwrap();
                    menu_state.set(MenuState::Disabled).unwrap();
                    if let Some(sink) = audio_sinks.get(&music_controller.0) {
                        sink.stop();
                    }
                }

                // MenuButtonAction::Design => {
                //     game_state.set(GameState::Design).unwrap();
                //     menu_state.set(MenuState::Disabled).unwrap();
                //     // info!("menu state: {:?}", menu_state);
                // }
                MenuButtonAction::BackToMainMenu => {
                    if let Some(sink) = audio_sinks.get(&music_controller.0) {
                        sink.play();
                    }
                    menu_state.set(MenuState::Main).unwrap();
                }
            }
        }
    }
}
