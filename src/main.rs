pub mod cam;
use cam::*;

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

use shapeshifter_level_maker::{input::Action, ShapeshifterLevelMakerPlugin};

use std::path::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "pen".to_string(),
            width: 1200.,
            height: 800.,
            present_mode: bevy::window::PresentMode::Immediate,
            ..Default::default()
        })
        //
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        // .add_startup_system(setup)
        .add_startup_system(camera_setup)
        .add_startup_system(setup)
        .add_startup_system(read_all_pts)
        .add_plugin(CamPlugin)
        .add_plugin(ShapeshifterLevelMakerPlugin)
        .run();
}
//

pub fn read_all_pts(mut action_event_writer: EventWriter<Action>) {
    let mut save_prepath = std::env::current_dir().unwrap();
    save_prepath.push("assets/meshes/");

    for (idx, entry) in std::fs::read_dir(save_prepath).unwrap().enumerate() {
        if idx < 4 {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_str().unwrap();
            if path_str.ends_with(".pts") {
                // let save_mesh_meta = read_save_mesh_meta(path_str);
                let name_of_file = path_str.split("/").last().unwrap();
                let name_of_file = name_of_file.split(".").next().unwrap();
                println!("{:?}", name_of_file);

                action_event_writer.send(Action::QuickLoad {
                    maybe_name: Some(name_of_file.to_string()),
                });
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut action_event_writer: EventWriter<Action>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    action_event_writer.send(Action::QuickLoad {
        maybe_name: Some("004_simplicity_square_parallel".to_string()),
    });
    action_event_writer.send(Action::QuickLoadTarget {
        maybe_name: Some("004_simplicity_square_parallel".to_string()),
    });
    action_event_writer.send(Action::ToggleGrid); //

    // commands.spawn_bundle(MaterialMesh2dBundle {
    //     mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
    //     transform: Transform::default().with_scale(Vec3::splat(128.)),
    //     material: materials.add(ColorMaterial::from(Color::PURPLE)),
    //     ..default()
    // });
}
