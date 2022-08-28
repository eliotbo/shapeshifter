// TODO: delete this example

mod cut;
mod poly;
mod target;
mod view;

pub mod input;
pub mod load_poly_wasm;
pub mod material;
pub mod util;

///// Delete when building for wasm

use cut::*;
use input::*;
use load_poly_wasm::*;
use material::*;
use poly::*;
use target::*;
use util::*;
use view::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod save;
#[cfg(not(target_arch = "wasm32"))]
use save::*;
#[cfg(not(target_arch = "wasm32"))]
pub mod load;
#[cfg(not(target_arch = "wasm32"))]
use load::*;

///// Delete when building for wasm

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
// use bevy_easings::*;
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

// use bevy_inspector_egui::WorldInspectorPlugin;
// use bevy_obj::*;

pub struct ShapeshifterLevelMakerPlugin;

#[cfg(not(target_arch = "wasm32"))]
fn add_save(app: &mut App) {
    app.add_plugin(SavePlugin);
    app.add_plugin(LoadPlugin);
}

#[cfg(not(target_os = "linux"))]
fn add_save(_app: &mut App) {
    // app.add_plugin(SavePlugin);
}

impl Plugin for ShapeshifterLevelMakerPlugin {
    fn build(&self, mut app: &mut App) {
        //
        app.add_event::<StartMakingSegment>()
            .add_event::<Action>()
            .add_event::<TestCollisionEvent>()
            .add_event::<TestWinEvent>()
            .add_event::<SpawnPoly>()
            .add_event::<SpawnTarget>()
            .add_event::<SpawnLevel>()
            .add_event::<HasWonLevelEvent>()
            .add_event::<PerformedCut>()
            .add_event::<TurnPolyIntoTarget>()
            .add_event::<SpawnTargetKeepTarget>()
            .add_event::<SpawnPolyKeepPoly>()
            .add_event::<PolyIsInsideTarget>()
            .add_event::<CheckPolyInsideTarget>()
            //
            //
            //
            //
            // .insert_resource(WindowDescriptor {
            //     title: "pen".to_string(),
            //     width: 1200.,
            //     height: 800.,
            //     present_mode: bevy::window::PresentMode::Immediate,
            //     ..Default::default()
            // })
            // .add_plugin(LogDiagnosticsPlugin::default())
            // .add_plugin(FrameTimeDiagnosticsPlugin::default())
            //
            //
            //
            .insert_resource(Globals::default())
            .insert_resource(Cursor::default())
            .insert_resource(LoadedPolygonsRaw::default())
            .insert_resource(CurrentLevel::default())
            //
            .add_plugin(bevy_easings::EasingsPlugin)
            .add_plugin(FillMesh2dPlugin)
            .add_plugin(TargetMesh2dPlugin)
            .add_plugin(CutMesh2dPlugin)
            //
            // .add_plugin(SavePlugin)
            // // // // // // .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(CutPlugin)
            .add_plugin(PolyMakerPlugin)
            .add_plugin(TargetPlugin)
            .add_startup_system(load_all_polygons)
            .add_system(setup_mesh)
            .add_system(spawn_poly)
            .add_system(spawn_target)
            .add_system(turn_poly_into_target)
            .add_system(record_mouse_events_system.exclusive_system().at_start())
            .add_system(direct_action)
            .add_system(glow_poly)
            .add_system(rotate_poly)
            .add_system(rotate_once)
            .add_system(delete_poly)
            .add_system(toggle_grid)
            .add_system(test_collisions)
            .add_system(revert_to_init)
            .add_system(move_path_point)
            .add_system(hover_path_point)
            .add_system(direct_release_action)
            .add_system(check_cut_timer)
            // delete me please
            // .add_system(debug_input)
            .add_system(transform_poly.exclusive_system().at_end());

        add_save(&mut app);

        // if cfg!(unix) {
        // app.add_plugin(SavePlugin);
        // }
    }
}

pub fn setup_mesh(
    // mut load_event_writer: EventWriter<Load>,
    // mut action_event_writer: EventWriter<Action>,
    loaded_polygons_raw: Res<LoadedPolygonsRaw>,
) {
    if loaded_polygons_raw.is_changed() {
        // load_event_writer.send(Load("my_mesh2".to_string()));
        // action_event_writer.send(Action::LoadDialog);
        // action_event_writer.send(Action::QuickLoadTarget { maybe_name: None });

        // action_event_writer.send(Action::ToggleGrid);
    }
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
    current_level: Res<CurrentLevel>,
    mut spawn_poly_event_writer: EventWriter<SpawnPoly>,
    mut spawn_target_event_writer: EventWriter<SpawnTarget>,
) {
    if let Some(Action::RevertToInit) = action_event_reader.iter().next() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        spawn_poly_event_writer.send(SpawnPoly {
            polygon: current_level.polygon.clone(),
            polygon_multiplier: current_level.polygon_multiplier,
        });
        spawn_target_event_writer.send(SpawnTarget {
            target: current_level.target.clone(),
            target_multiplier: current_level.target_multiplier,
        });
    }
}

//
//
// 0.3 seconds after a cut, the polygons is checked for collisions
pub fn check_cut_timer(
    mut collision_test_event: EventWriter<TestCollisionEvent>,
    mut cut_timer: ResMut<CutTimer>,
    time: Res<Time>,
) {
    if cut_timer.timer.tick(time.delta()).just_finished() {
        for entity in cut_timer.entities.iter() {
            collision_test_event.send(TestCollisionEvent(*entity));
            // info!("cut timer finished");
        }
        cut_timer.entities.clear();
    }
}

//
//
// To save programming time, we test all polygons against all other polygons when
// any polygon is moved.
pub fn test_collisions(
    // mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Transform,
            &mut MeshMeta,
            &Handle<FillMesh2dMaterial>,
        ),
        With<Polygon>,
    >,
    mut fill_mesh_assets: ResMut<Assets<FillMesh2dMaterial>>,
    target_query: Query<(&Transform, &Target)>,
    mut collision_test_event: EventReader<TestCollisionEvent>,
    mut check_win_condition_event: EventWriter<TestWinEvent>,
    mut check_poly_inside_writer_event: EventWriter<CheckPolyInsideTarget>,
) {
    //
    //
    //
    if let Some(TestCollisionEvent(entity)) = collision_test_event.iter().next() {
        //
        //
        //
        //
        // check whether the polygon is inside the target
        check_poly_inside_writer_event.send(CheckPolyInsideTarget { entity: *entity });
        //
        //
        //
        // check collisions for all polygons against all other polygons
        //
        let mut colliding_entities: Vec<Entity> = Vec::new();
        let mut iter = query.iter_combinations_mut();
        while let Some([(entity1, transform1, meta1, _), (entity2, transform2, meta2, _)]) =
            iter.fetch_next()
        {
            //
            // do not test collision with self
            if meta1.id == meta2.id {
                continue;
            }

            //
            //
            if meta1.bounding_box_collide(&meta2.path, &transform1, &transform2) {
                if meta1.precise_intersect_test(&meta2.path, &transform1, &transform2) {
                    // do_go_back_to_previous_pos = true;
                    //
                    //

                    colliding_entities.push(entity2);
                    colliding_entities.push(entity1);
                }
            }
        }
        //
        //
        // test the target zone
        for (entity1, transform1, meta1, _) in query.iter() {
            if let Some((transform, target)) = target_query.iter().next() {
                if meta1.precise_intersect_test(&target.path, &transform1, &transform) {
                    colliding_entities.push(entity1);
                }
            }
        }

        // for entity in colliding_entities {
        for (entity, _, mut meta, mat_handle) in query.iter_mut() {
            let fill_mat = fill_mesh_assets.get_mut(mat_handle).unwrap();
            if colliding_entities.contains(&entity) {
                meta.is_intersecting = true;

                fill_mat.is_intersecting = 1.0;
            } else {
                meta.is_intersecting = false;

                fill_mat.is_intersecting = 0.0;
            }
        }

        if colliding_entities.len() == 0 {
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
