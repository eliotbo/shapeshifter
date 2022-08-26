use crate::levels::*;
// use crate::spawn::*;
use crate::game_spawn::*;
// use crate::levels::send_tutorial_text;

// use bevy::audio::AudioSink;
use bevy::prelude::*;

use shapeshifter_level_maker::util::{
    HasWonLevelEvent, PerformedCut, Polygon, RemainingCuts, SpawnLevel, Target,
};

use super::GameState;
use super::TEXT_COLOR;

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
pub struct DesignPlugin;

impl Plugin for DesignPlugin {
    fn build(&self, app: &mut App) {
        //

        //
        app.add_system_set(
            SystemSet::on_enter(GameState::Design)
                // .with_system(design_setup)
                .with_system(spawn_shortcuts),
        )
        .add_system_set(SystemSet::on_update(GameState::Design).with_system(design_setup));
    }
}

fn design_setup(
    mut spawn_level_event_writer: EventWriter<SpawnLevel>,
    game_levels: ResMut<GameLevels>,
    mut spawn_instruction_event_writer: EventWriter<SpawnInstruction>,
) {
    // spawn_level_event_writer.send(game_levels.simplicity[5].clone());
    // send_tutorial_text(0, &mut spawn_instruction_event_writer);
    info!("design_setup");
}

pub fn spawn_shortcuts(mut commands: Commands, asset_server: Res<AssetServer>) {
    //
    info!("spawn_shortcuts");

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let text = "
move point:         Q  + left mouse
add point:          left shift + right click
select:             crtl + shift + left click
delete selected:    delete 
delete all:         crtl + shift + delete
";
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
                    top: Val::Px(50.0),
                    ..default()
                },
                ..default()
            },
            color: Color::rgba(1., 0., 0., 1.).into(),
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
