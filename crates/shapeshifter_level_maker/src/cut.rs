use crate::input::Cursor;
use crate::input::*;
use crate::material::*;
// use crate::poly::make_polygon_mesh;
use crate::poly::Polygon;
use crate::util::*;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use lyon::algorithms::area::*;
use lyon::algorithms::math::Vector;
use lyon::algorithms::raycast::*;
use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::Path;
// use lyon::algorithms::raycast::*;

use rand::{thread_rng, Rng};

#[derive(Component)]
pub struct MakingCutSegment {
    pub start: Vec2,
}

#[derive(Component)]
pub struct CutSegment;

#[derive(Component)]
pub struct JustMadeCut {
    pub segment: Segment,
}

#[derive(Debug, Clone)]
pub enum PolyPoint {
    Original(Point),
    Intersect(Point),
}

impl PolyPoint {
    pub fn is_intersect(&self) -> bool {
        if let PolyPoint::Intersect(_) = self {
            true
        } else {
            false
        }
    }
}

pub struct CutPlugin;

impl Plugin for CutPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(start_cut_segment)
            .add_system(end_cut_segment)
            .add_system(making_cut_segment)
            .add_system(move_after_cut)
            .add_system(perform_cut);
    }
}

pub fn start_cut_segment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // query: Query<Entity, With<MakingCutSegment>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    // mut start_segment_event_reader: EventReader<StartMakingCutSegment>,
    globals: Res<Globals>,
    cursor: Res<Cursor>,
    mut action_event_reader: EventReader<Action>,
) {
    // for start_segment in start_segment_event_reader.iter() {
    if let Some(Action::StartMakingCutSegment { start }) = action_event_reader.iter().next() {
        // let start = Vec2::new(start_segment.start.x, start_segment.start.y);
        // info!("start_cut_segment: {:?}", start);
        let segment = Segment {
            start: Point::new(start.x, start.y),

            end: cursor.clone().into(),
        };

        let segment_meta = get_segment_meta(segment);

        let mesh = bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            segment_meta.length,
            globals.cutting_segment_thickness,
        )))));

        let material = materials.add(globals.cutting_segment_color.into());
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh,
                material,
                transform: segment_meta.transform,
                ..Default::default()
            })
            .insert(MakingCutSegment { start: *start })
            .insert(CutSegment);
    }
}

// changes the position of the segment according to mouse position
pub fn making_cut_segment(
    mut query: Query<(&mut Transform, &Mesh2dHandle, &MakingCutSegment)>,
    mut meshes: ResMut<Assets<Mesh>>,
    globals: Res<Globals>,
    cursor: Res<Cursor>,
) {
    for (mut transform, mesh_handle, making_segment) in query.iter_mut() {
        // info!("making polygon");
        let mesh = meshes.get_mut(&mesh_handle.0).unwrap();

        let segment = Segment {
            start: Point::new(making_segment.start.x, making_segment.start.y),
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

pub fn end_cut_segment(
    mut commands: Commands,
    mut segment_query: Query<(Entity, &mut Transform, &Mesh2dHandle, &MakingCutSegment)>,

    mut meshes: ResMut<Assets<Mesh>>,
    globals: Res<Globals>,
    // cursor: Res<Cursor>,
    // mut end_segment_event_reader: EventReader<EndCutSegment>,
    mut action_event_reader: EventReader<Action>,
) {
    //
    // move one end of the segment to the cursor position

    // for end_segment in end_segment_event_reader.iter() {
    if let Some(Action::EndCutSegment { end }) = action_event_reader.iter().next() {
        for (entity, mut transform, mesh_handle, making_segment) in segment_query.iter_mut() {
            // info!("end cut segment at pos: {:?}", current_position);
            // let current_position = point(cursor.position.x, cursor.position.y);
            let mesh = meshes.get_mut(&mesh_handle.0).unwrap();

            let segment = Segment {
                start: Point::new(making_segment.start.x, making_segment.start.y),
                end: Point::new(end.x, end.y),
            };

            let segment_meta = get_segment_meta(segment.clone());

            *mesh = Mesh::from(shape::Quad::new(Vec2::new(
                segment_meta.length,
                globals.cutting_segment_thickness,
            )));

            *transform = segment_meta.transform;
            commands.entity(entity).remove::<MakingCutSegment>();
            commands.entity(entity).insert(JustMadeCut { segment });
        }
    }
}

pub fn move_after_cut(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut ForceMotion, &mut MeshMeta)>,
    globals: Res<Globals>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut animation, mut mesh_meta) in query.iter_mut() {
        //
        // compute new velocity
        animation.velocity = animation.velocity
            - animation.velocity.normalize() * globals.friction * time.delta_seconds();

        //
        // compute new position
        animation.position = animation.position + animation.velocity * time.delta_seconds();

        transform.translation = animation.position.extend(transform.translation.z);

        mesh_meta.previous_transform = transform.clone();

        // *transform =

        if animation.velocity.length() < globals.min_velocity {
            commands.entity(entity).remove::<ForceMotion>();
        }
    }
}

pub fn perform_cut(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,

    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    cut_query: Query<(Entity, &JustMadeCut)>,
    mut polygon_query: Query<
        (Entity, &Handle<FillMesh2dMaterial>, &Transform, &MeshMeta),
        With<Polygon>,
    >,
) {
    for (cut_entity, cut) in cut_query.iter() {
        let mut do_remove_cut_entity = true;
        for (poly_entity, _material_handle, transform, mesh_meta) in polygon_query.iter_mut() {
            //
            commands.entity(cut_entity).remove::<JustMadeCut>();
            //
            let direction = Vector::from(cut.segment.end - cut.segment.start);
            // make a ray
            let ray = Ray {
                origin: cut.segment.start,
                direction: direction,
            };

            let (transformed_path, _) = transform_path(&mesh_meta.path, transform);

            let maybe_hit = raycast_path(&ray, transformed_path.clone().iter(), 0.1);

            // only compute the cut if the ray from the cut hits the polygon
            if let None = maybe_hit {
                continue;
            }
            let mut points: Vec<PolyPoint> = Vec::new();

            for (k, segs) in transformed_path.iter().enumerate() {
                if k == 0 {
                    continue;
                }

                // println!("point: {:?}", mesh_meta.points[k - 1]);
                // info!("seg: {:?}", segs);
                let segment = Segment {
                    start: segs.from(),
                    end: segs.to(),
                };

                let intersection = segment.intersect(cut.segment.clone());

                points.push(PolyPoint::Original(segs.from()));
                if let Some(intersection) = intersection {
                    points.push(PolyPoint::Intersect(intersection));
                }
            }

            let mut only_intersects: Vec<(usize, &Point)> = points
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_intersect())
                .map(|(k, x)| {
                    if let PolyPoint::Intersect(y) = x {
                        (k, y)
                    } else {
                        panic!("should not happen")
                    }
                })
                // .cloned()
                .collect();

            // println!("only_intersects: {:#?}", only_intersects);

            // if the number of intersection is odd, the cut is invalid because,
            // the polygon cannot be separated properly
            let num_intersects = only_intersects.len();
            if num_intersects % 2 == 1 || num_intersects < 2 {
                println!("nope, number of intersects: {}", num_intersects);
                // remove the cut entity

                // // visual check for whether the intersects are positioned and sorted correctly
                // show_intersects(
                //     &mut commands,
                //     &mut meshes,
                //     &only_intersects,
                //     &mut fill_materials,
                // );

                continue;
            }

            // detect along which axis the intersections vary most from each other,
            // so that this axis can be used to sort the intersections
            let delta = (*only_intersects[1].1 - *only_intersects[0].1).abs();

            // sort intersects by distance along the cut segment
            only_intersects.sort_by(|(_, a), (_, b)| {
                if delta.x > delta.y {
                    a.x.partial_cmp(&b.x).unwrap()
                } else {
                    a.y.partial_cmp(&b.y).unwrap()
                }
            });

            // make pairs of intersects
            let mut pairs: Vec<(usize, usize)> = Vec::new();
            for (k, inter) in only_intersects.iter().enumerate() {
                if k % 2 == 0 {
                    continue;
                }

                let prev = only_intersects[k - 1].0;
                let curr = inter.0;

                pairs.push((prev, curr));
            }

            // take only the indices of intersects
            let intersects_inds: Vec<usize> = only_intersects.iter().map(|x| x.0).collect();

            let num_points_in_cut_poly = points.len();

            // let new_polygons: BinaryTree<Vec<PolyPoint>> = BinaryTree::new(points.clone());

            // polygons to operate on at every loop iteration
            let mut polys: Vec<Vec<usize>> = vec![(0..num_points_in_cut_poly).collect()];

            // polygons that are known to be closed
            let mut closed_polys: Vec<Vec<PolyPoint>> = Vec::new();

            for (idx, pair) in pairs.iter().enumerate() {
                // info!("pair idx: {:?}", idx);

                let k0 = pair.0;
                let k1 = pair.1;

                let rest_of_intersects_inds = intersects_inds.clone().split_off((idx + 1) * 2);

                let mut new_polys_to_explore: Vec<Vec<usize>> = vec![];
                for poly in polys.clone() {
                    if poly_contains_intersect(&poly, &vec![k0, k1]) {
                        // info!("poly contains intersect : {:?}", &vec![k0, k1]);
                        // check if poly contains the current intersects
                        let (poly_a, poly_b) = split_poly_at(&poly, k0, k1);
                        // let (pa, pb) = get_split_poly(&poly, k0, k1);

                        // check if the new polys contains any of the remaining intersect indices
                        // if it does, then it is not closed
                        let do_poly_a = poly_contains_intersect(&poly_a, &rest_of_intersects_inds);
                        let do_poly_b = poly_contains_intersect(&poly_b, &rest_of_intersects_inds);

                        if do_poly_a {
                            new_polys_to_explore.push(poly_a);
                        } else {
                            closed_polys.push(get_poly_points(&poly_a, &points));
                        }
                        if do_poly_b {
                            new_polys_to_explore.push(poly_b);
                        } else {
                            closed_polys.push(get_poly_points(&poly_b, &points));
                        }
                    }
                }

                polys = new_polys_to_explore;
            }

            let mut area_test_passed = true;
            let mut new_entities = Vec::new();

            //
            //
            // crate one path for every closed polygon
            for poly in closed_polys {
                //
                //
                // convert PolyPoint to Point for every element of poly
                let poly_points: Vec<Point> = poly
                    .iter()
                    .map(|x| match x {
                        PolyPoint::Original(p) => *p,
                        PolyPoint::Intersect(i) => *i,
                    })
                    .collect();

                //
                //
                //
                // initialize the new path
                let mut path = Path::builder();

                let mut all_points = vec![];
                for (k, point) in poly_points.iter().enumerate() {
                    if k == 0 {
                        path.begin(*point);
                        all_points.push(Vec2::new(point.x, point.y));
                    } else {
                        path.line_to(*point);
                        all_points.push(Vec2::new(point.x, point.y));
                    }
                }
                path.close();
                let built_path = path.build();

                let area = approximate_signed_area(0.1, &built_path);
                if area.abs() < 200.0 {
                    area_test_passed = false;
                    info!("area too small: {:?}", area);
                    break;
                }

                let (mesh, center_of_mass) = make_polygon_mesh(&built_path, true);

                // Useless at the moment, but here for future use
                let mat_handle = fill_materials.add(FillMesh2dMaterial {
                    color: Color::TEAL.into(),
                    show_com: 0.0, // show center of mass
                    selected: 0.0,
                    is_intersecting: 0.0,
                });

                let translation =
                    lyon::geom::Translation::new(-center_of_mass.x, -center_of_mass.y);
                let transformed_path = built_path.transformed(&translation);

                let mut rng = thread_rng();
                // let id = rng.gen::<u64>();

                let fill_transform =
                    Transform::from_translation(center_of_mass.extend(rng.gen::<f32>() + 1.0));

                // compute normal vector to direction
                let mut cut_normal = Vec2::new(-direction.y, direction.x);
                // compute sign of normal using the center of mass of the cut polygon and the cut starting ppoint
                let sign = cut_normal
                    .dot(Vec2::new(
                        center_of_mass.x - cut.segment.start.x,
                        center_of_mass.y - cut.segment.start.y,
                    ))
                    .signum();
                cut_normal = cut_normal * sign;

                let new_entity = commands
                    .spawn_bundle(MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(mesh)),
                        material: mat_handle,
                        transform: fill_transform,
                        ..Default::default()
                    })
                    .insert(Polygon)
                    .insert(MeshMeta {
                        id: rng.gen::<u64>(),
                        path: transformed_path.clone(),
                        // move points towards the origin
                        points: all_points
                            .clone()
                            .iter()
                            .map(|x| *x - center_of_mass)
                            .collect(),
                        previous_transform: fill_transform,
                        is_intersecting: false,
                        name: "".to_string(),
                    })
                    .insert(ForceMotion {
                        force: Vec2::new(0.0, 0.0),
                        area,
                        velocity: cut_normal * 0.05,
                        position: center_of_mass,
                    })
                    .id();

                new_entities.push(new_entity);

                //
                //
                // TODO: contour
                //
                //
                // commands.spawn_bundle(PathBundle {
                //     path,
                //     style: PathStyle {
                //         stroke_width: 1.0,
                //         stroke_color: Color::rgb(r, b, g),
                //         fill_color: Color::rgb(r, b, g),
                //     },
                //     ..Default::default()
                // });
            }

            // remove the polygon that was cut
            if area_test_passed {
                commands.entity(poly_entity).despawn();
                do_remove_cut_entity = false;
                // return;
            } else {
                // remove all newly created polygons
                for entity in new_entities.iter() {
                    commands.entity(*entity).despawn();
                }
            }
        }
        if do_remove_cut_entity {
            commands.entity(cut_entity).despawn();
        }
    }
}

// pub fn get_split_poly(l: usize, k0: usize, k1: usize) -> Vec<usize> {
//     if k0 > k1 {
//         let mut temp = (k0..l).collect::<Vec<usize>>();
//         temp.extend((0..k1 + 1).collect::<Vec<usize>>());
//         temp
//     } else {
//         (k0..k1 + 1).collect::<Vec<usize>>()
//     }
// }

pub fn get_split_poly(indices: &Vec<usize>, k0: usize, k1: usize) -> (Vec<usize>, Vec<usize>) {
    // find the index of k0 and k1 in indices
    let mut k0_idx = 0;
    let mut k1_idx = 0;

    for (idx, ind) in indices.iter().enumerate() {
        if *ind == k0 {
            k0_idx = idx;
        }
        if *ind == k1 {
            k1_idx = idx;
        }
    }

    let poly_a = if k0_idx > k1_idx {
        let mut temp = indices[k0_idx..].to_vec();
        temp.extend(indices[..k1_idx + 1].to_vec());
        temp
    } else {
        indices[k0_idx..k1_idx + 1].to_vec()
    };

    let poly_b = if k0_idx > k1_idx {
        let mut temp = indices[..k1_idx + 1].to_vec();
        temp.extend(indices[k0_idx..].to_vec());
        temp
    } else {
        let mut temp = indices[..k0_idx].to_vec();
        temp.extend(indices[k1_idx..].to_vec());
        temp
    };

    (poly_a, poly_b)
}

// TODO: figure out how to to do, but with ranges instead loops (see get_split_poly)
pub fn split_poly_at(indices: &Vec<usize>, k0: usize, k1: usize) -> (Vec<usize>, Vec<usize>) {
    let mut poly_a = Vec::new();
    let mut poly_b = Vec::new();

    // find the index of k0 in indices
    let mut k0_idx = 0;

    for (idx, ind) in indices.iter().enumerate() {
        if *ind == k0 {
            k0_idx = idx;
        }
    }

    let mut idxa = k0_idx;
    while indices[idxa] != k1 {
        poly_a.push(indices[idxa]);
        idxa = (idxa + 1) % indices.len();
    }

    poly_a.push(indices[idxa]);

    // k = k0 as i32;
    let mut idxa = k0_idx;
    while indices[idxa] != k1 {
        poly_b.push(indices[idxa]);

        if idxa == 0 {
            idxa = indices.len() - 1;
        } else {
            idxa = idxa - 1;
        }
    }

    poly_b.push(indices[idxa]);

    (poly_a, poly_b)
}

pub fn poly_contains_intersect(poly: &Vec<usize>, intersects: &Vec<usize>) -> bool {
    for k in poly {
        if intersects.contains(&k) {
            return true;
        }
    }

    false
}

pub fn get_poly_points(poly: &Vec<usize>, points: &Vec<PolyPoint>) -> Vec<PolyPoint> {
    let mut poly_points = Vec::new();

    for k in poly {
        poly_points.push(points[*k].clone());
    }

    poly_points
}

pub fn show_intersects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    only_intersects: &Vec<(usize, &Point)>,
    fill_materials: &mut ResMut<Assets<FillMesh2dMaterial>>,
) {
    // visual check for whether the intersects are positioned and sorted correctly
    let mut rng = thread_rng();
    let mut r = rng.gen::<f32>();
    let mut b = rng.gen::<f32>();
    let mut g = rng.gen::<f32>();
    for (k, inter) in only_intersects.iter().enumerate() {
        let pos = Vec2::new(inter.1.x, inter.1.y);

        if k % 2 == 0 {
            // println!("odd");
            r = rng.gen::<f32>();
            b = rng.gen::<f32>();
            g = rng.gen::<f32>();
        }

        let ends_mesh_handle =
            bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(Vec2::new(5., 5.)))));

        let mat_handle = fill_materials.add(FillMesh2dMaterial {
            color: Vec4::new(r, b, g, 1.),
            show_com: 0.0, // show center of mass
            selected: 0.0,
            is_intersecting: 0.0,
        });
        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: ends_mesh_handle.clone(),
            material: mat_handle,
            transform: Transform::from_translation(pos.extend(200.0)),
            ..Default::default()
        });
    }
}
