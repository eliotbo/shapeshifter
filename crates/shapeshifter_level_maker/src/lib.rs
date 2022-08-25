// TODO: delete this example

mod cut;
pub mod input;
pub mod load;
mod material;
mod poly;
pub mod save;
mod target;
mod util;
mod view;

// pub use cut::*;
// pub use input::*;
// pub use load::*;
// pub use material::*;
// pub use poly::*;
// pub use save::*;
// pub use target::*;
// pub use util::*;
// pub use view::*;

use cut::*;
use input::*;
use load::*;
use material::*;
use poly::*;
use save::*;
use target::*;
use util::*;
use view::*;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_easings::*;
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

// use bevy_inspector_egui::WorldInspectorPlugin;
// use bevy_obj::*;

pub struct ShapeshifterLevelMakerPlugin;

impl Plugin for ShapeshifterLevelMakerPlugin {
    fn build(&self, app: &mut App) {
        // .insert_resource(WindowDescriptor {
        //     title: "pen".to_string(),
        //     width: 1200.,
        //     height: 800.,
        //     // present_mode: bevy::window::PresentMode::Immediate,
        //     ..Default::default()
        // })
        //
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        //
        app.add_event::<StartMakingSegment>()
            .add_event::<Action>()
            .add_event::<TestCollisionEvent>()
            .add_event::<TestWinEvent>()
            .insert_resource(Globals::default())
            .insert_resource(Cursor::default())
            .add_plugin(bevy_easings::EasingsPlugin)
            .add_plugin(FillMesh2dPlugin)
            .add_plugin(TargetMesh2dPlugin)
            .add_plugin(CutMesh2dPlugin)
            .add_plugin(LoadPlugin)
            .add_plugin(SavePlugin)
            // // // // // // .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(CutPlugin)
            .add_plugin(PolyMakerPlugin)
            .add_plugin(TargetPlugin)
            .add_startup_system(setup_mesh)
            .add_system(record_mouse_events_system.exclusive_system().at_start())
            .add_system(direct_action)
            .add_system(glow_poly)
            .add_system(rotate_once)
            .add_system(delete_poly)
            .add_system(toggle_grid)
            .add_system(test_collisions)
            .add_system(revert_to_init)
            .add_system(move_path_point)
            .add_system(hover_path_point)
            .add_system(direct_release_action)
            // delete me please
            .add_system(debug_input)
            .add_system(transform_poly.exclusive_system().at_end());
    }
}

pub fn setup_mesh(
    mut load_event_writer: EventWriter<Load>,
    mut action_event_writer: EventWriter<Action>,
) {
    // load_event_writer.send(Load("my_mesh2".to_string()));
    // action_event_writer.send(Action::LoadDialog);
    // action_event_writer.send(Action::QuickLoadTarget { maybe_name: None });

    // action_event_writer.send(Action::ToggleGrid);
}

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
    mut load_target_event_writer: EventWriter<LoadTarget>,
    loaded_path: Res<LoadedTargetPath>,
    loaded_target_path: Res<LoadedTargetPath>,
    // mut action_event_writer: EventWriter<Action>,
) {
    if let Some(Action::RevertToInit) = action_event_reader.iter().next() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        if let Some(ref name) = loaded_path.maybe_path {
            load_event_writer.send(Load(name.clone()));
        }
        if let Some(ref name) = loaded_target_path.maybe_path {
            load_event_writer.send(Load(name.clone()));
        }
    }
}

pub fn test_collisions(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut MeshMeta), With<Polygon>>,
    target_query: Query<(&Transform, &Target)>,
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

        if let Some((transform, target)) = target_query.iter().next() {
            if meta1.precise_intersect_test(&target.path, &transform1, &transform) {
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
