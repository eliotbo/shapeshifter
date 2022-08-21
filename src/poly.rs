use crate::input::Cursor;
use crate::material::*;
use crate::util::*;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

use core::cmp::Ordering;

use lyon::algorithms::raycast::*;
use lyon::path::path::BuilderImpl;
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::{builder::NoAttributes, Path};
use lyon::tessellation::{FillOptions, FillTessellator, VertexBuffers};

use rand::{thread_rng, Rng};

pub fn make_square() -> (Path, Vec<Vec2>) {
    let mut path = Path::builder();
    path.begin(point(0.0, 0.0));
    path.line_to(point(100.0, 0.0));
    path.line_to(point(100.0, 100.0));
    path.line_to(point(0.0, 100.0));
    path.close();
    let built_path = path.build();

    let mut points = Vec::new();
    points.push(Vec2::new(0.0, 0.0));
    points.push(Vec2::new(100.0, 0.0));
    points.push(Vec2::new(100.0, 100.0));
    points.push(Vec2::new(0.0, 100.0));

    (built_path, points)
}

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

// events
pub struct StartMakingPolygon {
    pub pos: Point,
}
pub struct StartMakingSegment {
    pub pos: Point,
}
pub struct EndSegment {
    pub pos: Point,
}

pub struct EndMakingPolygon;

pub fn start_poly_segment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<Entity, With<MakingPolygon>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut start_segment_event_reader: EventReader<StartMakingSegment>,
    globals: Res<Globals>,
    cursor: Res<Cursor>,
) {
    for start_segment in start_segment_event_reader.iter() {
        for parent_polygon in query.iter() {
            let start = point(start_segment.pos.x, start_segment.pos.y);
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
    mut start_polygon_event_reader: EventReader<StartMakingPolygon>,
    globals: Res<Globals>,
    cursor: Res<Cursor>,
) {
    for start_poly in start_polygon_event_reader.iter() {
        // info!("start polygon:  {:?}", start_poly.pos);
        let start = point(start_poly.pos.x, start_poly.pos.y);
        let segment = Segment {
            start,
            end: cursor.clone().into(),
        };

        let segment_meta = get_segment_meta(segment);

        let mesh = bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            segment_meta.length,
            globals.cutting_segment_thickness,
        )))));

        let mut path = Path::builder();
        path.begin(start);

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
                current_point: start,
                starting_point: start,
                all_points: vec![Vec2::new(start.x, start.y)],
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
            .insert(MakingSegment { start })
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
        // info!("making polygon");
        let mesh = meshes.get_mut(&mesh_handle.0).unwrap();

        let segment = Segment {
            start: making_segment.start,
            end: point(cursor.position.x, cursor.position.y),
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
    cursor: Res<Cursor>,
    mut end_segment_event_reader: EventReader<EndSegment>,
    mut start_segment_event_writer: EventWriter<StartMakingSegment>,
) {
    //
    // move one end of the segment to the cursor position

    for _ in end_segment_event_reader.iter() {
        for (parent, entity, mut transform, mesh_handle, making_segment) in segment_query.iter_mut()
        {
            //
            // update polygon
            let mut making_polygon = polygon_query.get_mut(**parent).unwrap();
            let current_position = point(cursor.position.x, cursor.position.y);
            making_polygon.current_point = current_position;
            making_polygon.path.line_to(current_position);
            making_polygon
                .all_points
                .push(Vec2::new(current_position.x, current_position.y));

            info!("end segment at pos: {:?}", current_position);
            let mesh = meshes.get_mut(&mesh_handle.0).unwrap();

            let segment = Segment {
                start: making_segment.start,
                end: current_position,
            };

            let segment_meta = get_segment_meta(segment);

            *mesh = Mesh::from(shape::Quad::new(Vec2::new(
                segment_meta.length,
                globals.cutting_segment_thickness,
            )));

            *transform = segment_meta.transform;

            commands.entity(entity).remove::<MakingSegment>();

            start_segment_event_writer.send(StartMakingSegment {
                pos: current_position,
            });
        }
    }
}

// upon pressing Enter, end making the polygon
pub fn end_polygon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut segment_query: Query<(Entity, &mut Transform, &Mesh2dHandle, &MakingSegment)>,
    mut end_polygon_event: EventReader<EndMakingPolygon>,
    mut polygon_query: Query<&mut MakingPolygon>,
    globals: Res<Globals>,
) {
    for _ in end_polygon_event.iter() {
        // spawn last segment

        // There is only one MakingPolygon at a time
        for mut poly in polygon_query.iter_mut() {
            //
            //
            // end point of last segment must be the same as the starting point
            // close the polygon using the last segment

            {
                // There only is one MakingSegment at a time
                let (entity, mut transform, mesh_handle, making_segment) =
                    segment_query.single_mut();
                //
                //
                let segment = Segment {
                    start: making_segment.start,
                    end: poly.starting_point,
                };
                let segment_meta = get_segment_meta(segment);

                let mesh = meshes.get_mut(&mesh_handle.0).unwrap();
                *mesh = Mesh::from(shape::Quad::new(Vec2::new(
                    segment_meta.length,
                    globals.cutting_segment_thickness,
                )));

                *transform = segment_meta.transform;
                commands.entity(entity).remove::<MakingSegment>();
            }

            // add last segment to close the polygon
            poly.path.close();

            let path = poly.path.clone().build();
            println!("path: {:?}", path);
            let (mesh, center_of_mass) = make_polygon_mesh(&path, &globals.polygon_color);

            let fill_transform = Transform::from_translation(center_of_mass.extend(0.0));

            let mut rng = thread_rng();

            // Useless at the moment, but here for future use
            let mat_handle = fill_materials.add(FillMesh2dMaterial {
                color: globals.polygon_color.into(),
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
                .insert(Polygon)
                .insert(MeshMeta {
                    id,
                    path: path.clone(),
                    points: poly.all_points.clone(),
                })
                .id();
        }
    }
}

pub fn make_polygon_mesh(path: &Path, color: &Color) -> (Mesh, Vec2) {
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
            let mut new_pos = Vec2::new(x[0], x[1]);
            new_pos -= center_of_mass;

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

pub struct Segment {
    pub start: Point,
    pub end: Point,
}
pub struct SegmentMeta {
    pub length: f32,
    pub center_of_mass: Vec2,
    pub transform: Transform,
}

pub fn get_segment_meta(segment: Segment) -> SegmentMeta {
    let segment_length = (segment.end - segment.start).length();

    let segment_angle = (segment.end.y - segment.start.y).atan2(segment.end.x - segment.start.x);

    let segment_center_of_mass = Point::new(
        (segment.start.x + segment.end.x) / 2.0,
        (segment.start.y + segment.end.y) / 2.0,
    );

    let mut transform = Transform::default();

    transform.rotation = Quat::from_rotation_z(segment_angle);
    transform.translation = Vec3::new(segment_center_of_mass.x, segment_center_of_mass.y, 10.0);

    SegmentMeta {
        length: segment_length,
        center_of_mass: Vec2::new(segment_center_of_mass.x, segment_center_of_mass.y),
        transform: transform,
    }
}
