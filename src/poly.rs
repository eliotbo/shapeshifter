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

pub fn make_square() -> Path {
    let mut path = Path::builder();
    path.begin(point(0.0, 0.0));
    path.line_to(point(100.0, 0.0));
    path.line_to(point(100.0, 100.0));
    path.line_to(point(0.0, 100.0));
    path.close();
    path.build()
}

#[derive(Component)]
pub struct MakingSegment {
    pub start: Point,
}
#[derive(Component)]
pub struct MakingPolygon {
    path: NoAttributes<BuilderImpl>,
}

// events
pub struct StartMakingPolygon {
    pub pos: Vec2,
}
pub struct StartMakingSegment {
    pub pos: Vec2,
}
pub struct EndSedgment {
    pub pos: Vec2,
}

pub struct EndMakingPolygon;

pub fn start_polygon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,

    mut materials: ResMut<Assets<ColorMaterial>>,
    mut start_polygon_event_reader: EventReader<StartMakingPolygon>,
    globals: Res<Globals>,
) {
    for start_poly in start_polygon_event_reader.iter() {
        info!("start polygon:  {:?}", start_poly.pos);
        let start = point(start_poly.pos.x, start_poly.pos.y);
        let segment = Segment {
            start,
            end: point(-200.0, -200.0),
        };

        let segment_meta = get_segment_meta(segment);

        let mesh = bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            segment_meta.length,
            globals.cutting_segment_thickness,
        )))));

        let material = materials.add(globals.polygon_segment_color.into());
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh,
                material,
                transform: segment_meta.transform,
                ..Default::default()
            })
            .insert(MakingPolygon {
                path: Path::builder(),
            });
    }
}

// upon pressing Enter, end making the polygon
pub fn end_polygon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut end_polygon_event: EventReader<EndMakingPolygon>,
) {
    for ev in end_polygon_event.iter() {
        // make mesh from path information
    }
}

pub fn make_polygon_mesh(path: &Path, color: &Color) -> (Mesh, Vec2) {
    let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

    {
        let mut vertex_builder = simple_builder(&mut buffers);

        // Create the tessellator.
        let mut tessellator = FillTessellator::new();

        // Compute the tessellation.
        let result =
            tessellator.tessellate_path(path, &FillOptions::default(), &mut vertex_builder);
        assert!(result.is_ok());
    }

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
            info!("{:?} ", new_pos);
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

    fn bounds(v: &Vec<f32>) -> (f32, f32) {
        let max_v: &f32 = v
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap();

        let min_v: &f32 = v
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap();

        return (*min_v, *max_v);
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
