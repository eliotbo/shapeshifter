use bevy::prelude::*;

pub struct Globals {
    pub polygon_segment_color: Color,
    pub cutting_segment_thickness: f32,
    pub cutting_segment_color: Color,
}

impl Default for Globals {
    fn default() -> Self {
        Self {
            polygon_segment_color: Color::PINK,
            cutting_segment_thickness: 4.0,
            cutting_segment_color: Color::ORANGE,
        }
    }
}
