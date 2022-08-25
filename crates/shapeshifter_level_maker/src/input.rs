use bevy::{
    input::mouse::{MouseButton, MouseWheel},
    prelude::*,
};

use crate::cut::*;
// use crate::load::QuickLoad;
use crate::poly::{MakingPolygon, MakingSegment};
// use crate::save::SaveMeshEvent;
use crate::util::{Globals, MovingPathPoint};
// use crate::util::Globals;

use lyon::tessellation::math::Point;

#[derive(Debug, PartialEq)]
pub enum Action {
    StartMakingPolygon { pos: Point },
    EndMakingPolygon,
    StartMakingSegment { pos: Point },
    EndSegment { pos: Point },
    StartMakingCutSegment { start: Vec2 },
    EndCutSegment { end: Vec2 },
    RotateAt { pos: Vec2, dir: f32 },
    AddPointAt { pos: Vec2 },
    DeleteMakingPoly,
    DeleteSelected,
    SelectPoly { pos: Vec2, keep_selected: bool },
    DeleteAll,
    ToggleGrid,
    QuickLoadTarget { maybe_name: Option<String> },
    MaybeTranslatePoly,
    MaybeRotatePoly,
    RevertToInit,
    SaveOneDialog,
    LoadDialog,
    QuickLoad { maybe_name: Option<String> },
    QuickLoadAll, // no bindings yet
    QuickSave,
    LoadTarget,
    MovePathPoint,
    DeleteTarget,
}

#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    pub position: Vec2,
    pub pos_relative_to_click: Vec2,
    pub last_click_position: Vec2,
    pub last_right_click_position: Vec2,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            position: Vec2::ZERO,
            pos_relative_to_click: Vec2::ZERO,
            last_click_position: Vec2::ZERO,
            last_right_click_position: Vec2::ZERO,
        }
    }
}

impl Cursor {
    pub fn within_rect(&self, position: Vec2, size: Vec2) -> bool {
        if self.position.x < position.x + size.x / 2.0
            && self.position.x > position.x - size.x / 2.0
            && self.position.y < position.y + size.y / 2.0
            && self.position.y > position.y - size.y / 2.0
        {
            return true;
        }
        return false;
    }
}

impl Into<Point> for Cursor {
    fn into(self) -> Point {
        Point::new(self.position.x, self.position.y)
    }
}

pub fn record_mouse_events_system(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_res: ResMut<Cursor>,
    mut windows: ResMut<Windows>,
    cam_transform_query: Query<&Transform, With<OrthographicProjection>>,
) {
    for event in cursor_moved_events.iter() {
        let cursor_in_pixels = event.position; // lower left is origin
        let window_size = Vec2::new(
            windows.get_primary_mut().unwrap().width(),
            windows.get_primary_mut().unwrap().height(),
        );

        let screen_position = cursor_in_pixels - window_size / 2.0;

        let cam_transform = cam_transform_query.iter().next().unwrap();

        // this variable currently has no effect
        let scale = 1.0;

        let cursor_vec4: Vec4 = cam_transform.compute_matrix()
            * screen_position.extend(0.0).extend(1.0 / (scale))
            * scale;

        let cursor_pos = Vec2::new(cursor_vec4.x, cursor_vec4.y);
        cursor_res.position = cursor_pos;
        cursor_res.pos_relative_to_click = cursor_res.position - cursor_res.last_click_position;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        cursor_res.last_click_position = cursor_res.position;
        cursor_res.pos_relative_to_click = Vec2::ZERO;
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        cursor_res.last_right_click_position = cursor_res.position;
    }
}

pub fn direct_action(
    mut commands: Commands,
    // mut action_event_writer: EventWriter<Action>,
    making_poly_query: Query<&MakingPolygon>,
    making_cut_query: Query<(Entity, &MakingCutSegment)>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    // mut quickload_event_writer: EventWriter<QuickLoad>,

    // mut start_polygon: EventWriter<StartMakingPolygon>,
    // mut start_segment: EventWriter<EndSegment>,
    // mut start_cut_segment: EventWriter<StartMakingCutSegment>,
    // mut end_polygon: EventWriter<EndMakingPolygon>,
    // mut delete_event: EventWriter<DeleteEvent>,
    mut action_event: EventWriter<Action>,
    // mut quicksave_event_writer: EventWriter<SaveMeshEvent>,
    // mut end_cut_segment: EventWriter<EndCutSegment>,
    cursor: Res<Cursor>,
    globals: ResMut<Globals>,
) {
    // let mouse_pressed = mouse_button_input.pressed(MouseButton::Left);

    let mouse_just_pressed = mouse_button_input.just_pressed(MouseButton::Left);
    let mouse_right_just_pressed = mouse_button_input.just_pressed(MouseButton::Right);
    let mouse_just_released = mouse_button_input.just_released(MouseButton::Left);

    let mut mouse_wheel_up = false;
    let mut mouse_wheel_down = false;
    if let Some(mouse_wheel) = mouse_wheel_events.iter().next() {
        if mouse_wheel.y > 0.5 {
            mouse_wheel_up = true;
        }
        if mouse_wheel.y < -0.5 {
            mouse_wheel_down = true;
        }
    }

    let mut pos = cursor.position;
    if globals.snap_to_grid {
        pos = (pos.clone() / globals.grid_size).round() * globals.grid_size;
    }

    let making_cut = making_cut_query.iter().next().is_some();
    let making_poly = making_poly_query.iter().next().is_some();

    // only used for pattern matching
    let pressing_q = keyboard_input.pressed(KeyCode::Q);

    let pressed_g = keyboard_input.just_pressed(KeyCode::G);
    let _pressed_h = keyboard_input.just_pressed(KeyCode::H);
    let pressed_s = keyboard_input.just_pressed(KeyCode::S);
    let pressed_l = keyboard_input.just_pressed(KeyCode::L);
    let _pressed_z = keyboard_input.just_pressed(KeyCode::Z);
    let pressed_t = keyboard_input.just_pressed(KeyCode::T);
    let pressed_delete = keyboard_input.just_pressed(KeyCode::Delete);
    let pressed_enter = keyboard_input.just_pressed(KeyCode::Return);
    let pressed_escape = keyboard_input.just_pressed(KeyCode::Escape);
    let pressed_space = keyboard_input.just_pressed(KeyCode::Space);
    let pressed_back = keyboard_input.just_pressed(KeyCode::Back);

    // match keys / mouse buttons / mouse wheel combination and send event to corresponding action
    match (
        keyboard_input.pressed(KeyCode::LShift),
        keyboard_input.pressed(KeyCode::LControl),
        keyboard_input.pressed(KeyCode::Space),
    ) {
        (true, true, true) if pressed_delete => action_event.send(Action::DeleteTarget),
        //
        //
        //
        //
        //
        // cut on mouse release
        (_, _, _) if mouse_just_released && making_cut => {
            action_event.send(Action::EndCutSegment { end: pos });
        }
        //
        //
        // revert to initial state
        (_, _, _) if pressed_back => {
            action_event.send(Action::RevertToInit);
        }

        // Start a cut
        // cannot start a cut segment if one is already being made
        (false, true, false) if mouse_just_pressed && making_cut_query.iter().count() == 0 => {
            action_event.send(Action::StartMakingCutSegment { start: pos });
        }

        (_, _, _) if pressed_g => action_event.send(Action::ToggleGrid),

        (false, false, false) if mouse_wheel_up => {
            action_event.send(Action::RotateAt { pos: pos, dir: 1.0 })
        }
        (false, false, false) if mouse_wheel_down => action_event.send(Action::RotateAt {
            pos: pos,
            dir: -1.0,
        }),

        //
        //
        //

        //
        //
        //
        ///////////////////////////////// start of Part of level making  /////////////////////////////
        //
        //
        // ends the current polygon being made
        (false, false, _)
            if (pressed_enter || mouse_right_just_pressed || pressed_space) && making_poly =>
        {
            action_event.send(Action::EndMakingPolygon);
        }

        (false, false, false) if mouse_just_pressed && pressing_q => {
            action_event.send(Action::MovePathPoint);
        }
        //
        //
        // a click ends the current segment
        (false, false, false) if mouse_just_pressed && making_poly => {
            action_event.send(Action::EndSegment {
                pos: cursor.clone().into(),
            });
        }

        (false, true, false) if pressed_s => action_event.send(Action::QuickSave),
        (true, true, false) if pressed_s => action_event.send(Action::SaveOneDialog),
        (true, true, false) if pressed_l => action_event.send(Action::LoadDialog),
        (true, true, false) if pressed_t => action_event.send(Action::LoadTarget),
        (false, true, false) if pressed_l => {
            action_event.send(Action::QuickLoad { maybe_name: None })
        }
        (false, true, false) if pressed_t => {
            action_event.send(Action::QuickLoadTarget { maybe_name: None })
        }

        //
        //
        //
        (false, false, false) if pressed_escape && making_cut => {
            // delete cut segment
            let (entity, _) = making_cut_query.single();
            commands.entity(entity).despawn();
        }

        //
        //
        //
        (false, false, false) if making_poly && (pressed_delete || pressed_escape) => {
            action_event.send(Action::DeleteMakingPoly);
        }

        // cannot start a polygon if one is already being made
        (true, false, false) if (mouse_just_pressed && making_poly_query.iter().count() == 0) => {
            action_event.send(Action::StartMakingPolygon {
                pos: cursor.clone().into(),
            })
        }

        // add point
        (true, false, false) if mouse_right_just_pressed && !making_poly && !making_cut => {
            action_event.send(Action::AddPointAt { pos: pos });
        }

        (true, true, space) if mouse_just_pressed && !making_poly && !making_cut => {
            action_event.send(Action::SelectPoly {
                pos: pos,
                keep_selected: space,
            });
        }

        (true, true, false) if pressed_delete => action_event.send(Action::DeleteAll),

        (_, _, _) if pressed_delete => action_event.send(Action::DeleteSelected),

        (false, false, true) if pressed_space => action_event.send(Action::RotateAt {
            pos,
            dir: -10.0, // use dir to multiply the rotation angle
        }),

        ///////////////////////////////// end of Part of level making  /////////////////////////////

        //
        //
        // low_priority (but still important)
        //
        //
        //
        // translation (q is for moving points in level-making)
        (false, false, false) if mouse_just_pressed && !pressing_q => {
            info!("translation");
            action_event.send(Action::MaybeTranslatePoly)
        }
        //
        //
        //
        // rotation
        (false, false, false) if mouse_right_just_pressed && !pressing_q => {
            info!("rotation");
            action_event.send(Action::MaybeRotatePoly)
        }
        _ => {}
    }
}

pub fn direct_release_action(
    mut commands: Commands,
    segment_query: Query<Entity, With<MakingSegment>>,
    path_point_query: Query<Entity, With<MovingPathPoint>>,
    mouse_button_input: Res<Input<MouseButton>>,
    // mut start_polygon: EventWriter<StartMakingPolygon>,
    // mut action_event_writer: EventWriter<Action>,
    // keyboard_input: Res<Input<KeyCode>>,
    // mut mouse_wheel_events: EventReader<MouseWheel>,
    // cursor: Res<Cursor>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        // delete MakingSegment if it exists
        // for entity in segment_query.iter() {
        //     commands.entity(entity).remove::<MakingSegment>();
        // }

        for entity in path_point_query.iter() {
            commands.entity(entity).remove::<MovingPathPoint>();
            info!("removing MovingPathPoint");
        }
    }
}
