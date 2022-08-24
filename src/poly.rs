use crate::input::Action;
use crate::input::Cursor;
use crate::material::*;
use crate::util::*;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use lyon::path::path::BuilderImpl;
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::{builder::NoAttributes, Path};
use lyon::tessellation::{FillOptions, FillTessellator, VertexBuffers};

use rand::{thread_rng, Rng};

pub struct PolyPlugin;

impl Plugin for PolyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(delete_making_polygon)
            .add_system(end_polygon)
            .add_system(start_polygon)
            .add_system(making_segment)
            .add_system(end_segment)
            .add_system(add_point_to_poly)
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

pub fn add_point_to_poly(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TargetMesh2dMaterial>>,
    globals: Res<Globals>,
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

            let new_poly_entity = commands
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

// make a mesh from a path
//
// shift_com: shift the center of mass of path to origin.
pub fn make_polygon_mesh(path: &Path, shift_com: bool) -> (Mesh, Vec2) {
    let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

    let mut vertex_builder = simple_builder(&mut buffers);

    // Create the tessellator.
    let mut tessellator = FillTessellator::new();

    // Compute the tessellation.
    let result = tessellator.tessellate_path(path, &FillOptions::default(), &mut vertex_builder);
    assert!(result.is_ok());

    let mut mesh_pos_attributes: Vec<[f32; 3]> = Vec::new();
    let mut mesh_attr_uvs: Vec<[f32; 2]> = Vec::new();
    let mut new_indices: Vec<u32> = Vec::new();

    // show points from look-up table

    let color = Color::WHITE;
    let mut colors = Vec::new();

    for position in buffers.vertices[..].iter() {
        let pos_x = position.x;
        let pos_y = position.y;
        mesh_pos_attributes.push([pos_x, pos_y, 0.0]);

        colors.push([color.r(), color.g(), color.b(), 1.0]);
    }

    for ind in buffers.indices[..].iter().rev() {
        new_indices.push(ind.clone() as u32);
    }
    //
    //
    //
    //
    /////////////////////// compute center of mass /////////////////////////
    //
    //
    // compute center of mass using center of mass of each triangle
    //
    let mut center_of_mass = Vec2::ZERO;
    let num_triangles = new_indices.iter().count() / 3;

    for ind in 0..num_triangles {
        let index = ind * 3;
        let triangle = [
            mesh_pos_attributes[new_indices[index] as usize],
            mesh_pos_attributes[new_indices[index + 1] as usize],
            mesh_pos_attributes[new_indices[index + 2] as usize],
        ];

        center_of_mass += Vec2::new(
            (triangle[0][0] + triangle[1][0] + triangle[2][0]) / 3.0,
            (triangle[0][1] + triangle[1][1] + triangle[2][1]) / 3.0,
        ) / num_triangles as f32;
    }

    /////////////////////// compute center of mass /////////////////////////
    //
    //
    //
    //

    // adjust the mesh position attributes such that the center of mass is at the origin
    mesh_pos_attributes = mesh_pos_attributes
        .iter()
        .map(|x| {
            let new_pos = Vec2::new(x[0], x[1]) - center_of_mass * shift_com as i32 as f32;
            [new_pos.x, new_pos.y, 0.0]
        })
        .collect();

    //
    //
    //
    //
    //////////////////////////// uvs ///////////////////////////////
    //
    let xs: Vec<f32> = mesh_pos_attributes.iter().map(|v| v[0]).collect();
    let ys: Vec<f32> = mesh_pos_attributes.iter().map(|v| v[1]).collect();

    // find the min and max of a vec of f32
    fn bounds(v: &Vec<f32>) -> (f32, f32) {
        let mut min = v[0];
        let mut max = v[0];
        for i in 1..v.len() {
            if v[i] < min {
                min = v[i];
            }
            if v[i] > max {
                max = v[i];
            }
        }
        (min, max)
    }

    let bounds_x = bounds(&xs);
    let size_x = bounds_x.1 - bounds_x.0;
    let bounds_y = bounds(&ys);
    let size_y = bounds_y.1 - bounds_y.0;

    let mut normals = Vec::new();
    for pos in &mesh_pos_attributes {
        let (pos_x, pos_y) = (pos[0], pos[1]);

        mesh_attr_uvs.push([
            1.0 * (pos_x - bounds_x.0) / size_x,
            1.0 * (pos_y - bounds_y.0) / size_y,
        ]);

        normals.push([0.0, 0.0, 1.0]);
    }
    //
    //////////////////////////// uvs ///////////////////////////////

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_pos_attributes.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.set_indices(Some(Indices::U32(new_indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_attr_uvs);

    return (mesh, center_of_mass);
}
