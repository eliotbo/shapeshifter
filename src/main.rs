// TODO: delete this example

mod cam;
pub mod input;
mod io;
pub mod material;
mod poly;
pub mod util;

use bevy::{
    ecs::entity,
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
use lyon::algorithms::hit_test::*;
use lyon::algorithms::raycast::*;
use lyon::path::FillRule;
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
        .add_system(glow_poly)
        .add_system(rotate_poly)
        // .add_system(save_mesh)
        .run();
}

pub fn setup_mesh(
    mut commands: Commands,
    // mut action_event_writer: EventWriter<Action>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut quickload_event_writer: EventWriter<QuickLoad>,
    globals: Res<Globals>,
) {
    quickload_event_writer.send(QuickLoad);

    // // action_event_writer.send(Action::QuickLoad);

    // /////////////////////////// cutting segment /////////////////////////////
    // let segment = Segment {
    //     start: point(-200.0, 0.0),
    //     end: point(200.0, 200.0),
    // };

    // let segment_meta = get_segment_meta(segment);

    // let ends_mesh_handle = bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(
    //     Vec2::new(segment_meta.length, globals.cutting_segment_thickness),
    // ))));

    // let id = commands
    //     .spawn_bundle(MaterialMesh2dBundle {
    //         mesh: ends_mesh_handle.clone(),
    //         material: materials.add(ColorMaterial::from(globals.cutting_segment_color)),
    //         transform: segment_meta.transform,
    //         ..Default::default()
    //     })
    //     .id();
    // /////////////////////////// cutting segment /////////////////////////////
}

// use core::num::PI;
// use lyon::geom::{Rotation, Translation};

// make polygon glow upon hover
pub fn glow_poly(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
    query: Query<(Entity, &Handle<FillMesh2dMaterial>, &Transform, &MeshMeta), With<Polygon>>,
    mut materials: ResMut<Assets<FillMesh2dMaterial>>,
) {
    let left_mouse = mouse_button_input.just_pressed(MouseButton::Left);
    let right_mouse = mouse_button_input.just_pressed(MouseButton::Right);

    for (entity, material_handle, transform, mesh_meta) in query.iter() {
        let mut path = mesh_meta.path.clone();
        // info!("is clicked");
        let (_, transform_rotation_angle) = transform.rotation.to_axis_angle();

        let rot = lyon::geom::Rotation::radians(transform_rotation_angle);
        let translation =
            lyon::geom::Translation::new(transform.translation.x, transform.translation.y);

        // the points are at the origin, so we need to take the translation + rotation into account
        let transformed_path = path.transformed(&rot).transformed(&translation);

        let is_inside_poly = hit_test_path(
            &cursor.clone().into(),
            transformed_path.iter(),
            FillRule::NonZero,
            0.1,
        );

        let mut material = materials.get_mut(&material_handle).unwrap();

        if is_inside_poly {
            material.show_com = 1.0;
        } else {
            material.show_com = 0.0;
        }

        info!("is inside poly: {}", is_inside_poly);

        if is_inside_poly && right_mouse {
            commands.entity(entity).insert(Rotating {
                starting_angle: transform_rotation_angle,
            });
        }

        // material.color = Vec4::new(1.0, 0.0, 0.0, 1.0);
    }
}

#[derive(Component)]
pub struct Rotating {
    pub starting_angle: f32,
}

// rotate Polygon using right mouse button
pub fn rotate_poly(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
    mut query: Query<(Entity, &mut Transform, &Rotating), With<Polygon>>,
) {
    if mouse_button_input.pressed(MouseButton::Right) {
        for (_, mut transform, rotating) in query.iter_mut() {
            println!("rotating");
            let vertical_mouse_dist = cursor.position.y - cursor.last_right_click_position.y;
            transform.rotation =
                Quat::from_rotation_z(vertical_mouse_dist * 0.0025 + rotating.starting_angle);
        }
    }

    if mouse_button_input.just_released(MouseButton::Right) {
        // remove Rotating
        for (entity, _, _) in query.iter_mut() {
            commands.entity(entity).remove::<Rotating>();
        }
    }
}
