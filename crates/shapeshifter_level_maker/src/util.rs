use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    // sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use lyon::algorithms::hit_test::*;
use lyon::path::FillRule;
use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::Path;

use lyon::tessellation::geometry_builder::simple_builder;

use lyon::tessellation::{FillOptions, FillTessellator, VertexBuffers};

use serde::Deserialize;
use serde::Serialize;

pub struct Globals {
    pub polygon_segment_color: Color,
    pub polygon_color: Color,
    pub cutting_segment_thickness: f32,
    pub cutting_segment_color: Color,
    pub target_color: Color,
    pub min_turn_angle: f32,
    pub cut_polygon: Color,
    pub min_velocity: f32,
    pub friction: f32,
    pub snap_to_grid: bool,
    pub grid_size: f32,
    pub target_size_multiplier: f32,
    pub ghost_color: Color,
}

impl Default for Globals {
    fn default() -> Self {
        Self {
            polygon_segment_color: Color::PINK,
            polygon_color: Color::PURPLE,
            cutting_segment_thickness: 2.0,
            cutting_segment_color: Color::ORANGE,
            target_color: Color::DARK_GREEN,
            ghost_color: Color::rgba(0.02, 0.01, 0.21, 1.0),
            min_turn_angle: core::f32::consts::PI / 20.0,
            cut_polygon: Color::TEAL,
            min_velocity: 0.5,
            friction: 50.0,
            snap_to_grid: false,
            grid_size: 20.0,
            target_size_multiplier: 1.2,
        }
    }
}

#[derive(Component)]
pub struct ForceMotion {
    pub force: Vec2,
    pub area: f32,
    pub velocity: Vec2,
    pub position: Vec2,
}

pub struct EntityZ {
    pub entity: Entity,
    pub z: f32,
}

#[derive(Component)]
pub struct PathPoint;

#[derive(Component)]
pub struct MovingPathPoint {
    pub index: usize,
    pub previous_pos: Vec2,
}

#[derive(Component)]
pub struct Hovered;

#[derive(Component)]
pub struct Grid;

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Ghost;

#[derive(Component)]
pub struct Rotating {
    pub starting_angle: f32,
}

#[derive(Component)]
pub struct Translating {
    pub starting_pos: Vec2,
}

pub struct TestWinEvent;

pub struct DeleteEvent;

pub struct TestCollisionEvent(pub Entity);

pub type MeshId = u64;

#[derive(Component, Clone)]
pub struct MeshMeta {
    pub id: MeshId,
    pub path: Path,
    // pub center_of_mass: Vec2,
    pub points: Vec<Vec2>,
    pub previous_transform: Transform,
}

impl MeshMeta {
    //
    //
    //
    // get the closest point on the path to the given point
    pub fn get_close_from_pos(
        &mut self,
        pos: Vec2,
        transform: &Transform,
        limit: f32,
    ) -> Option<(usize, Vec2)> {
        //
        self.uptdate_points(transform);

        let mut closest_index = None;
        let mut closest_distance = limit;
        for (index, point) in self.points.iter().enumerate() {
            let distance = (*point - pos).length();
            if distance < closest_distance {
                closest_index = Some((index, *point));
                closest_distance = distance;
            }
        }

        closest_index
    }

    // converts a Path to Vec<Vec2> and update its points field
    pub fn uptdate_points(&mut self, transform: &Transform) {
        let (transformed_path, _) = transform_path(&self.path, transform);

        // the first point is the Begin of the path, which is redundant
        let mut iter_over_path = transformed_path.iter();
        iter_over_path.next();

        let mut new_points = Vec::new();

        for seg in iter_over_path {
            //
            //
            let point_pos = match seg {
                lyon::path::Event::Line { from, to: _ } => Vec2::new(from.x, from.y),
                lyon::path::Event::End {
                    last,
                    first: _,
                    close: _,
                } => Vec2::new(last.x, last.y),
                //
                //
                //
                lyon::path::Event::Begin { at: _ } => continue,
                _ => continue,
            };

            new_points.push(point_pos);
        }

        self.points = new_points;
    }

    // Test whether the mouse is inside the polygon
    pub fn hit_test(&self, pos: &Point, transform: &Transform) -> (bool, f32) {
        //
        //
        //
        // the points are at the origin, so we need to take the translation + rotation into account
        // let transformed_path = path.transformed(&rot).transformed(&translation);
        let (transformed_path, angle) = transform_path(&self.path, transform);

        (
            //
            //
            //  The path is now translated and rotated. We can now check whether the mouse in inside the path
            hit_test_path(pos, transformed_path.iter(), FillRule::EvenOdd, 0.1),
            angle,
        )
    }

    //
    //
    //
    // Fast test of overlapping bounding boxes
    pub fn bounding_box_collide(
        &self,
        other: &Path,
        transform: &Transform,
        other_transform: &Transform,
    ) -> bool {
        //
        //
        //
        let (transformed_path, _) = transform_path(&self.path, transform);
        let (transformed_other, _) = transform_path(other, other_transform);

        let bb1 = lyon::algorithms::aabb::fast_bounding_box(&transformed_path);
        let bb2 = lyon::algorithms::aabb::fast_bounding_box(&transformed_other);

        bb1.min.x <= bb2.max.x
            && bb1.max.x >= bb2.min.x
            && bb1.min.y <= bb2.max.y
            && bb1.max.y >= bb2.min.y
    }

    //
    //
    //
    // Test whether the path is intersecting with another path by checking all intersecting segments
    pub fn precise_intersect_test(
        &self,
        other: &Path,
        transform: &Transform,
        other_transform: &Transform,
    ) -> bool {
        let (transformed_path, _) = transform_path(&self.path, transform);
        let (transformed_other, _) = transform_path(other, other_transform);

        for seg in transformed_path.iter() {
            let segment = Segment {
                start: seg.from(),
                end: seg.to(),
            };
            for other_seg in transformed_other.iter() {
                let other_segment = Segment {
                    start: other_seg.from(),
                    end: other_seg.to(),
                };
                if segment.intersect(other_segment).is_some() {
                    return true;
                }
            }
        }
        return false;
    }
}

#[derive(Serialize, Deserialize)]
pub struct SaveMeshMeta {
    pub points: Vec<Vec2>,
    pub translation: Vec2,
    pub rotation: f32,
}

// impl From<&MeshMeta> for SaveMeshMeta {
//     fn from(mesh_meta: &MeshMeta) -> Self {
//         Self {
//             points: mesh_meta.points.clone(),
//         }
//     }
// }

#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub start: Point,
    pub end: Point,
}

impl Segment {
    // function that computes the intersection of two finite segments in 2d
    pub fn intersect(&self, other: Segment) -> Option<Point> {
        let a = self.start;
        let b = self.end;
        let c = other.start;
        let d = other.end;

        let denom = (b.x - a.x) * (d.y - c.y) - (b.y - a.y) * (d.x - c.x);
        if denom == 0.0 {
            return None;
        }

        let nume_a = (a.y - c.y) * (d.x - c.x) - (a.x - c.x) * (d.y - c.y);
        let nume_b = (a.y - c.y) * (b.x - a.x) - (a.x - c.x) * (b.y - a.y);

        if nume_a == 0.0 && nume_b == 0.0 {
            return None;
        }

        let u_a = nume_a / denom;
        let u_b = nume_b / denom;

        if u_a >= 0.0 && u_a <= 1.0 && u_b >= 0.0 && u_b <= 1.0 {
            return Some(point(a.x + u_a * (b.x - a.x), a.y + u_a * (b.y - a.y)));
        }

        return None;
    }
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

// The path is by default centered at the origin, so we need to translate it to the
// position of the entity.
pub fn transform_path(path: &Path, transform: &Transform) -> (Path, f32) {
    let (axis, transform_rotation_angle) = transform.rotation.to_axis_angle();
    let angle = axis.z * transform_rotation_angle;

    let rot = lyon::geom::Rotation::radians(angle);
    let translation =
        lyon::geom::Translation::new(transform.translation.x, transform.translation.y);

    // the points are at the origin, so we need to take the translation + rotation into account
    let transformed_path = path.clone().transformed(&rot).transformed(&translation);

    return (transformed_path, angle);
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

pub fn shift_to_center_of_mass(v: &Vec<Vec2>) -> Vec<Vec2> {
    let mut center_of_mass = Vec2::ZERO;
    for pos in v.iter() {
        center_of_mass += *pos;
    }
    center_of_mass /= v.len() as f32;
    return v.iter().map(|x| *x - center_of_mass).collect();
}

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