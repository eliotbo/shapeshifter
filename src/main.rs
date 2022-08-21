// TODO: delete this example

mod cam;
pub mod input;
mod io;
pub mod material;
mod poly;
pub mod util;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};
use cam::*;
use input::*;
use io::*;
use material::*;
use poly::*;
use util::*;

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use lyon::algorithms::raycast::*;
use lyon::path::path::BuilderImpl;
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::{builder::NoAttributes, Path};
use lyon::tessellation::{FillOptions, FillTessellator, VertexBuffers};

use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_obj::*;

use rand::{thread_rng, Rng};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "pen".to_string(),
            width: 1200.,
            height: 800.,
            // vsync: true,
            ..Default::default()
        })
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_event::<StartMakingPolygon>()
        .add_event::<StartMakingSegment>()
        .add_event::<EndSegment>()
        .add_event::<EndMakingPolygon>()
        .add_event::<DeleteEvent>()
        .add_event::<QuickLoad>()
        .add_event::<SaveMeshEvent>()
        .insert_resource(Globals::default())
        .insert_resource(Cursor::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(CamPlugin)
        .add_plugin(FillMesh2dPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(ObjPlugin)
        .add_startup_system(camera_setup)
        .add_startup_system(setup_mesh)
        .add_system(end_polygon)
        .add_system(start_polygon)
        .add_system(record_mouse_events_system)
        .add_system(direct_make_polygon_action)
        .add_system(making_segment)
        .add_system(end_segment)
        .add_system(start_poly_segment)
        .add_system(quick_load_mesh)
        .add_system(quick_save)
        // .add_system(save_mesh)
        .run();
}

pub fn setup_mesh(
    mut commands: Commands,
    // mut action_event_writer: EventWriter<Action>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    globals: Res<Globals>,
) {
    // action_event_writer.send(Action::QuickLoad);

    /////////////////////////// cutting segment /////////////////////////////
    let segment = Segment {
        start: point(-200.0, 0.0),
        end: point(200.0, 200.0),
    };

    let segment_meta = get_segment_meta(segment);

    let ends_mesh_handle = bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(
        Vec2::new(segment_meta.length, globals.cutting_segment_thickness),
    ))));

    let id = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: ends_mesh_handle.clone(),
            material: materials.add(ColorMaterial::from(globals.cutting_segment_color)),
            transform: segment_meta.transform,
            ..Default::default()
        })
        .id();
    /////////////////////////// cutting segment /////////////////////////////

    /////////////////////////// polygon /////////////////////////////
    let (path, points) = make_square();
    let color = Color::RED;
    let (mesh, center_of_mass) = make_polygon_mesh(&path, &color);

    let fill_transform = Transform::from_translation(center_of_mass.extend(0.0));

    let mut rng = thread_rng();

    // Useless at the moment, but here for future use
    let mat_handle = fill_materials.add(FillMesh2dMaterial {
        color: color.into(),
        show_com: 0.0, // show center of mass
    });

    let id = rng.gen::<u64>();
    let entity = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(mesh)),
            material: mat_handle,
            transform: fill_transform,
            ..default()
        })
        .insert(MeshMeta {
            id,
            path: path.clone(),
            points,
        })
        .id();
}
