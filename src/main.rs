pub mod cam;
use cam::*;

use bevy::prelude::*;

use shapeshifter_level_maker::{input::Action, ShapeshifterLevelMakerPlugin};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "pen".to_string(),
            width: 1200.,
            height: 800.,
            // present_mode: bevy::window::PresentMode::Immediate,
            ..Default::default()
        })
        //
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        // .add_startup_system(setup)
        .add_startup_system(camera_setup)
        .add_startup_system(setup)
        .add_plugin(CamPlugin)
        .add_plugin(ShapeshifterLevelMakerPlugin)
        .run();
}
//

fn setup(
    mut commands: Commands,
    mut action_event_writer: EventWriter<Action>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    action_event_writer.send(Action::QuickLoad {
        maybe_name: Some("002_simplicity_square".to_string()),
    });
    action_event_writer.send(Action::QuickLoadTarget {
        maybe_name: Some("002_simplicity_square".to_string()),
    });
    action_event_writer.send(Action::ToggleGrid); //

    // commands.spawn_bundle(MaterialMesh2dBundle {
    //     mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
    //     transform: Transform::default().with_scale(Vec3::splat(128.)),
    //     material: materials.add(ColorMaterial::from(Color::PURPLE)),
    //     ..default()
    // });
}
