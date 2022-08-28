use bevy::prelude::*;

use crate::input::*;
use crate::material::*;
use crate::util::*;

use lyon::algorithms::hit_test::*;
use lyon::path::FillRule;
use lyon::tessellation::math::Point;

pub enum PossibleMoves {
    Translation(Vec2),
    Rotation(f32),
}

//
//
//
// make polygon glow upon hover and insert Rotating (right mouse click) or
// Translating (left mouse click) component
//
//
// TODO: move inputs to input
pub fn glow_poly(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<
        (
            Entity,
            &Handle<FillMesh2dMaterial>,
            &Transform,
            &MeshMeta,
            Option<Or<(&MaybeRotating, &Translating)>>,
        ),
        With<Polygon>,
    >,
    mut materials: ResMut<Assets<FillMesh2dMaterial>>,
) {
    // TODO: move these to inputs
    let left_mouse_click = mouse_button_input.just_pressed(MouseButton::Left);
    let right_mouse_click = mouse_button_input.just_pressed(MouseButton::Right);
    // let mut moving_entity = None;
    let mut maybe_highlight_entity = None;

    let mut maybe_move_entity: Option<(Entity, PossibleMoves)> = None;

    let ctrl = keyboard_input.pressed(KeyCode::LControl);
    let shift = keyboard_input.pressed(KeyCode::LShift);
    let pressed_q = keyboard_input.pressed(KeyCode::Q);

    for (entity, material_handle, transform, mesh_meta, maybe_moving) in query.iter() {
        //
        //
        //

        //
        // let (transformed_path, angle) = transform_path(&mesh_meta.path, transform);
        //
        //
        //

        //
        // let is_inside_poly = hit_test_path(
        //     &cursor.clone().into(),
        //     transformed_path.iter(),
        //     FillRule::EvenOdd,
        //     0.1,
        // );

        let (is_inside_poly, angle) = mesh_meta.hit_test(&cursor.clone().into(), &transform);

        let mut material = materials.get_mut(&material_handle).unwrap();
        material.show_com = 0.0;

        if is_inside_poly && left_mouse_click && !ctrl && !shift && !pressed_q {
            maybe_move_entity = Some((
                entity,
                PossibleMoves::Translation(transform.translation.truncate()),
            ));
        }

        if is_inside_poly && right_mouse_click && !ctrl && !shift && !pressed_q {
            maybe_move_entity = Some((entity, PossibleMoves::Rotation(angle)));
        }

        if let Some(_) = maybe_moving {
            maybe_highlight_entity = Some(entity);
        } else if is_inside_poly {
            maybe_highlight_entity = Some(entity);
        }
    }

    // add Rotating or Translating component to clicked entity
    //

    // TODO: prioritize higher z pos
    if !pressed_q {
        if let Some((entity, moves)) = maybe_move_entity {
            let (_, material_handle, _, _, _) = query.get(entity).unwrap();
            let mut material = materials.get_mut(&material_handle).unwrap();
            match moves {
                PossibleMoves::Translation(translation) => {
                    commands.entity(entity).insert(Translating {
                        starting_pos: translation,
                    });
                }
                PossibleMoves::Rotation(angle) => {
                    commands.entity(entity).insert(MaybeRotating {
                        starting_angle: angle,
                    });
                }
            }

            material.show_com = 1.0;
        } else if let Some(highlighted_entity) = maybe_highlight_entity {
            //
            // if no movement is happening, highlight one entity that is hovered over
            if let Ok((_, material_handle, _, _, _)) = query.get(highlighted_entity) {
                let mut material = materials.get_mut(&material_handle).unwrap();
                material.show_com = 1.0;
            }
        }
    }
}

//
// if user right clicked on a polygon and dragged the mouse further than 100 px aways,
// this system with be called and the polygon will turn
pub fn rotate_poly(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
    // keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &mut Transform, &Rotating), With<Polygon>>,
    // mut materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut collision_test_writer: EventWriter<TestCollisionEvent>,
) {
    for (_, mut transform, rotating) in query.iter_mut() {
        // let v1 = (cursor.last_right_click_position - transform.translation.truncate()).normalize();

        let v1 = rotating.mouse_vec;
        let v2 = cursor.position - cursor.last_right_click_position;
        let delta_angle = v1.angle_between(v2);
        println!("{:?}", delta_angle);

        let new_angle = delta_angle + rotating.starting_angle;
        transform.rotation = Quat::from_rotation_z(new_angle);
    }

    if mouse_button_input.just_released(MouseButton::Right)
        || mouse_button_input.just_pressed(MouseButton::Left)
    {
        // remove Rotating
        for (entity, _, _) in query.iter_mut() {
            commands.entity(entity).remove::<Rotating>();
            collision_test_writer.send(TestCollisionEvent(entity));
        }
    }
}

// TODO: get rid of cursor here
// translate and rotate Polygon using right mouse button
pub fn transform_poly(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
    mut queries: ParamSet<(
        // Query<(Entity, &mut Transform, &MaybeRotating, &MeshMeta), With<Polygon>>,
        Query<(Entity, &MaybeRotating), With<Polygon>>,
        Query<(Entity, &mut Transform, &Translating), With<Polygon>>,
    )>,
    // globals: Res<Globals>,
    mut collision_test_writer: EventWriter<TestCollisionEvent>,
) {
    // for (entity, mut transform, maybe_rotating, _) in queries.p0().iter_mut() {
    for (entity, maybe_rotating) in queries.p0().iter_mut() {
        // //
        // //
        // // rotate using the center of mass of the polygon vs the mouse position
        // let v1 = (cursor.last_right_click_position - transform.translation.truncate()).normalize();
        // let v2 = (cursor.position - transform.translation.truncate()).normalize();
        // let delta_angle = v1.angle_between(v2);

        let mouse_vec = cursor.position - cursor.last_right_click_position;

        if mouse_vec.length() > 100.0 {
            commands
                .entity(entity)
                .insert(Rotating {
                    starting_angle: maybe_rotating.starting_angle,
                    mouse_vec,
                })
                .remove::<MaybeRotating>();
        }
        //
        //
        // rotate using the y axis
        // let diag_mouse_dist = cursor.position.y - cursor.last_right_click_position.y;
        // let free_angle = -diag_mouse_dist * 0.0035 + rotating.starting_angle;

        // let free_angle = delta_angle + rotating.starting_angle;

        // let angle = free_angle;
        // transform.rotation = Quat::from_rotation_z(angle);
    }

    for (_, mut transform, translating) in queries.p1().iter_mut() {
        //
        //
        let mouse_delta = cursor.position - cursor.last_click_position;
        transform.translation =
            (translating.starting_pos + mouse_delta).extend(transform.translation.z);
    }

    // upon release the mouse button, remove the Translating or Rotating component
    // and check for collisions
    if mouse_button_input.just_released(MouseButton::Left) {
        // remove Translating
        for (entity, _, _) in queries.p1().iter_mut() {
            commands.entity(entity).remove::<Translating>();
            collision_test_writer.send(TestCollisionEvent(entity));
            // info!("sending collision after translating");
        }
    }

    if mouse_button_input.just_released(MouseButton::Right)
        || mouse_button_input.just_pressed(MouseButton::Left)
    {
        // remove Rotating
        for (entity, _) in queries.p0().iter_mut() {
            commands.entity(entity).remove::<MaybeRotating>();
            collision_test_writer.send(TestCollisionEvent(entity));
        }
    }
}

// Rotates polygon upon mousewheel event
pub fn rotate_once(
    mut query: Query<(Entity, &mut Transform, &MeshMeta), With<Polygon>>,
    mut action_event_reader: EventReader<Action>,
    globals: Res<Globals>,
    mut collision_test_writer: EventWriter<TestCollisionEvent>,
) {
    // triggered by mousewheel
    if let Some(Action::RotateAt { pos, dir }) = action_event_reader.iter().next() {
        for (entity, mut transform, mesh_meta) in query.iter_mut() {
            //
            //
            let (transformed_path, angle) = transform_path(&mesh_meta.path, transform.as_ref());

            let cursor_point = Point::new(pos.x, pos.y);

            // if the position of the cursor is inside the polygon, rotate it by a minimal amount
            if hit_test_path(
                &cursor_point,
                transformed_path.iter(),
                FillRule::EvenOdd,
                0.1,
            ) {
                transform.rotation = Quat::from_rotation_z(angle + dir * globals.min_turn_angle);
                collision_test_writer.send(TestCollisionEvent(entity));

                return;
            }
        }
    }
}
