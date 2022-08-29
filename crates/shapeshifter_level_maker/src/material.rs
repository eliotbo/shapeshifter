use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

// #[derive(AsBindGroup, TypeUuid, Debug, Clone, Component, Default)]
// #[uuid = "f690fdae-d598-1133-5566-97e2a3f95a9a"]
// pub struct FeltMaterial {
//     #[uniform(0)]
//     pub color: Vec4,
//     #[uniform(0)]
//     pub show_com: f32,
//     #[uniform(0)]
//     pub selected: f32,
//     #[uniform(0)]
//     pub is_intersecting: f32,
//     //
//     #[texture(1)]
//     #[sampler(2)]
//     pub felt: Handle<Image>,
// }

// pub struct FeltPlugin;

// impl Plugin for FeltPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_plugin(Material2dPlugin::<FeltMaterial>::default());
//     }
// }

// impl Material2d for FeltMaterial {
//     fn fragment_shader() -> ShaderRef {
//         "shaders/felt.wgsl".into()
//     }
// }

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Component, Default)]
#[uuid = "f690fdae-d598-45ab-1684-97e2a3f95a9a"]
pub struct FillMesh2dMaterial {
    #[uniform(0)]
    pub color: Vec4,
    #[uniform(0)]
    pub show_com: f32,
    #[uniform(0)]
    pub selected: f32,
    #[uniform(0)]
    pub is_intersecting: f32,
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

//
//
//
#[derive(AsBindGroup, TypeUuid, Debug, Clone, Component, Default)]
#[uuid = "f690fdae-d512-fd79-9517-9678a3f95537"]
pub struct TargetMesh2dMaterial {
    #[uniform(0)]
    pub color: Vec4,
}

pub struct TargetMesh2dPlugin;

impl Plugin for TargetMesh2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<TargetMesh2dMaterial>::default());
    }
}

impl Material2d for TargetMesh2dMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/target.wgsl".into()
    }
}
