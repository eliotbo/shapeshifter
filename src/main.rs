// TODO: delete this example

mod cam;
pub mod cut;
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
use cut::*;
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
        //
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        //
        // .add_event::<StartMakingPolygon>()
        .add_event::<StartMakingSegment>()
        // .add_event::<EndSegment>()
        // .add_event::<EndMakingPolygon>()
        // .add_event::<DeleteEvent>()
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
        .add_startup_system(camera_setup)
        .add_startup_system(setup_mesh)
        .add_system(delete_making_polygon)
        .add_system(end_polygon)
        .add_system(start_polygon)
        .add_system(record_mouse_events_system.exclusive_system().at_start())
        .add_system(direct_make_polygon_action)
        .add_system(making_segment)
        .add_system(end_segment)
        .add_system(start_poly_segment)
        .add_system(quick_load_mesh)
        .add_system(quick_save)
        .add_system(glow_poly)
        .add_system(perform_cut)
        .add_system(transform_poly.exclusive_system().at_end())
        // .add_system(save_mesh)
        .run();
}

pub fn setup_mesh(mut load_event_writer: EventWriter<Load>) {
    load_event_writer.send(Load("my_mesh7".to_string()));
    load_event_writer.send(Load("my_mesh6".to_string()));
    // load_event_writer.send(Load("my_mesh6".to_string()));

    // mut commands: Commands,
    // // mut action_event_writer: EventWriter<Action>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    // mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,

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

pub enum PossibleMoves {
    Translation(Vec2),
    Rotation(f32),
}
// use core::num::PI;
// use lyon::geom::{Rotation, Translation};

// make polygon glow upon hover and insert Rotating (right mouse click) or
// Translating (left mouse click) component
pub fn glow_poly(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
    query: Query<
        (
            Entity,
            &Handle<FillMesh2dMaterial>,
            &Transform,
            &MeshMeta,
            Option<Or<(&Rotating, &Translating)>>,
        ),
        With<Polygon>,
    >,
    mut materials: ResMut<Assets<FillMesh2dMaterial>>,
) {
    let left_mouse_click = mouse_button_input.just_pressed(MouseButton::Left);
    let right_mouse_click = mouse_button_input.just_pressed(MouseButton::Right);
    // let mut moving_entity = None;
    let mut maybe_highlight_entity = None;

    let mut maybe_move_entity: Option<(Entity, PossibleMoves)> = None;

    for (entity, material_handle, transform, mesh_meta, maybe_moving) in query.iter() {
        //
        //
        //
        // The path is by default centered at the origin, so we need to translate it to the
        // position of the entity.
        // let path = mesh_meta.path.clone();
        // let (axis, transform_rotation_angle) = transform.rotation.to_axis_angle();
        // let angle = axis.z * transform_rotation_angle;

        // let rot = lyon::geom::Rotation::radians(angle);
        // let translation =
        //     lyon::geom::Translation::new(transform.translation.x, transform.translation.y);

        // // the points are at the origin, so we need to take the translation + rotation into account
        // let transformed_path = path.transformed(&rot).transformed(&translation);

        let (transformed_path, angle) = transform_path(&mesh_meta.path, transform);
        //
        //
        // The path is now translated and rotated. We can now check whether the mouse in inside the path

        let is_inside_poly = hit_test_path(
            &cursor.clone().into(),
            transformed_path.iter(),
            FillRule::EvenOdd,
            0.1,
        );

        let mut material = materials.get_mut(&material_handle).unwrap();
        material.show_com = 0.0;

        if is_inside_poly && left_mouse_click {
            maybe_move_entity = Some((
                entity,
                PossibleMoves::Translation(transform.translation.truncate()),
            ));
        }

        if is_inside_poly && right_mouse_click {
            maybe_move_entity = Some((entity, PossibleMoves::Rotation(angle)));
        }

        if let Some(_) = maybe_moving {
            maybe_highlight_entity = Some(entity);
        } else if is_inside_poly {
            maybe_highlight_entity = Some(entity);
        }
    }

    // add Rotating or Translating component to clicked entity
    if let Some((entity, moves)) = maybe_move_entity {
        let (_, material_handle, _, _, _) = query.get(entity).unwrap();
        let mut material = materials.get_mut(&material_handle).unwrap();
        match moves {
            PossibleMoves::Translation(translation) => {
                commands.entity(entity).insert(Translating {
                    starting_pos: translation,
                });
            }
            PossibleMoves::Rotation(angle) => {
                commands.entity(entity).insert(Rotating {
                    starting_angle: angle,
                });
            }
        }

        material.show_com = 1.0;
    } else if let Some(highlighted_entity) = maybe_highlight_entity {
        //
        // if no movement is happening, highlight one entity that is hovered over
        if let Ok((_, material_handle, _, _, _)) = query.get(highlighted_entity) {
            let mut material = materials.get_mut(&material_handle).unwrap();
            material.show_com = 1.0;
        }
    }
}

// translate and rotate Polygon using right mouse button
pub fn transform_poly(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
    mut queries: ParamSet<(
        Query<(Entity, &mut Transform, &Rotating), With<Polygon>>,
        Query<(Entity, &mut Transform, &Translating), With<Polygon>>,
    )>,
) {
    for (_, mut transform, rotating) in queries.p0().iter_mut() {
        // println!("rotating");
        let diag_mouse_dist = cursor.position.y + cursor.position.x
            - cursor.last_right_click_position.y
            - cursor.last_right_click_position.x;
        // latch the final angle to fixed angles at every pi/25 radians
        let free_angle = -diag_mouse_dist * 0.0035 + rotating.starting_angle;
        let angle =
            (free_angle / (core::f32::consts::PI / 25.0)).round() * (core::f32::consts::PI / 25.0);
        transform.rotation = Quat::from_rotation_z(angle);
    }

    for (_, mut transform, translating) in queries.p1().iter_mut() {
        // println!("rotating");
        let mouse_delta = cursor.position - cursor.last_click_position;
        transform.translation =
            (translating.starting_pos + mouse_delta).extend(transform.translation.z);
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        // remove Translating
        for (entity, _, _) in queries.p1().iter_mut() {
            commands.entity(entity).remove::<Translating>();
        }
    }

    if mouse_button_input.just_released(MouseButton::Right) {
        // remove Rotating
        for (entity, _, _) in queries.p0().iter_mut() {
            commands.entity(entity).remove::<Rotating>();
        }
    }
}
