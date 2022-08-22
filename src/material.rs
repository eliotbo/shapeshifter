use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Component, Default)]
#[uuid = "f690fdae-d598-45ab-1684-97e2a3f95a9a"]
pub struct FillMesh2dMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[uniform(0)]
    pub show_com: f32,
    #[uniform(0)]
    pub selected: f32,
}

pub struct FillMesh2dPlugin;

impl Plugin for FillMesh2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<FillMesh2dMaterial>::default());
    }
}

impl Material2d for FillMesh2dMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/polygon.wgsl".into()
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Component, Default)]
#[uuid = "f690fdae-d512-45ab-1663-9678a3f95a9a"]
pub struct CutMesh2dMaterial {
    #[uniform(0)]
    pub color: Vec4,
}

pub struct CutMesh2dPlugin;

impl Plugin for CutMesh2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<FillMesh2dMaterial>::default());
    }
}

impl Material2d for CutMesh2dMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/cut.wgsl".into()
    }
}
