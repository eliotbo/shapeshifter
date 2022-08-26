use crate::levels::*;
// use crate::spawn::*;
use crate::spawn::*;

use bevy::audio::AudioSink;
use bevy::prelude::*;

use shapeshifter_level_maker::util::{
    HasWonLevelEvent, Polygon, RemainingCuts, SpawnLevel, Target,
};

use super::GameState;

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameLevels::default())
            .insert_resource(CurrentLevel::Simplicity(0))
            .insert_resource(UnlockedCities { cities: Vec::new() })
            .add_event::<NextLevel>()
            .add_event::<PreviousLevel>()
            .add_event::<WonTheGame>()
            .add_event::<SpawnPauseMenu>()
            .add_event::<SpawnNextLevelButton>()
            .add_system_set(SystemSet::on_exit(GameState::Game).with_system(delete_game_entities))
            .add_system_set(
                SystemSet::on_enter(GameState::Game)
                    .with_system(game_setup)
                    .with_system(spawn_remaining_cuts_label),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(spawn_next_level_button)
                    .with_system(spawn_pause_menu)
                    .with_system(next_level)
                    .with_system(spawn_level_adjust_cuts_resource)
                    .with_system(previous_level)
                    .with_system(next_button_action)
                    .with_system(force_next_level)
                    .with_system(adjust_cuts_label)
                    .with_system(show_pause_menu)
                    .with_system(activate_next_level_button),
            );

        // .add_system_set(
        //     SystemSet::on_exit(GameState::Game).with_system(despawn_screen::<OnGameScreen>),
        // );
    }
}

pub struct UnlockedCities {
    pub cities: Vec<CurrentLevel>,
}

// #[derive(Deref, DerefMut)]
// struct GameTimer(Timer);

#[derive(Clone)]
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
pub enum GameButtonAction {
    GoNext,
    Restart,
    GoBack,
    ToMenu,
}

fn delete_game_entities(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<PauseMenu>,
            With<NextButtonParent>,
            With<Target>,
            With<Polygon>,
            With<RemainingCutsComponent>,
        )>,
    >,
    mut current_level: ResMut<CurrentLevel>,
) {
    current_level.simplicity(0);
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn show_pause_menu(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<Entity, With<PauseMenu>>,
    mut spawn_pause_menu_event_writer: EventWriter<SpawnPauseMenu>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if query.iter().count() == 0 {
            spawn_pause_menu_event_writer.send(SpawnPauseMenu);
        } else {
            for entity in query.iter_mut() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn spawn_level_adjust_cuts_resource(
    mut remaining_cuts: ResMut<RemainingCuts>,

    mut spawn_level_event_reader: EventReader<SpawnLevel>,
) {
    for level in spawn_level_event_reader.iter() {
        remaining_cuts.remaining = level.number_of_cuts;
    }
}

fn adjust_cuts_label(
    remaining_cuts: ResMut<RemainingCuts>,
    mut query: Query<&mut Text, With<RemainingCutsComponent>>,
) {
    if remaining_cuts.is_changed() {
        let label = format!("Cuts: {}", remaining_cuts.remaining);
        for mut text in query.iter_mut() {
            if let Some(mut section) = text.sections.get_mut(0) {
                section.value = label.clone();
            }
        }
    }
}

fn next_level(
    // mut commands: Commands,
    game_levels: ResMut<GameLevels>,
    mut unlocked_cities: ResMut<UnlockedCities>,
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
        unlocked_cities.cities.push(current_level.clone());
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
    if keyboard_input.just_pressed(KeyCode::Left) {
        previous_level_event_writer.send(PreviousLevel);
    }
}

fn next_button_action(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &GameButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut previous_level_event_writer: EventWriter<PreviousLevel>,
    mut next_level_event_writer: EventWriter<NextLevel>,
    mut game_state: ResMut<State<GameState>>,
    game_levels: ResMut<GameLevels>,
    current_level: Res<CurrentLevel>,
    mut spawn_level_event_writer: EventWriter<SpawnLevel>,
    pause_menu_query: Query<Entity, With<PauseMenu>>,
    next_button_query: Query<Entity, With<NextButtonParent>>,
    music_controller: Res<crate::menu::MusicController>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    let mut has_despawned_next_button = false;
    for (interaction, menu_button_action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                // MenuButtonAction::Quit => app_exit_events.send(AppExit),
                GameButtonAction::GoNext => {
                    next_level_event_writer.send(NextLevel);

                    // commands.entity(entity).insert(super::menu::Inactive);
                    // vis.is_visible = false;
                }
                GameButtonAction::GoBack => {
                    previous_level_event_writer.send(PreviousLevel);
                    // commands.entity(entity).insert(super::menu::Inactive);
                    // vis.is_visible = false;
                }
                GameButtonAction::ToMenu => {
                    game_state.set(GameState::Menu).unwrap();
                    if let Some(sink) = audio_sinks.get(&music_controller.0) {
                        sink.play();
                    }
                }
                GameButtonAction::Restart => {
                    let spawn_level = game_levels.get(&current_level.clone());
                    spawn_level_event_writer.send(spawn_level);
                } // _ => {}
            }

            //
            //
            // despawn whole pause menu
            if let Some(pause_menu_entity) = pause_menu_query.iter().next() {
                commands.entity(pause_menu_entity).despawn_recursive();
            }

            if !has_despawned_next_button {
                for entity in next_button_query.iter() {
                    commands.entity(entity).despawn_recursive();
                    has_despawned_next_button = true;
                }
            }
        }
    }
}

fn activate_next_level_button(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut has_won_event_reader: EventReader<HasWonLevelEvent>,
    mut go_next_button_query: Query<(Entity, &mut Visibility), With<Button>>,
    mut spawn_next_level_button_event_writer: EventWriter<SpawnNextLevelButton>,
) {
    //
    if let Some(_) = has_won_event_reader.iter().next() {
        spawn_next_level_button_event_writer.send(SpawnNextLevelButton);

        for (entity, mut vis) in go_next_button_query.iter_mut() {
            vis.is_visible = true;
            commands.entity(entity).remove::<super::menu::Inactive>();
        }
    }
}

fn game_setup(
    mut spawn_level_event_writer: EventWriter<SpawnLevel>,
    game_levels: ResMut<GameLevels>,
) {
    spawn_level_event_writer.send(game_levels.simplicity[0].clone());
}
