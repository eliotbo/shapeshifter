use bevy::prelude::*;
use lyon::algorithms::hit_test::*;
use lyon::path::FillRule;
use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::Path;

use serde::Deserialize;
use serde::Serialize;

pub struct Globals {
    pub polygon_segment_color: Color,
    pub polygon_color: Color,
    pub cutting_segment_thickness: f32,
    pub cutting_segment_color: Color,
    pub min_turn_angle: f32,
    pub cut_polygon: Color,
}

impl Default for Globals {
    fn default() -> Self {
        Self {
            polygon_segment_color: Color::PINK,
            polygon_color: Color::PURPLE,
            cutting_segment_thickness: 2.0,
            cutting_segment_color: Color::ORANGE,
            min_turn_angle: core::f32::consts::PI / 25.0,
            cut_polygon: Color::TEAL,
        }
    }
}

pub struct EntityZ {
    pub entity: Entity,
    pub z: f32,
}

// keeps track of the z position of the polygons
pub struct PolyOrder {
    pub entities: Vec<EntityZ>,
}

impl Default for PolyOrder {
    fn default() -> Self {
        PolyOrder { entities: vec![] }
    }
}

impl PolyOrder {
    pub fn add(&mut self, entity: Entity, z: f32) {
        self.entities.push(EntityZ { entity, z });
        self.sort();
    }

    pub fn remove(&mut self, entity: Entity) {
        self.entities.retain(|e| e.entity != entity);
    }

    pub fn sort(&mut self) {
        // sort by z
        self.entities.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());
    }
}

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Rotating {
    pub starting_angle: f32,
}

#[derive(Component)]
pub struct Translating {
    pub starting_pos: Vec2,
}

pub struct DeleteEvent;

pub type MeshId = u64;

#[derive(Component)]
pub struct MeshMeta {
    pub id: MeshId,
    pub path: Path,
    pub points: Vec<Vec2>,
}

impl MeshMeta {
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

// pub fn make_square() -> (Path, Vec<Vec2>) {
//     let mut path = Path::builder();
//     path.begin(point(0.0, 0.0));
//     path.line_to(point(100.0, 0.0));
//     path.line_to(point(100.0, 100.0));
//     path.line_to(point(0.0, 100.0));
//     path.close();
//     let built_path = path.build();

//     let mut points = Vec::new();
//     points.push(Vec2::new(0.0, 0.0));
//     points.push(Vec2::new(100.0, 0.0));
//     points.push(Vec2::new(100.0, 100.0));
//     points.push(Vec2::new(0.0, 100.0));

//     (built_path, points)
// }
