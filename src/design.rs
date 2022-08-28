use crate::game::WholeGameCuts;
use crate::game_spawn::*;
use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use shapeshifter_level_maker::{input::Cursor, material::*, util::*};

use super::GameState;
use super::TEXT_COLOR;

// This plugin will contain the game. In this case, it's just be a screen that will
// display the current settings for 5 seconds before returning to the menu
pub struct DesignPlugin;

pub struct TargetScale {
    pub scale: f32,
    pub up: bool,
}

impl Plugin for DesignPlugin {
    fn build(&self, app: &mut App) {
        //

        //
        app.insert_resource(TargetScale {
            scale: 1.0,
            up: true,
        })
        .add_event::<SpawnDesignPoly>()
        .add_system_set(
            SystemSet::on_enter(GameState::Design)
                .with_system(design_setup)
                .with_system(spawn_shortcuts)
                .with_system(spawn_target_scale),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Design)
                .with_system(adjust_target_scale)
                .with_system(glow_design_poly)
                .with_system(spawn_design_poly)
                .with_system(browse_poly)
                .with_system(adjust_target_scale_view),
        );
        // .add_system_set(
        //     SystemSet::on_update(GameState::Design).with_run_criteria(FixedTimestep::step(0.2)), // .with_system(glow_design_poly),
        // );
    }
}

#[derive(Component)]
pub struct TargetScaleView;

fn adjust_target_scale(mut target_scale: ResMut<TargetScale>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Minus) {
        target_scale.scale *= 0.975;
        target_scale.up = false;
    }
    if keyboard_input.just_pressed(KeyCode::Equals) {
        target_scale.scale *= 1.025;
        target_scale.up = true;
    }
}

fn adjust_target_scale_view(
    target_scale: Res<TargetScale>,
    mut query: Query<&mut Text, With<TargetScaleView>>,
    mut target_query: Query<(&mut Target, &mut Mesh2dHandle)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if target_scale.is_changed() {
        //
        for mut text in query.iter_mut() {
            if let Some(mut section) = text.sections.get_mut(0) {
                let text = format!("target scale: {}", target_scale.scale);
                section.value = text.clone();
            }
        }

        for (mut target, mesh_handle) in target_query.iter_mut() {
            if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
                //  transform the path by rescaling it
                // target.path = target.path.
                // target.path = target_scale.scale;

                let scale = if target_scale.up { 1.025 } else { 0.975 };

                let scaling = lyon::geom::Scale::new(scale);
                let transformed_path = target.path.clone().transformed(&scaling);
                target.path = transformed_path;

                let (mesh2, _center_of_mass) = make_polygon_mesh(&target.path, false);
                *mesh = mesh2;
            }
        }
    }
}

fn design_setup(
    // mut spawn_level_event_writer: EventWriter<SpawnLevel>,
    // game_levels: ResMut<GameLevels>,
    // mut spawn_instruction_event_writer: EventWriter<SpawnInstruction>,
    poly_raw_map: Res<LoadedPolygonsRaw>,
    mut spawn_designpoly_event_writer: EventWriter<SpawnDesignPoly>,
    mut whole_game_cuts: ResMut<WholeGameCuts>,
) {
    // spawn_level_event_writer.send(game_levels.simplicity[5].clone());
    // send_tutorial_text(0, &mut spawn_instruction_event_writer);
    // info!("design_setup");
    whole_game_cuts.cuts = 111111111;
    let mut position = Vec2::new(-575.0, -300.);
    for (name, _polygon) in poly_raw_map.polygons.iter() {
        //
        //
        spawn_designpoly_event_writer.send(SpawnDesignPoly {
            polygon: name.clone(),
            position: position.clone(),
        });

        position.x += 100.0;
    }
}

pub fn spawn_target_scale(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    target_scale: ResMut<TargetScale>,
) {
    //
    // info!("spawn_shortcuts");

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let text = format!("target scale: {}", target_scale.scale);

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 22.0,
        color: TEXT_COLOR,
    };

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(5.0),
                    top: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
            color: Color::rgba(0., 0., 0., 0.).into(),
            ..default()
        })
        .with_children(|parent| {
            // Display the game name
            parent
                .spawn_bundle(
                    TextBundle::from_section(text, text_style).with_style(Style {
                        margin: UiRect::all(Val::Px(5.0)),
                        ..default()
                    }),
                )
                .insert(TargetScaleView);
        });
}

pub fn spawn_shortcuts(mut commands: Commands, asset_server: Res<AssetServer>) {
    //
    // info!("spawn_shortcuts");

    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let text = "
select:              s + left click
select multiple:     s + a + left click
delete selected:     delete 
toggle grid:         g
draw polygon:        shift + left click (then release shift to continue)
turn poly in target: select polygon + e + t
move point:          Q  + left mouse
add point:           left shift + right click
delete all:          a + delete

Note: there is no way to save a level on the browser version";

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 22.0,
        color: TEXT_COLOR,
    };

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(5.0),
                    top: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
            color: Color::rgba(0., 0., 0., 0.).into(),
            ..default()
        })
        .insert(Instruction)
        .with_children(|parent| {
            // Display the game name
            parent.spawn_bundle(
                TextBundle::from_section(text, text_style).with_style(Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                }),
            );
        });
}

pub struct SpawnDesignPoly {
    polygon: String,
    position: Vec2,
}

#[derive(Component)]
pub struct DesignPolygon;

// spawns a polygon from a MeshMeta
pub fn spawn_design_poly(
    mut commands: Commands,
    poly_raw_map: Res<LoadedPolygonsRaw>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut spawn_polykeep_event_reader: EventReader<SpawnDesignPoly>,
    // globals: Res<Globals>,
) {
    for SpawnDesignPoly { polygon, position } in spawn_polykeep_event_reader.iter() {
        // let mesh_meta: MeshMeta = save_format_mesh_meta.into();

        if let Some(save_format_mesh_meta) = poly_raw_map.polygons.get(polygon) {
            let points = shift_to_center_of_mass(&save_format_mesh_meta.points);

            let built_path = build_path_from_points(&points, 1.0);

            let mesh_meta = MeshMeta {
                id: 0,
                path: built_path,
                points: points.clone(),
                previous_transform: Transform::default(),
                is_intersecting: false,
                name: save_format_mesh_meta.name.clone(),
            };

            let (mesh, center_of_mass) = make_polygon_mesh(&mesh_meta.path, true);

            let mat_handle = fill_materials.add(FillMesh2dMaterial {
                color: Color::rgb(0.22, 0.4, 0.05).into(),
                show_com: 0.0,
                selected: 0.0,
                is_intersecting: 0.0,
            });

            let path_translation =
                lyon::geom::Translation::new(-center_of_mass.x, -center_of_mass.y);
            let transformed_path = mesh_meta.path.transformed(&path_translation);

            let z = 0.001;

            let mut transform = Transform::from_translation(position.extend(z));
            let scale = 0.2;
            transform.scale = Vec3::new(scale, scale, 1.0);

            let mesh_handle = meshes.add(mesh);

            //
            //
            //
            // spawn the polygon
            let _entity = commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(mesh_handle.clone()),
                    material: mat_handle,
                    transform,
                    ..default()
                })
                .insert(DesignPolygon)
                .insert(MeshMeta {
                    id: 0,
                    path: transformed_path.clone(),
                    points: mesh_meta.points, //TODO
                    previous_transform: transform,
                    is_intersecting: false,
                    name: mesh_meta.name,
                })
                .id();
        }
    }
}

pub fn browse_poly(
    mut query: Query<&mut Transform, With<DesignPolygon>>,
    // cursor: Res<Cursor>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    if let Some(mouse_wheel) = mouse_wheel_events.iter().next() {
        for mut transform in query.iter_mut() {
            // let transform = Transform::from_translation(-100.0,0.,0.);
            let delta = Vec3::new(5.0, 0.0, 0.0);
            if mouse_wheel.y > 0.5 {
                transform.translation += delta;
            }
            if mouse_wheel.y < -0.5 {
                transform.translation -= delta;
            }
        }
    }
}

pub fn glow_design_poly(
    // mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
    // keyboard_input: Res<Input<KeyCode>>,
    query: Query<(&Handle<FillMesh2dMaterial>, &Transform, &MeshMeta), With<DesignPolygon>>,
    mut materials: ResMut<Assets<FillMesh2dMaterial>>,

    mut spawn_polykeep_event_writer: EventWriter<SpawnPolyKeepPoly>,
    mut spawn_target_event_writer: EventWriter<SpawnTarget>,
) {
    let left_mouse_click = mouse_button_input.just_pressed(MouseButton::Left);
    let right_mouse_click = mouse_button_input.just_pressed(MouseButton::Right);

    // let ctrl = keyboard_input.pressed(KeyCode::LControl);
    // let shift = keyboard_input.pressed(KeyCode::LShift);

    for (material_handle, transform, mesh_meta) in query.iter() {
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

        let x = transform.translation.x;

        if cursor.position.x > x - 150. && cursor.position.x < x + 150. {
            let (is_inside_poly, _) = mesh_meta.hit_test(&cursor.clone().into(), &transform);

            let mut material = materials.get_mut(&material_handle).unwrap();
            material.show_com = 0.0;
            if is_inside_poly {
                material.show_com = 1.0;
            }

            if is_inside_poly && left_mouse_click {
                // info!("inside poly: {}", mesh_meta.name);
                spawn_polykeep_event_writer.send(SpawnPolyKeepPoly {
                    polygon: mesh_meta.name.clone(),
                    polygon_multiplier: 1.0,
                });
            }

            if is_inside_poly && right_mouse_click {
                // info!("inside poly: {}", mesh_meta.name);
                spawn_target_event_writer.send(SpawnTarget {
                    target: mesh_meta.name.clone(),
                    target_multiplier: 1.0,
                });
            }
        }
    }

    // add Rotating or Translating component to clicked entity
    //
}
