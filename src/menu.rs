use bevy::{app::AppExit, prelude::*};
use shapeshifter_level_maker::util::SpawnLevel;

use super::{despawn_screen, DisplayQuality, GameState, TEXT_COLOR};

// This plugin manages the menu, with 5 different screens:
// - a main menu with "New Game", "Settings", "Quit"
// - a settings menu with two submenus and a back button
// - two settings screen with a setting that can be set and a back button
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // At start, the menu is not enabled. This will be changed in `menu_setup` when
            // entering the `GameState::Menu` state.
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
            // Systems to handle the display settings screen
            .add_system_set(
                SystemSet::on_enter(MenuState::SettingsDisplay)
                    .with_system(display_settings_menu_setup),
            )
            .add_system_set(
                SystemSet::on_update(MenuState::SettingsDisplay)
                    .with_system(setting_button::<DisplayQuality>),
            )
            .add_system_set(
                SystemSet::on_exit(MenuState::SettingsDisplay)
                    .with_system(despawn_screen::<OnDisplaySettingsMenuScreen>),
            )
            // Systems to handle the sound settings screen
            // .add_system_set(
            //     SystemSet::on_enter(MenuState::SettingsSound)
            //         .with_system(sound_settings_menu_setup),
            // )
            // .add_system_set(
            //     SystemSet::on_update(MenuState::SettingsSound)
            //         .with_system(setting_button::<Volume>),
            // )
            .add_system_set(
                SystemSet::on_exit(MenuState::SettingsSound)
                    .with_system(despawn_screen::<OnSoundSettingsMenuScreen>),
            )
            // Common systems to all screens that handles buttons behaviour
            .add_system_set(
                SystemSet::on_update(GameState::Menu)
                    .with_system(menu_action)
                    .with_system(button_system),
            );
    }
}

// State used for the current menu screen
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MenuState {
    Main,
    Settings,
    SettingsDisplay,
    SettingsSound,
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

// Tag component used to tag entities added on the display settings menu screen
#[derive(Component)]
struct OnDisplaySettingsMenuScreen;

// Tag component used to tag entities added on the sound settings menu screen
#[derive(Component)]
struct OnSoundSettingsMenuScreen;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// Tag component used to mark wich setting is currently selected
#[derive(Component)]
struct SelectedOption;

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    Play,
    GoToCity,
    SettingsDisplay,
    SettingsSound,
    BackToMainMenu,
    BackToSettings,
}

// This system handles changing all buttons color based on mouse interaction
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>, Without<Inactive>),
    >,
) {
    for (interaction, mut color, selected) in &mut interaction_query {
        *color = match (*interaction, selected) {
            (Interaction::Clicked, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

// This system updates the settings when a new value for a setting is selected, and marks
// the button as the one currently selected
fn setting_button<T: Component + PartialEq + Copy>(
    interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
    mut selected_query: Query<(Entity, &mut UiColor), With<SelectedOption>>,
    mut commands: Commands,
    mut setting: ResMut<T>,
) {
    for (interaction, button_setting, entity) in &interaction_query {
        if *interaction == Interaction::Clicked && *setting != *button_setting {
            let (previous_button, mut previous_color) = selected_query.single_mut();
            *previous_color = NORMAL_BUTTON.into();
            commands.entity(previous_button).remove::<SelectedOption>();
            commands.entity(entity).insert(SelectedOption);
            *setting = *button_setting;
        }
    }
}

fn menu_setup(mut menu_state: ResMut<State<MenuState>>) {
    let _ = menu_state.set(MenuState::Main);
}

fn main_menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_level_event_writer: EventWriter<SpawnLevel>,
) {
    spawn_level_event_writer.send(SpawnLevel {
        polygon: "cat2".to_string(),
        target: "shark1".to_string(),
        target_multiplier: 1.1,
    });

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
        .insert(OnMainMenuScreen)
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
                .insert(MenuButtonAction::Play)
                .with_children(|parent| {
                    let icon = asset_server.load("textures/Game Icons/right.png");
                    parent.spawn_bundle(ImageBundle {
                        style: button_icon_style.clone(),
                        image: UiImage(icon),
                        ..default()
                    });
                    parent.spawn_bundle(TextBundle::from_section(
                        "New Game",
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

fn settings_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_style = Style {
        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
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
        .insert(OnSettingsMenuScreen)
        .with_children(|parent| {
            for (action, text) in [
                (MenuButtonAction::SettingsDisplay, "Simplicity"),
                (MenuButtonAction::SettingsSound, "Convexity"),
                (MenuButtonAction::SettingsSound, "Perplexity"),
                (MenuButtonAction::SettingsSound, "Complexity"),
                (MenuButtonAction::BackToMainMenu, "Back"),
            ] {
                parent
                    .spawn_bundle(ButtonBundle {
                        style: button_style.clone(),
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    })
                    .insert(action)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            text,
                            button_text_style.clone(),
                        ));
                    });
            }
        });
}

fn display_settings_menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    display_quality: Res<DisplayQuality>,
) {
    let button_style = Style {
        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
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
            color: Color::CRIMSON.into(),
            ..default()
        })
        .insert(OnDisplaySettingsMenuScreen)
        .with_children(|parent| {
            // Create a new `NodeBundle`, this time not setting its `flex_direction`. It will
            // use the default value, `FlexDirection::Row`, from left to right.
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: Color::CRIMSON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Display a label for the current setting
                    parent.spawn_bundle(TextBundle::from_section(
                        "Display Quality",
                        button_text_style.clone(),
                    ));
                    // Display a button for each possible value
                    for quality_setting in [
                        DisplayQuality::Low,
                        DisplayQuality::Medium,
                        DisplayQuality::High,
                    ] {
                        let mut entity = parent.spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                                ..button_style.clone()
                            },
                            color: NORMAL_BUTTON.into(),
                            ..default()
                        });
                        entity.insert(quality_setting).with_children(|parent| {
                            parent.spawn_bundle(TextBundle::from_section(
                                format!("{quality_setting:?}"),
                                button_text_style.clone(),
                            ));
                        });
                        if *display_quality == quality_setting {
                            entity.insert(SelectedOption);
                        }
                    }
                });
            // Display the back button to return to the settings screen
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style,
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(MenuButtonAction::BackToSettings)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section("Back", button_text_style));
                });
        });
}

fn sound_settings_menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // volume: Res<Volume>,
) {
    let button_style = Style {
        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
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
            color: Color::CRIMSON.into(),
            ..default()
        })
        .insert(OnSoundSettingsMenuScreen)
        .with_children(|parent| {
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::CRIMSON.into(),
                ..default()
            });

            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style,
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(MenuButtonAction::BackToSettings)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle::from_section("Back", button_text_style));
                });
        });
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    // mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<State<MenuState>>,
    mut game_state: ResMut<State<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                // MenuButtonAction::Quit => app_exit_events.send(AppExit),
                MenuButtonAction::Play => {
                    game_state.set(GameState::Game).unwrap();
                    menu_state.set(MenuState::Disabled).unwrap();
                }
                MenuButtonAction::GoToCity => menu_state.set(MenuState::Settings).unwrap(),
                MenuButtonAction::SettingsDisplay => {
                    menu_state.set(MenuState::SettingsDisplay).unwrap();
                }
                MenuButtonAction::SettingsSound => {
                    menu_state.set(MenuState::SettingsSound).unwrap();
                }
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main).unwrap(),
                MenuButtonAction::BackToSettings => {
                    menu_state.set(MenuState::Settings).unwrap();
                }
            }
        }
    }
}
