// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

use shapeshifter_level_maker::{
    input::Action,
    material::FillMesh2dMaterial,
    util::{Globals, LoadedPolygonsRaw, MeshMeta, SpawnPoly, SpawnTarget},
    ShapeshifterLevelMakerPlugin,
};

use std::collections::HashMap;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "miport_pts".to_string(),
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
        .add_startup_system(setup)
        .add_startup_system(read_all_pts)
        //
        .add_system(setup_polygons)
        .add_system(build_ptsmap)
        .add_plugin(ShapeshifterLevelMakerPlugin)
        .run();
}
//

#[derive(Debug)]
pub struct PTS {
    pub points: Vec<Vec2>,
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
    mut action_event_writer: EventWriter<Action>,
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
                points: mesh_meta.points.clone(),
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

        for (name, pts) in pts_map.map.iter_mut() {
            // normalize the area

            let area_sqrt = pts.area.sqrt();
            // let new_points: Vec<Vec2> = pts.points.iter().map(|pt| *pt / area_sqrt).collect();
            let mut new_points = vec![];

            let mut center_of_mass = Vec2::ZERO;
            for pt in pts.points.iter() {
                center_of_mass += *pt;
            }
            center_of_mass /= pts.points.len() as f32;

            // move points to their center of maa

            for point in pts.points.iter() {
                // normalize such that the new area is 100000
                new_points.push((point.clone() - center_of_mass) / area_sqrt * 50000.0_f32.sqrt());
            }

            action_event_writer.send(Action::SaveOneSent {
                name: name.clone(),
                pts: new_points,
            });
        }
    }
}

// imports all the polygons in the same level using the QuickLoad event
pub fn read_all_pts(mut commands: Commands, mut action_event_writer: EventWriter<Action>) {
    let mut save_prepath = std::env::current_dir().unwrap();
    save_prepath.push("assets/meshes/");
    info!("save_prepath: {:?}", save_prepath);

    for (idx, entry) in std::fs::read_dir(save_prepath).unwrap().enumerate() {
        // if idx < 2 {
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

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

// only runs after load_poly_wasm(..) is done
fn setup_polygons(
    loaded_polygons_raw: ResMut<LoadedPolygonsRaw>,
    mut spawn_poly_event_writer: EventWriter<SpawnPoly>,
    mut spawn_target_event_writer: EventWriter<SpawnTarget>,
) {
    //
    //
    //
    if loaded_polygons_raw.is_changed() {
        spawn_poly_event_writer.send(SpawnPoly {
            polygon: "004_simplicity_square_parallel".to_string(),
            polygon_multiplier: 1.0,
        });

        spawn_target_event_writer.send(SpawnTarget {
            target: "004_simplicity_square_parallel".to_string(),
            target_multiplier: 1.1,
        });
    }
}
