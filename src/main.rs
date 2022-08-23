// TODO: delete this example

mod cam;
pub mod cut;
pub mod input;
mod io;
pub mod material;
mod poly;
pub mod util;
pub mod view;

use bevy::prelude::*;
use cam::*;
use cut::*;
use input::*;
use io::*;
use material::*;
use poly::*;
use util::*;
use view::*;

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_obj::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "pen".to_string(),
            width: 1200.,
            height: 800.,
            // vsync: true,
            ..Default::default()
        })
        //
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        //
        .add_event::<StartMakingSegment>()
        .add_event::<Action>()
        .add_event::<QuickLoad>()
        .add_event::<Load>()
        .add_event::<SaveMeshEvent>()
        .insert_resource(Globals::default())
        .insert_resource(Cursor::default())
        .insert_resource(PolyOrder::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(CamPlugin)
        .add_plugin(FillMesh2dPlugin)
        .add_plugin(CutMesh2dPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(CutPlugin)
        .add_plugin(ObjPlugin)
        .add_plugin(PolyPlugin)
        .add_startup_system(camera_setup)
        .add_startup_system(setup_mesh)
        .add_system(record_mouse_events_system.exclusive_system().at_start())
        .add_system(direct_make_polygon_action)
        .add_system(quick_load_mesh)
        .add_system(quick_save)
        .add_system(glow_poly)
        .add_system(rotate_once)
        .add_system(select_poly)
        .add_system(delete_poly)
        .add_system(delete_all)
        .add_system(transform_poly.exclusive_system().at_end())
        .run();
}

pub fn setup_mesh(mut load_event_writer: EventWriter<Load>) {
    load_event_writer.send(Load("my_mesh0".to_string()));
    // load_event_writer.send(Load("my_mesh6".to_string()));
    // load_event_writer.send(Load("my_mesh8".to_string()));
}

use lyon::tessellation::math::Point;

pub fn delete_all(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Polygon>, With<CutSegment>)>>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::DeleteAll) = action_event_reader.iter().next() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn select_poly(
    mut commands: Commands,
    mut fill_mesh_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut query: Query<(Entity, &MeshMeta, &Transform, &Handle<FillMesh2dMaterial>), With<Polygon>>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::SelectPoly { pos, keep_selected }) = action_event_reader.iter().next() {
        if !keep_selected {
            for (entity, _, _, mat_handle) in query.iter_mut() {
                commands.entity(entity).remove::<Selected>();
                let mat = fill_mesh_materials.get_mut(mat_handle).unwrap();
                mat.selected = 0.0;
            }
        }

        for (entity, mesh_meta, transform, mat_handle) in query.iter_mut() {
            //
            let mat = fill_mesh_materials.get_mut(mat_handle).unwrap();

            if mesh_meta.hit_test(&Point::new(pos.x, pos.y), &transform).0 {
                commands.entity(entity).insert(Selected);
                mat.selected = 1.0;
                break;
            }
        }
    }
}

pub fn delete_poly(
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::DeleteSelected) = action_event_reader.iter().next() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
