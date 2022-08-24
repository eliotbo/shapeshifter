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
    pub target_color: Color,
    pub min_turn_angle: f32,
    pub cut_polygon: Color,
    pub min_velocity: f32,
    pub friction: f32,
    pub snap_to_grid: bool,
    pub grid_size: f32,
    pub target_size_multiplier: f32,
}

impl Default for Globals {
    fn default() -> Self {
        Self {
            polygon_segment_color: Color::PINK,
            polygon_color: Color::PURPLE,
            cutting_segment_thickness: 2.0,
            cutting_segment_color: Color::ORANGE,
            target_color: Color::DARK_GREEN,
            min_turn_angle: core::f32::consts::PI / 25.0,
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
pub struct Grid;

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

pub struct TestWinEvent;

pub struct DeleteEvent;

pub struct TestCollisionEvent(pub Entity);

pub type MeshId = u64;

#[derive(Component, Clone)]
pub struct MeshMeta {
    pub id: MeshId,
    pub path: Path,
    pub points: Vec<Vec2>,
    pub previous_transform: Transform,
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
