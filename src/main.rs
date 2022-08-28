mod design;
mod game;
mod game_spawn;
mod levels;
mod menu;
mod splash;

use bevy::prelude::*;
use shapeshifter_level_maker::ShapeshifterLevelMakerPlugin;

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const BG_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

// Enum that will be used as a global state for the game
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Splash,
    Menu,
    Game,
    CityTitle,
    Design,
}

// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Debug, Component, PartialEq, Eq, Clone, Copy)]
struct Volume(u32);
// 1280x720
fn main() {
    App::new()
        //
        //
        //
        // .insert_resource(WindowDescriptor {
        //     title: "pen".to_string(),
        //     width: 1280.,
        //     height: 720.,
        //     position: WindowPosition::At(Vec2::new(500., 200.)),
        //     present_mode: bevy::window::PresentMode::Immediate,
        //     ..Default::default()
        // })
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        //
        //
        //
        .insert_resource(ClearColor(BG_COLOR))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapeshifterLevelMakerPlugin)
        // .add_plugin(CamPlugin)
        // Insert as resource the initial value for the settings resources
        // .add_startup_system(camera_setup)
        .add_startup_system(setup)
        // Declare the game state, and set its startup value
        .add_state(GameState::Splash)
        // Adds the plugins for each state
        .add_plugin(splash::SplashPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(design::DesignPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

// mod game {

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
