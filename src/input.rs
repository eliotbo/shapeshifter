use bevy::{
    input::mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
};

use crate::poly::{EndSedgment, StartMakingPolygon, StartMakingSegment};
use crate::util::*;

pub struct Cursor {
    pub position: Vec2,
    pub pos_relative_to_click: Vec2,
    pub last_click_position: Vec2,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            position: Vec2::ZERO,
            pos_relative_to_click: Vec2::ZERO,
            last_click_position: Vec2::ZERO,
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

pub fn record_mouse_events_system(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_res: ResMut<Cursor>,
    mut windows: ResMut<Windows>,
    cam_transform_query: Query<&Transform, With<OrthographicProjection>>,
    // cam_ortho_query: Query<&OrthographicProjection>,
    // globals: Res<Globals>,
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

        // for ortho in cam_ortho_query.iter() {
        //     scale = ortho.scale;
        // }

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
}

pub fn direct_action(
    // mut action_event_writer: EventWriter<Action>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut start_polygon: EventWriter<StartMakingPolygon>,
    cursor: Res<Cursor>,
) {
    let mouse_pressed = mouse_button_input.pressed(MouseButton::Left);
    let mouse_just_pressed = mouse_button_input.just_pressed(MouseButton::Left);

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

    // only used for pattern matching
    let _pressed_g = keyboard_input.just_pressed(KeyCode::G);
    let _pressed_h = keyboard_input.just_pressed(KeyCode::H);
    let _pressed_s = keyboard_input.just_pressed(KeyCode::S);
    let _pressed_l = keyboard_input.just_pressed(KeyCode::L);
    let _pressed_z = keyboard_input.just_pressed(KeyCode::Z);
    let _pressed_t = keyboard_input.just_pressed(KeyCode::T);
    let _pressed_delete = keyboard_input.just_pressed(KeyCode::Delete);

    // match keys / mouse buttons / mouse wheel combination and send event to corresponding action
    match (
        keyboard_input.pressed(KeyCode::LShift),
        keyboard_input.pressed(KeyCode::LControl),
        keyboard_input.pressed(KeyCode::Space),
    ) {
        (true, false, false) if mouse_just_pressed => start_polygon.send(StartMakingPolygon {
            pos: cursor.position,
        }),

        // TODO: move to mouseclick event router
        (false, true, false) if _pressed_g => {}
        (true, true, false) if _pressed_g => {}
        (false, true, false) if _pressed_h => {}
        (true, true, false) if _pressed_h => {}
        (false, true, false) if _pressed_s => {}
        (false, true, false) if _pressed_l => {}
        (false, true, false) if _pressed_z => {}
        (true, true, false) if _pressed_z => {}
        (false, true, false) if mouse_wheel_up => {}
        (false, true, false) if mouse_wheel_down => {}
        (false, false, false) if _pressed_delete => {}
        (true, false, false) if _pressed_t => {}

        _ => {}
    }
}
