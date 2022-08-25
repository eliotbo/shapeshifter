// pub mod cam;
// use cam::*;

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

use shapeshifter_level_maker::{input::Action, util::MeshMeta, ShapeshifterLevelMakerPlugin};

use std::collections::HashMap;
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
        .insert_resource(PTSMap {
            map: HashMap::new(),
        })
        .add_plugins(DefaultPlugins)
        // .add_startup_system(setup)
        // .add_startup_system(camera_setup)
        .add_startup_system(setup)
        .add_startup_system(read_all_pts)
        .add_system(build_ptsmap)
        // .add_plugin(CamPlugin)
        .add_plugin(ShapeshifterLevelMakerPlugin)
        .run();
}
//

#[derive(Debug)]
pub struct PTS {
    pub pts: Vec<Vec2>,
    pub name: String,
    pub area: f32,
    pub height: f32,
    pub width: f32,
}

pub struct PTSMap {
    pub map: HashMap<String, PTS>,
}

use lyon::algorithms::{aabb, area};

// built the PTSMap from the loaded paths
pub fn build_ptsmap(
    mut pts_map: ResMut<PTSMap>,
    query: Query<&MeshMeta>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::M) {
        let mut map = HashMap::new();
        for mesh_meta in query.iter() {
            let path = mesh_meta.path.clone();
            let area = area::approximate_signed_area(0.1, path.iter());
            let bb = aabb::bounding_box(&path);
            let height = bb.max.y - bb.min.y;
            let width = bb.max.x - bb.min.x;

            let pts = PTS {
                pts: mesh_meta.points.clone(),
                name: mesh_meta.name.clone(),
                area: area.abs(),
                height: height,
                width: width,
            };

            map.insert(mesh_meta.name.clone(), pts);
        }
        pts_map.map = map;

        info!(
            "pts map: {:#?}",
            pts_map
                .map
                .values()
                .map(|pts| (pts.name.clone(), pts.area, pts.height, pts.width))
                .collect::<Vec<_>>()
        );
    }
}

pub fn read_all_pts(mut action_event_writer: EventWriter<Action>) {
    let mut save_prepath = std::env::current_dir().unwrap();
    save_prepath.push("assets/meshes/");
    info!("save_prepath: {:?}", save_prepath);

    for (idx, entry) in std::fs::read_dir(save_prepath).unwrap().enumerate() {
        // if idx < 15 {
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
        // }
    }
}

fn setup(
    mut commands: Commands,
    mut action_event_writer: EventWriter<Action>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // insert 2d Camera
    commands.spawn_bundle(Camera2dBundle::default());

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
