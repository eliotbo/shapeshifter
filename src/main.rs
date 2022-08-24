// TODO: delete this example

mod cam;
pub mod cut;
pub mod input;
mod load;
pub mod material;
mod poly;
mod save;
pub mod target;
pub mod util;
pub mod view;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use cam::*;
use cut::*;
use input::*;
use load::*;
use material::*;
use poly::*;
use save::*;
use target::*;
use util::*;
use view::*;

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

// use bevy_inspector_egui::WorldInspectorPlugin;
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
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_event::<StartMakingSegment>()
        .add_event::<Action>()
        // .add_event::<QuickLoad>()
        .add_event::<Load>()
        .add_event::<TestCollisionEvent>()
        .add_event::<TestWinEvent>()
        .insert_resource(Globals::default())
        .insert_resource(Cursor::default())
        .add_plugin(bevy_easings::EasingsPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(CamPlugin)
        .add_plugin(FillMesh2dPlugin)
        .add_plugin(TargetMesh2dPlugin)
        .add_plugin(CutMesh2dPlugin)
        .add_plugin(LoadPlugin)
        .add_plugin(SavePlugin)
        // .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(CutPlugin)
        .add_plugin(ObjPlugin)
        .add_plugin(PolyMakerPlugin)
        .add_plugin(TargetPlugin)
        .add_startup_system(camera_setup)
        .add_startup_system(setup_mesh)
        .add_system(record_mouse_events_system.exclusive_system().at_start())
        .add_system(quick_load_target)
        .add_system(direct_action)
        .add_system(quick_load_mesh)
        .add_system(quick_save)
        .add_system(glow_poly)
        .add_system(rotate_once)
        .add_system(delete_poly)
        .add_system(toggle_grid)
        .add_system(test_collisions)
        .add_system(revert_to_init)
        //
        // delete me please
        .add_system(debug_input)
        .add_system(transform_poly.exclusive_system().at_end())
        .run();
}

pub fn setup_mesh(
    mut load_event_writer: EventWriter<Load>,
    mut action_event_writer: EventWriter<Action>,
    // mut quickload_event_writer: EventWriter<QuickLoad>,
) {
    load_event_writer.send(Load("my_mesh2".to_string()));
    // quickload_event_writer.send(QuickLoad);
    action_event_writer.send(Action::QuickLoadTarget);
}

use bevy_easings::*;

pub fn debug_input(mut action_event_reader: EventReader<Action>) {
    for action in action_event_reader.iter() {
        info!("{:?}", action)
    }
}

pub fn revert_to_init(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Polygon>, With<CutSegment>)>>,
    mut action_event_reader: EventReader<Action>,
    mut load_event_writer: EventWriter<Load>,
) {
    if let Some(Action::RevertToInit) = action_event_reader.iter().next() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        load_event_writer.send(Load("my_mesh2".to_string()));
    }
}

pub fn test_collisions(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut MeshMeta), With<Polygon>>,
    target_query: Query<&Target>,
    mut collision_test_event: EventReader<TestCollisionEvent>,
    mut check_win_condition_event: EventWriter<TestWinEvent>,
) {
    for TestCollisionEvent(entity) in collision_test_event.iter() {
        let mut do_go_back_to_previous_pos = false;
        let (transform1, meta1) = query.get(*entity).unwrap();
        for (transform2, meta2) in query.iter() {
            //
            // do not test collision with self
            if meta1.id == meta2.id {
                continue;
            }

            //
            //
            if meta1.bounding_box_collide(&meta2.path, &transform1, &transform2) {
                if meta1.precise_intersect_test(&meta2.path, &transform1, &transform2) {
                    do_go_back_to_previous_pos = true;
                }
            }
        }

        if let Some(target) = target_query.iter().next() {
            if meta1.precise_intersect_test(&target.path, &transform1, &Transform::identity()) {
                // println!("target collision");
                do_go_back_to_previous_pos = true;
            }
        }

        let (transform1, mut meta1) = query.get_mut(*entity).unwrap();

        if do_go_back_to_previous_pos {
            info!("inserting easing");
            commands.entity(*entity).insert(transform1.ease_to(
                meta1.previous_transform,
                bevy_easings::EaseFunction::BounceOut,
                bevy_easings::EasingType::Once {
                    duration: std::time::Duration::from_secs_f32(0.3),
                },
            ));
        } else {
            meta1.previous_transform = transform1.clone();
            check_win_condition_event.send(TestWinEvent);
        }
    }
}

// spawn grid at startup
pub fn toggle_grid(
    mut commands: Commands,
    mut globals: ResMut<Globals>,
    query: Query<Entity, With<Grid>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::ToggleGrid) = action_event_reader.iter().next() {
        globals.snap_to_grid = !globals.snap_to_grid;

        let num_grid_pints = 25;
        let num_grid_pints_x = 30;

        if globals.snap_to_grid {
            //
            let mesh = bevy::sprite::Mesh2dHandle(
                meshes.add(Mesh::from(shape::Quad::new(Vec2::new(3., 3.)))),
            );
            //
            //
            //
            //
            for x in -num_grid_pints_x..num_grid_pints_x {
                for y in -num_grid_pints..num_grid_pints {
                    commands
                        .spawn_bundle(MaterialMesh2dBundle {
                            material: materials.add(Color::rgb(0.5, 0.4, 0.5).into()),
                            mesh: mesh.clone(),
                            transform: Transform::from_translation(Vec3::new(
                                x as f32 * globals.grid_size,
                                y as f32 * globals.grid_size,
                                0.0,
                            )),
                            ..Default::default()
                        })
                        .insert(Grid);
                }
            }
        } else {
            for entity in query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
