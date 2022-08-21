use bevy::prelude::*;
use lyon::tessellation::path::{builder::NoAttributes, Path};

use serde::Deserialize;
use serde::Serialize;

pub struct Globals {
    pub polygon_segment_color: Color,
    pub polygon_color: Color,
    pub cutting_segment_thickness: f32,
    pub cutting_segment_color: Color,
}

impl Default for Globals {
    fn default() -> Self {
        Self {
            polygon_segment_color: Color::PINK,
            polygon_color: Color::PURPLE,
            cutting_segment_thickness: 4.0,
            cutting_segment_color: Color::ORANGE,
        }
    }
}

pub struct DeleteEvent;

pub type MeshId = u64;

#[derive(Component)]
pub struct MeshMeta {
    pub id: MeshId,
    pub path: Path,
    pub points: Vec<Vec2>,
}

#[derive(Serialize, Deserialize)]
pub struct SaveMeshMeta {
    pub points: Vec<Vec2>,
}

impl From<&MeshMeta> for SaveMeshMeta {
    fn from(mesh_meta: &MeshMeta) -> Self {
        Self {
            points: mesh_meta.points.clone(),
        }
    }
}
