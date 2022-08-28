use crate::game_spawn::*;
// use crate::levels::send_tutorial_text;
use crate::levels::*;

use bevy::audio::AudioSink;
use bevy::{prelude::*, utils::Duration};

use shapeshifter_level_maker::util::{
    HasWonLevelEvent, PerformedCut, PolyIsInsideTarget, Polygon, RemainingCuts, SpawnLevel, Target,
};

use super::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameLevels::default())
            .insert_resource(CurrentLevel {
                level: Level::Tutorial(0),
            })
            .insert_resource(UnlockedLevels { levels: Vec::new() })
            .insert_resource(UnlockedCities {
                cities: vec![City::Tutorial],
            })
            .insert_resource(WholeGameCuts { cuts: 0 })
            .init_resource::<WinSoundTimer>()
            .init_resource::<CityTitleTimer>()
            .add_event::<NextLevel>()
            .add_event::<PreviousLevel>()
            .add_event::<WonTheGame>()
            .add_event::<TogglePauseMenu>()
            .add_event::<SpawnNextLevelButton>()
            .add_event::<SpawnInstruction>()
            // .add_event::<SpawnCityTitle>()
            .add_system_set(SystemSet::on_enter(GameState::CityTitle).with_system(spawn_city_title))
            .add_system_set(SystemSet::on_update(GameState::CityTitle).with_system(despawn_city))
            .add_system_set(SystemSet::on_exit(GameState::Game).with_system(delete_game_entities))
            // .add_system_set(SystemSet::on_enter(GameState::Victory).with_system())
            .add_system_set(
                SystemSet::on_enter(GameState::Game)
                    .with_system(game_setup)
                    .with_system(spawn_options_button)
                    .with_system(spawn_current_level)
                    .with_system(spawn_remaining_cuts_label),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(crate::menu::button_system)
                    .with_system(spawn_won_screen)
                    .with_system(spawn_next_level_button)
                    .with_system(spawn_pause_menu)
                    .with_system(show_current_level_int)
                    .with_system(next_level)
                    .with_system(win_sound)
                    .with_system(spawn_level_adjustments)
                    .with_system(spawn_instruction)
                    .with_system(previous_level)
                    .with_system(inscrease_total_cuts)
                    .with_system(next_button_action)
                    .with_system(force_next_level)
                    .with_system(show_cuts_label)
                    .with_system(show_pause_menu)
                    .with_system(play_inside_target_sound)
                    .with_system(activate_next_level_button),
            );
    }
}

#[derive(Default)]
pub struct WinSoundTimer {
    pub maybe_timer: Option<Timer>,
}

pub struct WholeGameCuts {
    pub cuts: usize,
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
    OptionsMenu,
}

fn game_setup(
    mut spawn_level_event_writer: EventWriter<SpawnLevel>,
    game_levels: Res<GameLevels>,
    mut spawn_instruction_event_writer: EventWriter<SpawnInstruction>,
    current_level: Res<CurrentLevel>,
) {
    let spawn_level = game_levels.get(&current_level.level);
    spawn_level_event_writer.send(spawn_level.clone());

    match current_level.level {
        Level::Tutorial(0) => {
            send_tutorial_text(0, &mut spawn_instruction_event_writer);
        }
        _ => {}
    }
}

fn show_current_level_int(
    mut query: Query<&mut Text, With<LevelInt>>,
    game_levels: Res<GameLevels>,
    current_level: Res<CurrentLevel>,
) {
    if current_level.is_changed() {
        for mut text in query.iter_mut() {
            // let level_int = game_levels.to_int(&current_level.level.clone());

            if let Some(mut section) = text.sections.get_mut(0) {
                // info!("level_int: {:?}", &current_level.level.clone());
                let level_int = game_levels.to_int(&current_level.level.clone()) + 1;
                let label = format!("Level {} / {}", level_int, game_levels.get_total_levels());
                section.value = label;
            }
        }
    }
}

fn show_cuts_label(
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

fn inscrease_total_cuts(
    mut performed_cut_event_reader: EventReader<PerformedCut>,
    mut whole_game_cuts: ResMut<WholeGameCuts>,
    sound_map: Res<crate::menu::SoundMap>,
    audio: Res<Audio>,
) {
    for _ in performed_cut_event_reader.iter() {
        whole_game_cuts.cuts += 1;
        sound_map.play("cut", &audio);
    }
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
            With<Instruction>,
            With<OptionButton>,
            With<LevelInt>,
        )>,
    >,
    // mut current_level: ResMut<CurrentLevel>,
) {
    // current_level.simplicity(0);
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn play_inside_target_sound(
    audio: Res<Audio>,
    sound_map: Res<crate::menu::SoundMap>,
    mut poly_inside_target_event_reader: EventReader<PolyIsInsideTarget>,
) {
    for _ in poly_inside_target_event_reader.iter() {
        sound_map.play("target", &audio);
    }
}

fn show_pause_menu(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<Entity, With<PauseMenu>>,
    mut spawn_pause_menu_event_writer: EventWriter<TogglePauseMenu>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) || keyboard_input.just_pressed(KeyCode::M) {
        if query.iter().count() == 0 {
            spawn_pause_menu_event_writer.send(TogglePauseMenu);
        } else {
            for entity in query.iter_mut() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

// HERE
fn spawn_level_adjustments(
    mut commands: Commands,
    mut remaining_cuts: ResMut<RemainingCuts>,

    mut spawn_level_event_reader: EventReader<SpawnLevel>,
    query: Query<Entity, With<Instruction>>,

    unlocked_levels: Res<UnlockedLevels>,
    current_level: Res<CurrentLevel>,
    mut spawn_next_level_button_event_writer: EventWriter<SpawnNextLevelButton>,
) {
    for level in spawn_level_event_reader.iter() {
        info!("spawn_level_adjustments: {:?}", level);
        remaining_cuts.remaining = level.number_of_cuts;
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        if unlocked_levels
            .levels
            .contains(&current_level.level.clone())
        {
            spawn_next_level_button_event_writer.send(SpawnNextLevelButton);
        }
    }
}

fn next_level(
    // mut commands: Commands,
    game_levels: ResMut<GameLevels>,
    // mut unlocked_levels: ResMut<UnlockedLevels>,
    mut next_level_event_reader: EventReader<NextLevel>,
    mut current_level: ResMut<CurrentLevel>,

    mut won_the_game_event_writer: EventWriter<WonTheGame>,
    mut spawn_level_event_writer: EventWriter<SpawnLevel>,
    mut spawn_instruction_event_writer: EventWriter<SpawnInstruction>,
    mut unlocked_cities: ResMut<UnlockedCities>,
    mut game_state: ResMut<State<crate::GameState>>,
    // mut spawn_city_title_event_writer: EventWriter<SpawnCityTitle>,
) {
    if let Some(_) = next_level_event_reader.iter().next() {
        match current_level.level {
            //
            //
            //
            //
            //
            Level::Tutorial(level) => {
                //
                if level < game_levels.tutorial.len() - 1 {
                    current_level.level.tutorial(level + 1);
                    spawn_level_event_writer.send(game_levels.tutorial[level + 1].clone());
                    send_tutorial_text(level + 1, &mut spawn_instruction_event_writer);

                    //  info!("next_level: {:?}", current_level);

                    //
                } else {
                    if let Some(_) = game_levels.simplicity.get(0) {
                        current_level.level.simplicity(0);
                        unlocked_cities.cities.push(City::Simplicity);
                        game_state.set(crate::GameState::CityTitle).unwrap();

                        // spawn_city_title_event_writer.send(SpawnCityTitle {
                        //     city: City::Convexity,
                        // });

                        // spawn_level_event_writer.send(game_levels.convexity[0].clone());
                    } else {
                        // should never occur
                        won_the_game_event_writer.send(WonTheGame);
                    }
                }
            }

            Level::Simplicity(level) => {
                if level < game_levels.simplicity.len() - 1 {
                    current_level.level.simplicity(level + 1);
                    spawn_level_event_writer.send(game_levels.simplicity[level + 1].clone());
                    //
                    //
                } else {
                    if let Some(_) = game_levels.convexity.get(0) {
                        current_level.level.convexity(0);
                        // spawn_level_event_writer.send(level.clone());
                        unlocked_cities.cities.push(City::Convexity);
                        game_state.set(crate::GameState::CityTitle).unwrap();
                    } else {
                        // should never occur
                        won_the_game_event_writer.send(WonTheGame);
                    }
                }
            }
            //
            //
            //
            //
            //
            Level::Convexity(level) => {
                if level < game_levels.convexity.len() - 1 {
                    current_level.level.convexity(level + 1);
                    spawn_level_event_writer.send(game_levels.convexity[level + 1].clone());
                    //
                    //
                } else {
                    if let Some(level) = game_levels.perplexity.get(0) {
                        current_level.level.perplexity(0);
                        // spawn_level_event_writer.send(level.clone());
                        unlocked_cities.cities.push(City::Perplexity);
                        game_state.set(crate::GameState::CityTitle).unwrap();
                    } else {
                        // should never occur
                        won_the_game_event_writer.send(WonTheGame);
                    }
                }
            }
            //
            //
            Level::Perplexity(level) => {
                if level < game_levels.perplexity.len() - 1 {
                    current_level.level.perplexity(level + 1);
                    spawn_level_event_writer.send(game_levels.perplexity[level + 1].clone());
                } else {
                    current_level.level.complexity(0);
                    // spawn_level_event_writer.send(game_levels.complexity[0].clone());
                    if let Some(level) = game_levels.complexity.get(0) {
                        // spawn_level_event_writer.send(level.clone());
                        unlocked_cities.cities.push(City::Complexity);
                        game_state.set(crate::GameState::CityTitle).unwrap();
                    } else {
                        // should never occur
                        won_the_game_event_writer.send(WonTheGame);
                    }
                }
            }
            Level::Complexity(level) => {
                if level < game_levels.complexity.len() - 1 {
                    current_level.level.complexity(level + 1);
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
        match current_level.level {
            Level::Tutorial(level) => {
                if level > 0 {
                    current_level.level.tutorial(level - 1);
                    spawn_level_event_writer.send(game_levels.tutorial[level - 1].clone());
                } // do nothing if we're at the first level
            }
            Level::Simplicity(level) => {
                if level > 0 {
                    current_level.level.simplicity(level - 1);
                    spawn_level_event_writer.send(game_levels.simplicity[level - 1].clone());
                } else {
                    current_level.level.tutorial(game_levels.tutorial.len() - 1);
                    spawn_level_event_writer
                        .send(game_levels.tutorial[game_levels.tutorial.len() - 1].clone());
                }
            }
            Level::Convexity(level) => {
                if level > 0 {
                    current_level.level.convexity(level - 1);
                    spawn_level_event_writer.send(game_levels.convexity[level - 1].clone());
                } else {
                    current_level
                        .level
                        .simplicity(game_levels.simplicity.len() - 1);
                    spawn_level_event_writer
                        .send(game_levels.simplicity[game_levels.simplicity.len() - 1].clone());
                }
            }
            Level::Perplexity(level) => {
                if level > 0 {
                    current_level.level.perplexity(level - 1);
                    spawn_level_event_writer.send(game_levels.perplexity[level - 1].clone());
                } else {
                    current_level
                        .level
                        .convexity(game_levels.convexity.len() - 1);
                    spawn_level_event_writer
                        .send(game_levels.convexity[game_levels.convexity.len() - 1].clone());
                }
            }
            Level::Complexity(level) => {
                if level > 0 {
                    current_level.level.complexity(level - 1);
                    spawn_level_event_writer.send(game_levels.complexity[level - 1].clone());
                } else {
                    current_level
                        .level
                        .perplexity(game_levels.perplexity.len() - 1);
                    spawn_level_event_writer
                        .send(game_levels.perplexity[game_levels.perplexity.len() - 1].clone());
                }
            }
        }
    }
}

fn force_next_level(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut next_level_event_writer: EventWriter<NextLevel>,
    mut previous_level_event_writer: EventWriter<PreviousLevel>,
    next_button_query: Query<Entity, With<NextButtonParent>>,
) {
    if keyboard_input.just_pressed(KeyCode::Right) {
        next_level_event_writer.send(NextLevel);

        for entity in next_button_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
    if keyboard_input.just_pressed(KeyCode::Left) {
        previous_level_event_writer.send(PreviousLevel);
        for entity in next_button_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
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
    mut spawn_pause_menu_event_writer: EventWriter<TogglePauseMenu>,
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
                    let spawn_level = game_levels.get(&current_level.level.clone());
                    spawn_level_event_writer.send(spawn_level);
                } // _ => {}
                GameButtonAction::OptionsMenu => {
                    //
                    spawn_pause_menu_event_writer.send(TogglePauseMenu);
                    //
                    //
                    //
                    //
                    // Upon pressing options, the effects below should not occur
                    return;
                    // spawn_pause_menu_event_writer.send(TogglePauseMenu);
                }
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

fn win_sound(
    mut win_sound_timer: ResMut<WinSoundTimer>,
    audio: Res<Audio>,
    sound_map: Res<crate::menu::SoundMap>,
    time: Res<Time>,
) {
    if let Some(ref mut timer) = win_sound_timer.maybe_timer {
        if timer.tick(time.delta()).just_finished() {
            sound_map.play("victory", &audio);
        }
    }
}

fn activate_next_level_button(
    mut commands: Commands,
    // mut win_sound_timer: ResMut<WinSoundTimer>,
    mut has_won_event_reader: EventReader<HasWonLevelEvent>,
    mut go_next_button_query: Query<(Entity, &mut Visibility), With<Button>>,
    mut spawn_next_level_button_event_writer: EventWriter<SpawnNextLevelButton>,
    mut unlocked_levels: ResMut<UnlockedLevels>,
    current_level: Res<CurrentLevel>,
) {
    //
    if let Some(_) = has_won_event_reader.iter().next() {
        spawn_next_level_button_event_writer.send(SpawnNextLevelButton);

        //
        //
        // activate win sound timer
        // win_sound_timer.timer = Timer::new(Duration::from_millis(300), false);
        //
        //

        if !unlocked_levels
            .levels
            .contains(&current_level.level.clone())
        {
            unlocked_levels.levels.push(current_level.level.clone());

            commands.insert_resource(WinSoundTimer {
                maybe_timer: Some(Timer::new(Duration::from_millis(500), false)),
            });
        }

        for (entity, mut vis) in go_next_button_query.iter_mut() {
            vis.is_visible = true;
            commands.entity(entity).remove::<super::menu::Inactive>();
        }
    }
}
