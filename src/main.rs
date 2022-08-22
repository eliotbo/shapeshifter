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
        .add_system(transform_poly.exclusive_system().at_end())
        .run();
}

pub fn setup_mesh(mut load_event_writer: EventWriter<Load>) {
    // load_event_writer.send(Load("my_mesh7".to_string()));
    load_event_writer.send(Load("my_mesh6".to_string()));
    // load_event_writer.send(Load("my_mesh5".to_string()));
}
