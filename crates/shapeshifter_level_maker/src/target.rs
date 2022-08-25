use crate::material::*;
// use crate::poly::make_polygon_mesh;
use crate::input::Action;
use crate::poly::Polygon;
use crate::util::*;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use lyon::algorithms::hit_test::*;
use lyon::path::FillRule;
use lyon::tessellation::math::Point;
use lyon::tessellation::path::Path;

#[derive(Component)]
pub struct Target {
    pub path: Path,
}

pub struct LoadedTarget {
    pub save_mesh_meta: SaveMeshMeta,
}
pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadedTarget>()
            .add_system(spawn_target)
            .add_system(delete_target)
            .add_system(check_win_condition);
    }
}

pub fn delete_target(
    mut commands: Commands,
    query: Query<Entity, With<Target>>,
    mut action_event_reader: EventReader<Action>,
) {
    // if let Some(Action::DeleteTarget) = action_event_reader.iter().next() {
    if action_event_reader
        .iter()
        .any(|x| x == &Action::DeleteTarget)
    {
        info!("delete_target");
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

//
//
//
// spawn a target, where all polygons will need to fit in as a win condition
//
pub fn spawn_target(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TargetMesh2dMaterial>>,
    globals: Res<Globals>,
    query: Query<Entity, With<Target>>,
    mut loaded_target_event: EventReader<LoadedTarget>,
) {
    for loaded_target in loaded_target_event.iter() {
        //
        //
        // remove current target is there is one
        //
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        //
        // shift the target to the center of the screen, so that the path is
        // always in the center of the screen. The entity containing the path
        // will be shifted to the right.
        let points = shift_to_center_of_mass(&loaded_target.save_mesh_meta.points);

        //
        // trace the path of the target
        let mut builder = Path::builder();
        for (idx, original_pos) in points.iter().enumerate() {
            //
            //
            // The target area should be larger than the smallest area of the corresponding polygon
            // to allow for leeway when placing the polygon pieces
            //
            let pos = globals.target_size_multiplier * *original_pos;
            //
            //
            if idx == 0 {
                builder.begin(Point::new(pos.x, pos.y));
            } else {
                builder.line_to(Point::new(pos.x, pos.y));
            };
        }
        //
        //
        builder.close();
        let path = builder.build();

        //
        //
        // Make the mesh corresponding to the target path. The "false" means that the path
        // should not be displaced to the origin
        //
        let (mesh, _center_of_mass) = make_polygon_mesh(&path, true);

        let material = materials.add(TargetMesh2dMaterial {
            color: globals.target_color.into(),
            ..Default::default()
        });

        //
        // spawn the target
        //
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(mesh)),
                material,
                transform: Transform::from_translation(Vec3::new(300.0, 0.0, 0.0)),
                ..Default::default()
            })
            .insert(Target { path });
    }
}

//
//
// Checks whether all the points of all polygons are within the bounds of the target path
pub fn check_win_condition(
    query: Query<(&Transform, &MeshMeta), With<Polygon>>,
    target_query: Query<(&Target, &Transform)>,
    mut check_win_condition_event: EventReader<TestWinEvent>,
) {
    for _ in check_win_condition_event.iter() {
        //
        //
        //
        if let Some((target, target_transform)) = target_query.iter().next() {
            let mut has_won = true;
            let (transformed_target_path, _) = transform_path(&target.path, target_transform);
            for (transform, meta) in query.iter() {
                //
                //
                //
                // At this point, we know that the polygon segments are not intersecting with
                // the target's segments, because this test was passed before sending TestWinEvent
                // from test_collisions(..)
                let (transformed_path, _) = transform_path(&meta.path, transform);
                for seg in transformed_path.iter() {
                    let pos: Point = seg.from();

                    if !hit_test_path(&pos, transformed_target_path.iter(), FillRule::EvenOdd, 0.1)
                    {
                        has_won = false;
                    }
                }
            }
            if has_won {
                println!("You won!");
            }
        }
    }
}