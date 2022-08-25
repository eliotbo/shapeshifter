use crate::cut::CutSegment;
use crate::input::Action;
use crate::input::Cursor;
use crate::material::*;

use crate::util::*;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use lyon::path::path::BuilderImpl;

use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::{builder::NoAttributes, Path};

use rand::{thread_rng, Rng};

pub struct PolyMakerPlugin;

impl Plugin for PolyMakerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(delete_making_polygon)
            .add_system(end_polygon)
            .add_system(start_polygon)
            .add_system(making_segment)
            .add_system(end_segment)
            .add_system(add_point_to_poly)
            .add_system(delete_poly)
            .add_system(delete_all)
            .add_system(select_poly)
            .add_system(start_poly_segment);
    }
}

pub struct StartMakingSegment {
    pub start: Point,
}

#[derive(Component)]
pub struct PolySegmentComponent;

#[derive(Component)]
pub struct Polygon;

#[derive(Component)]
pub struct MakingSegment {
    pub start: Point,
}
#[derive(Component)]
pub struct MakingPolygon {
    pub path: NoAttributes<BuilderImpl>,
    pub current_point: Point,
    pub starting_point: Point,
    pub all_points: Vec<Vec2>,
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

pub fn delete_all(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<Polygon>,
            With<CutSegment>,
            With<Ghost>,
            With<PathPoint>,
        )>,
    >,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::DeleteAll) = action_event_reader.iter().next() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
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

pub fn add_point_to_poly(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<TargetMesh2dMaterial>>,
    // globals: Res<Globals>,
    mut query: Query<
        (
            Entity,
            &mut MeshMeta,
            &Transform,
            &Handle<FillMesh2dMaterial>,
        ),
        With<Polygon>,
    >,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::AddPointAt { pos }) = action_event_reader.iter().next() {
        info!("add point at {:?}", pos);
        //
        //
        //
        if let Some((entity, mut mesh_meta, transform, mat_handle)) = query.iter_mut().next() {
            //
            //
            //
            // find closest path segment to the cursor pos
            //
            let mut min_distance = f32::MAX;
            let mut closest_segment = 111111111111111111;

            let (transformed_path, _angle) = transform_path(&mesh_meta.path, transform);

            for (k, seg) in transformed_path.iter().enumerate() {
                //
                //
                //

                let segment: (Vec2, Vec2) = (
                    Vec2::new(seg.from().x, seg.from().y),
                    Vec2::new(seg.to().x, seg.to().y),
                );

                let dist = distance_from_point_to_segment(*pos, segment);
                info!("distance from point to segment: {:?}", dist);

                if dist < min_distance {
                    info!("new min distance: {}", dist);
                    min_distance = dist;
                    closest_segment = k;
                }
            }

            let mut builder = Path::builder();
            let mut all_points = Vec::new();
            for (k, seg) in transformed_path.iter().enumerate() {
                if k == 0 {
                    builder.begin(seg.from());
                    all_points.push(Vec2::new(seg.from().x, seg.from().y));
                } else {
                    builder.line_to(seg.from());
                    all_points.push(Vec2::new(seg.from().x, seg.from().y));
                }

                if k == closest_segment {
                    //
                    //
                    //
                    builder.line_to(Point::new(pos.x, pos.y));
                    all_points.push(Vec2::new(pos.x, pos.y));
                }
            }
            builder.close();

            mesh_meta.path = builder.build();

            let (mesh, center_of_mass) = make_polygon_mesh(&mesh_meta.path, false);

            mesh_meta.points = all_points
                .clone()
                .iter()
                .map(|x| *x - center_of_mass)
                .collect();

            let _new_poly_entity = commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(mesh)),
                    material: mat_handle.clone(),
                    transform: Transform::identity(),
                    ..default()
                })
                .insert(Polygon)
                .insert(mesh_meta.clone())
                .id();

            info!("new poly entity: {:?}", _new_poly_entity);
            //
            // remove old poly
            commands.entity(entity).despawn_recursive();
        }
    }
}

// computes the distance from a point to a segment
pub fn distance_from_point_to_segment(point: Vec2, segment: (Vec2, Vec2)) -> f32 {
    let (start, end) = segment;
    let l2 = (end - start).length_squared();
    if l2 == 0.0 {
        return (point - start).length();
    }
    let t = ((point - start).dot(end - start)) / l2;
    if t < 0.0 {
        return (point - start).length();
    } else if t > 1.0 {
        return (point - end).length();
    }
    let projection = start + t * (end - start);
    (point - projection).length()
}

pub fn start_poly_segment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<Entity, With<MakingPolygon>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut start_segment_event_reader: EventReader<StartMakingSegment>,
    mut action_event_reader: EventReader<Action>,
    globals: Res<Globals>,
    cursor: Res<Cursor>,
) {
    // the start segment event can either come from the Action event or the StartMakingSegment event
    let mut maybe_pos =
        if let Some(Action::StartMakingSegment { pos }) = action_event_reader.iter().next() {
            Some(*pos)
        } else {
            None
        };

    for ev in start_segment_event_reader.iter() {
        maybe_pos = Some(Point::new(ev.start.x, ev.start.y));
    }

    if let Some(start) = maybe_pos {
        for parent_polygon in query.iter() {
            let segment = Segment {
                start,
                end: cursor.clone().into(),
            };

            let segment_meta = get_segment_meta(segment);

            let mesh = bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(
                Vec2::new(segment_meta.length, globals.cutting_segment_thickness),
            ))));

            let material = materials.add(globals.polygon_segment_color.into());
            let child_segment = commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh,
                    material,
                    transform: segment_meta.transform,
                    ..Default::default()
                })
                .insert(MakingSegment { start })
                .insert(PolySegmentComponent)
                .id();

            commands
                .entity(parent_polygon)
                .push_children(&[child_segment]);
        }
    }
}

pub fn start_polygon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,

    mut materials: ResMut<Assets<ColorMaterial>>,
    mut action_event_reader: EventReader<Action>,
    // mut start_polygon_event_reader: EventReader<StartMakingPolygon>,
    globals: Res<Globals>,
    cursor: Res<Cursor>,
) {
    // for start_poly in start_polygon_event_reader.iter() {
    if let Some(Action::StartMakingPolygon { mut pos }) = action_event_reader.iter().next() {
        //
        // snap end to grid
        if globals.snap_to_grid {
            pos = (pos.clone() / globals.grid_size).round() * globals.grid_size;
        }

        let segment = Segment {
            start: pos,
            end: cursor.clone().into(),
        };

        let segment_meta = get_segment_meta(segment);

        let mesh = bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            segment_meta.length,
            globals.cutting_segment_thickness,
        )))));

        let mut path = Path::builder();
        path.begin(pos);

        // make invisible parent entity
        let parent_polygon = commands
            .spawn_bundle((
                Visibility { is_visible: true }, // visibility is inherited by all children
                ComputedVisibility::not_visible(), // the parent entity is not a rendered object
                GlobalTransform::default(),
                Transform::default(),
            ))
            .insert(MakingPolygon {
                path,
                current_point: pos,
                starting_point: pos,
                all_points: vec![Vec2::new(pos.x, pos.y)],
            })
            .id();

        let material = materials.add(globals.polygon_segment_color.into());
        let child_segment = commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh,
                material,
                transform: segment_meta.transform,
                ..Default::default()
            })
            .insert(MakingSegment { start: pos })
            .insert(PolySegmentComponent)
            .id();

        commands
            .entity(parent_polygon)
            .push_children(&[child_segment]);
    }
}

// changes the position of the segment according to mouse position
pub fn making_segment(
    mut query: Query<(&mut Transform, &Mesh2dHandle, &MakingSegment)>,
    mut meshes: ResMut<Assets<Mesh>>,
    globals: Res<Globals>,
    cursor: Res<Cursor>,
) {
    for (mut transform, mesh_handle, making_segment) in query.iter_mut() {
        let mesh = meshes.get_mut(&mesh_handle.0).unwrap();

        let mut end = point(cursor.position.x, cursor.position.y);
        // snap end to grid
        if globals.snap_to_grid {
            end = (end / globals.grid_size).round() * globals.grid_size;
        }

        let segment = Segment {
            start: making_segment.start,
            end,
        };

        let segment_meta = get_segment_meta(segment);

        *mesh = Mesh::from(shape::Quad::new(Vec2::new(
            segment_meta.length,
            globals.cutting_segment_thickness,
        )));

        *transform = segment_meta.transform;
    }
}

pub fn end_segment(
    mut commands: Commands,
    mut segment_query: Query<(
        &Parent,
        Entity,
        &mut Transform,
        &Mesh2dHandle,
        &MakingSegment,
    )>,
    mut polygon_query: Query<&mut MakingPolygon>,
    mut meshes: ResMut<Assets<Mesh>>,
    globals: Res<Globals>,
    mut action_event_reader: EventReader<Action>,
    mut start_segment_event_writer: EventWriter<StartMakingSegment>,
) {
    //
    // move one end of the segment to the cursor position

    // for _ in end_segment_event_reader.iter() {
    if let Some(Action::EndSegment { mut pos }) = action_event_reader.iter().next() {
        // if action_event_reader.iter().any(|x| match x {
        //     &Action::EndSegment { pos: _ } => true,
        //     _ => false,
        // }) {
        // let mut pos = Point::new(0.0, 0.0);
        info!("up here");
        for (parent, entity, mut transform, mesh_handle, making_segment) in segment_query.iter_mut()
        {
            // snap end to grid
            if globals.snap_to_grid {
                pos = (pos / globals.grid_size).round() * globals.grid_size;
            }

            //
            // update polygon
            let mut making_polygon = polygon_query.get_mut(**parent).unwrap();
            // let current_position = point(cursor.position.x, cursor.position.y);
            making_polygon.current_point = pos; //current_position;
            making_polygon.path.line_to(pos);
            making_polygon.all_points.push(Vec2::new(pos.x, pos.y));

            let mesh = meshes.get_mut(&mesh_handle.0).unwrap();

            let segment = Segment {
                start: making_segment.start,
                end: pos,
            };

            let segment_meta = get_segment_meta(segment);

            *mesh = Mesh::from(shape::Quad::new(Vec2::new(
                segment_meta.length,
                globals.cutting_segment_thickness,
            )));

            *transform = segment_meta.transform;

            commands.entity(entity).remove::<MakingSegment>();

            info!("end segment, and start new");

            start_segment_event_writer.send(StartMakingSegment { start: pos });
        }
    }
}

// upon pressing escape, delete the polygon
pub fn delete_making_polygon(
    mut commands: Commands,
    mut action_event_reader: EventReader<Action>,
    polygon_query: Query<(Entity, &MakingPolygon)>,
) {
    if let Some(Action::DeleteMakingPoly) = action_event_reader.iter().next() {
        for (entity, _) in polygon_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

use lyon::algorithms::area::*;

// upon pressing Enter, end making the polygon
pub fn end_polygon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut action_event_reader: EventReader<Action>,
    mut polygon_query: Query<(Entity, &mut MakingPolygon)>,
    globals: Res<Globals>,
) {
    //
    //
    if let Some(Action::EndMakingPolygon) = action_event_reader.iter().next() {
        //
        // There is only one MakingPolygon at a time
        for (entity, mut poly) in polygon_query.iter_mut() {
            //
            //
            // end point of last segment must be the same as the starting point
            // close the polygon using the last segment.
            //
            // add last segment to close the polygon
            poly.path.close();

            let path = poly.path.clone().build();
            let area = approximate_signed_area(0.1, &path);

            info!("area : {}", area);
            if area.abs() < 200.0 {
                info!("area too small: {}", area);
                commands.entity(entity).despawn_recursive();
                return;
            }

            // the path is shifted to the origin and the mesh transform is moved instead
            let (mesh, center_of_mass) = make_polygon_mesh(&path, true);

            let mat_handle = fill_materials.add(FillMesh2dMaterial {
                color: globals.polygon_color.into(),
                show_com: 0.0, // show center of mass
                selected: 0.0,
            });

            let translation = lyon::geom::Translation::new(-center_of_mass.x, -center_of_mass.y);
            let transformed_path = path.transformed(&translation);

            let mut rng = thread_rng();
            let id = rng.gen::<u64>();

            let fill_transform =
                Transform::from_translation(center_of_mass.extend(rng.gen::<f32>()));

            let _new_poly_entity = commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(mesh)),
                    material: mat_handle,
                    transform: fill_transform,
                    ..default()
                })
                .insert(Polygon)
                .insert(MeshMeta {
                    id,
                    path: transformed_path.clone(),
                    // move points towards the origin
                    points: poly
                        .all_points
                        .clone()
                        .iter()
                        .map(|x| *x - center_of_mass)
                        .collect(),
                    previous_transform: fill_transform,
                })
                .id();

            // despawn the MakingPolygon invisible entity and the child segments
            commands.entity(entity).despawn_recursive();
        }
    }
}

// gets a MeshMeta and ajusts its Path by translating one of its points, that point that
// is closest to the cursor position
pub fn hover_path_point(
    mut commands: Commands,
    mut polygon_query: Query<(Entity, &Transform, &mut MeshMeta), With<Polygon>>,
    path_point_query: Query<(Entity, &PathPoint)>,
    cursor: Res<Cursor>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut action_event_reader: EventReader<Action>,
) {
    for (entity, _) in path_point_query.iter() {
        commands.entity(entity).despawn();
    }

    for (entity, transform, mut mesh_meta) in polygon_query.iter_mut() {
        // let (transformed_path, _) = transform_path(&mesh_meta.path, transform);
        // info!("points: {:?}", mesh_meta.points);
        // let mut iter_over_path = transformed_path.iter();
        // iter_over_path.next(); // skip first point

        //

        //
        //
        //
        if let Some((index, point)) = mesh_meta.get_close_from_pos(cursor.position, transform, 30.)
        {
            let mesh = bevy::sprite::Mesh2dHandle(
                meshes.add(Mesh::from(shape::Quad::new(Vec2::new(3., 3.)))),
            );
            // spawn a circle at the point
            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    material: materials.add(Color::rgb(0.5, 0.4, 0.5).into()),
                    mesh: mesh.clone(),
                    transform: Transform::from_translation(point.extend(1.0)),
                    ..Default::default()
                })
                .insert(PathPoint);

            if let Some(Action::MovePathPoint) = action_event_reader.iter().next() {
                commands.entity(entity).insert(MovingPathPoint {
                    index,
                    previous_pos: point,
                });
            }
        }
    }
}

pub fn move_path_point(
    mut commands: Commands,
    mut polygon_query: Query<(
        Entity,
        &mut Transform,
        &mut Mesh2dHandle,
        &mut MeshMeta,
        &MovingPathPoint,
    )>,
    // mut path_point_query: Query<(Entity, &PathPoint)>,
    cursor: Res<Cursor>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transform, mut mesh_handle, mut mesh_meta, moving_path_point) in
        polygon_query.iter_mut()
    {
        // let mut path = *mesh_meta.path;

        // let mut builder = Path::builder();
        // let mut all_points = Vec::new();
        //
        //
        //
        // let l = mesh_meta.points.len();
        println!("START: {:#?}", mesh_meta.points);

        // if let Some(what) = mesh_meta.path.iter().nth(moving_path_point.index) {
        //     info!("{:?}", what.from());
        // }

        if let Some(v) = mesh_meta.points.iter_mut().nth(moving_path_point.index) {
            *v = moving_path_point.previous_pos + (cursor.position - cursor.last_click_position);
        }

        let mut builder = Path::builder();
        for (k, pt) in mesh_meta.points.iter().enumerate() {
            let seg = Point::new(pt.x, pt.y);
            if k == 0 {
                builder.begin(seg);
            } else {
                builder.line_to(seg);
            }
        }
        builder.close();
        mesh_meta.path = builder.build();

        // for (k, seg) in mesh_meta.path.clone().iter().enumerate() {
        //     println!("path: {:#?}", mesh_meta.path);
        //     //
        //     //
        //     let mut point_pos_v2 = Vec2::new(seg.from().x, seg.from().y);
        //     //
        //     //
        //     //

        //     if k == moving_path_point.index {
        //         // info!("k : {}", k);
        //         point_pos_v2 =
        //             moving_path_point.previous_pos + (cursor.position - cursor.last_click_position);
        //     }

        //     let point_pos = Point::new(point_pos_v2.x, point_pos_v2.y);

        //     if k == 0 {
        //         builder.begin(point_pos);
        //     } else {
        //         builder.line_to(point_pos);
        //         all_points.push(point_pos_v2);
        //     }
        // }

        // builder.close();

        // mesh_meta.path = builder.build();

        let (mesh, _center_of_mass) = make_polygon_mesh(&mesh_meta.path, false);

        *mesh_handle = Mesh2dHandle(meshes.add(mesh));

        // transform.translation = Vec3::new(0., 0., transform.translation.z);
        *transform = Transform::identity();
    }
}
