use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::input::Action;
use crate::material::FillMesh2dMaterial;
use crate::poly::Polygon;
use crate::target::*;
use crate::util::*;

// use std::fs::create_dir;

use std::io::Read;

use rand::{thread_rng, Rng};

use lyon::tessellation::math::Point;
use lyon::tessellation::path::{builder::NoAttributes, path::BuilderImpl, Path};

// pub struct QuickLoad;
pub struct Load(pub String);
pub struct LoadTarget(pub String);

#[derive(Default)]
pub struct LoadedPolyPath {
    pub maybe_path: Option<String>,
}

#[derive(Default)]
pub struct LoadedTargetPath {
    pub maybe_path: Option<String>,
}

pub struct LoadPlugin;

impl Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadedPolyPath::default())
            .insert_resource(LoadedTargetPath::default())
            .add_event::<Load>()
            .add_event::<LoadTarget>()
            .add_system(quick_load_all_mesh)
            .add_system(quick_load)
            .add_system(load_target)
            .add_system(load_mesh)
            .add_system(quick_load_target);
    }
}

// opens file dialog
pub fn load_target(
    mut action_event_reader: EventReader<Action>,
    mut loaded_target_event: EventWriter<LoadedTarget>,
    mut loaded_path: ResMut<LoadedTargetPath>,
) {
    if let Some(Action::LoadTarget) = action_event_reader.iter().next() {
        info!("load target");
        let mut save_prepath = std::env::current_dir().unwrap();
        save_prepath.push("assets/meshes/targets/");

        if let Some(chosen_path) = rfd::FileDialog::new()
            .set_directory(&save_prepath)
            .pick_file()
        {
            let file_name = chosen_path.file_name().unwrap().to_str().unwrap();
            loaded_path.maybe_path = Some(file_name.to_owned());
            //
            let mut file = std::fs::File::open(chosen_path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let loaded_mesh_params: SaveMeshMeta = serde_json::from_str(&contents).unwrap();
            loaded_target_event.send(LoadedTarget {
                save_mesh_meta: loaded_mesh_params,
            });
        }
    }
}

// always loads my_target
pub fn quick_load_target(
    mut loaded_target_event: EventWriter<LoadedTarget>,
    mut action_event_reader: EventReader<Action>,
    mut loaded_target_path: ResMut<LoadedTargetPath>,
    mut load_target_event_reader: EventReader<LoadTarget>,
) {
    //
    //
    let mut maybe_file_name: Option<String> = None;
    let mut do_load = false;

    match action_event_reader.iter().next() {
        Some(Action::QuickLoadTarget {
            maybe_name: Some(name),
        }) => {
            maybe_file_name = Some(name.to_owned());
            do_load = true;
        }
        Some(Action::QuickLoadTarget { maybe_name: None }) => {
            do_load = true;
        }
        _ => {}
    };

    for load in load_target_event_reader.iter() {
        // load_names.push(load.0.clone());
        maybe_file_name = Some(load.0.clone());
        do_load = true;
    }

    //
    //
    // if let Some(Action::QuickLoadTarget { maybe_name }) = action_event_reader.iter().next() {
    if do_load {
        let mut save_path = std::env::current_dir().unwrap();
        save_path.push("assets/meshes/");
        //
        //
        // load TARGET
        let file_name = if let Some(name) = maybe_file_name {
            name.to_owned()
        } else {
            "001_simplicity_square".to_owned()
        };

        loaded_target_path.maybe_path = Some(file_name.clone());
        save_path.push(file_name + ".pts");

        let mut file = std::fs::File::open(save_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let loaded_mesh_params: SaveMeshMeta = serde_json::from_str(&contents).unwrap();
        loaded_target_event.send(LoadedTarget {
            save_mesh_meta: loaded_mesh_params,
        });
    }
}

pub fn quick_load(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    // mut quickload_event_reader: EventReader<QuickLoad>,
    // mut load_event_reader: EventReader<Load>,
    mut action_event_reader: EventReader<Action>,
    globals: Res<Globals>,
    mut loaded_path: ResMut<LoadedPolyPath>,
    mut load_event_reader: EventReader<Load>,
) {
    let mut maybe_file_name_vec: Vec<String> = vec![];

    for action in action_event_reader.iter() {
        match action {
            Action::QuickLoad {
                maybe_name: Some(name),
            } => {
                maybe_file_name_vec.push(name.to_owned());
            }
            Action::QuickLoad { maybe_name: None } => {}
            _ => {}
        };
    }

    for load in load_event_reader.iter() {
        // load_names.push(load.0.clone());
        maybe_file_name_vec.push(load.0.clone());
    }

    for maybe_file_name in maybe_file_name_vec {
        let mut save_prepath = std::env::current_dir().unwrap();
        save_prepath.push("assets/meshes/");
        //
        //
        //
        // load POLYGON

        let file_name = maybe_file_name.to_owned();

        // let file_name = if let Some(name) = maybe_file_name {
        //     name.to_owned()
        // } else {
        //     "001_simplicity_square".to_owned()
        // };

        loaded_path.maybe_path = Some(file_name.clone());
        save_prepath.push(file_name + ".pts");

        // save_prepath.push(.to_owned());

        //
        //
        //
        //
        // info!("loading {}", save_prepath.to_str().unwrap());
        let mut file = std::fs::File::open(save_prepath).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let loaded_mesh_params: SaveMeshMeta = serde_json::from_str(&contents).unwrap();

        let points = shift_to_center_of_mass(&loaded_mesh_params.points);

        let mat_handle = fill_materials.add(FillMesh2dMaterial {
            color: globals.polygon_color.into(),
            show_com: 0.0,
            selected: 0.0,
            is_intersecting: 0.0,
        });

        let mut path: NoAttributes<BuilderImpl> = Path::builder();

        for (idx, pos) in points.iter().enumerate() {
            //
            if idx == 0 {
                path.begin(Point::new(pos.x, pos.y));
            } else {
                path.line_to(Point::new(pos.x, pos.y));
            };
        }

        path.close();

        let built_path: Path = path.clone().build();

        let (mesh, center_of_mass) = make_polygon_mesh(&built_path, true);

        // // Sets path center of mass to the origin
        let path_translation = lyon::geom::Translation::new(-center_of_mass.x, -center_of_mass.y);
        let transformed_path = built_path.transformed(&path_translation);

        let mut rng = thread_rng();
        let id = rng.gen::<u64>();
        let z = rng.gen::<f32>();

        let transform = Transform::from_translation(Vec2::new(-300.0, 0.0).extend(z));
        // transform.rotate_axis(Vec3::Z, loaded_mesh_params.rotation);
        info!("rotation: {:?}", loaded_mesh_params.rotation);

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
            .insert(Polygon)
            .insert(MeshMeta {
                id,
                path: transformed_path.clone(),
                points: loaded_mesh_params.points, //TODO
                previous_transform: transform,
                is_intersecting: false,
            })
            .id();

        let ghost_mat_handle = fill_materials.add(FillMesh2dMaterial {
            color: globals.ghost_color.into(),
            show_com: 0.0,
            selected: 0.0,
            is_intersecting: 0.0,
        });

        let mut ghost_transform = transform;
        ghost_transform.translation.z = -10.0;

        let _ghost_entity = commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh_handle.clone()),
                material: ghost_mat_handle,
                transform: ghost_transform,
                ..default()
            })
            .insert(Ghost)
            .id();
    }
}

// either loads the "assets/meshes/my_mesh0" folder with the QuickLoad event
// or loads the "assets/meshes/<name>" folder with the Load event.
//
// Groups of polygons are not tagged as a group though
//
//

pub fn quick_load_all_mesh(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    // mut quickload_event_reader: EventReader<QuickLoad>,
    // mut load_event_reader: EventReader<Load>,
    mut action_event_reader: EventReader<Action>,
    globals: Res<Globals>,
    // mut loaded_path: ResMut<LoadedPolyPath>,
) {
    let mut load_names = Vec::new();
    let mut save_prepath = std::env::current_dir().unwrap();
    save_prepath.push("assets/meshes/".to_owned());

    let prefix = "my_mesh".to_string();
    let extension = "points".to_string();

    if action_event_reader
        .iter()
        .any(|x| x == &Action::QuickLoadAll)
    {
        //
        //
        // to change the folder that QuickLoad will load, change the string here
        load_names.push("my_mesh0".to_string());
    }

    // for load in load_event_reader.iter() {
    //     load_names.push(load.0.clone());
    // }

    for load_name in load_names.iter() {
        //
        //
        // initialize path
        let mut save_path = save_prepath.clone();
        save_path.push(load_name);

        //
        //
        let all_files = std::fs::read_dir(&save_path).unwrap();
        let single_mesh = all_files.count() == 2; // obj and point
        info!("Is single_mesh?: {:?}", single_mesh);

        //
        //
        // insert the first name
        save_path.push(prefix.clone() + "0" + "." + &extension);

        //
        //
        //
        // load every my_mesh*.obj file and my_mesh*.point file
        let mut k = 0;
        loop {
            //
            //
            // name of file with incrementing k
            let mut name = prefix.to_string();
            name.push_str(&(k.to_string()));
            k = k + 1;

            save_path = save_path.with_file_name(&name);
            save_path = save_path.with_extension(&extension);

            // Only condition is that the file exists. If not, loading is terminated
            if !save_path.is_file() {
                return;
            }

            //
            //
            //
            // load the mesh with an .obj loader (the bevy_obj crate)
            //
            // let mesh_handle_loaded: Handle<Mesh> = asset_server.load(save_path.to_str().unwrap());

            //
            //
            //
            // get mesh meta info using the .points extension
            //
            let saved_mesh_data = save_path.with_extension("points");
            let mut file = std::fs::File::open(saved_mesh_data).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let loaded_mesh_params: SaveMeshMeta = serde_json::from_str(&contents).unwrap();

            let poly_color = if single_mesh {
                globals.polygon_color
            } else {
                globals.cut_polygon
            };

            let mat_handle = fill_materials.add(FillMesh2dMaterial {
                color: poly_color.into(),
                show_com: 0.0,
                selected: 0.0,
                is_intersecting: 0.0,
            });

            //
            //
            //
            // build the polygon
            //
            let mut path: NoAttributes<BuilderImpl> = Path::builder();

            for (idx, pos) in loaded_mesh_params.points.iter().enumerate() {
                //
                if idx == 0 {
                    path.begin(Point::new(pos.x, pos.y));
                } else {
                    path.line_to(Point::new(pos.x, pos.y));
                };
            }

            path.close();

            let built_path: Path = path.clone().build();

            let (mesh, center_of_mass) = make_polygon_mesh(&built_path, true);

            // // Sets path center of mass to the origin
            let path_translation =
                lyon::geom::Translation::new(-center_of_mass.x, -center_of_mass.y);
            let transformed_path = built_path.transformed(&path_translation);

            let mut rng = thread_rng();
            let id = rng.gen::<u64>();
            let z = rng.gen::<f32>();

            let mut transform =
                Transform::from_translation(loaded_mesh_params.translation.extend(z));
            transform.rotate_axis(Vec3::Z, loaded_mesh_params.rotation);

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
                .insert(Polygon)
                .insert(MeshMeta {
                    id,
                    path: transformed_path.clone(),
                    points: loaded_mesh_params.points, //TODO
                    previous_transform: transform,
                    is_intersecting: false,
                })
                .id();

            let ghost_mat_handle = fill_materials.add(FillMesh2dMaterial {
                color: globals.ghost_color.into(),
                show_com: 0.0,
                selected: 0.0,
                is_intersecting: 0.0,
            });

            let mut ghost_transform = transform;
            ghost_transform.translation.z = -10.0;

            let _ghost_entity = commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(mesh_handle.clone()),
                    material: ghost_mat_handle,
                    transform: ghost_transform,
                    ..default()
                })
                .insert(Ghost)
                .id();
        }
    }
}

// simple dialog load
pub fn load_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut action_event_reader: EventReader<Action>,
    globals: Res<Globals>,
) {
    if action_event_reader.iter().any(|x| x == &Action::LoadDialog) {
        let mut save_prepath = std::env::current_dir().unwrap();
        save_prepath.push("assets/meshes/".to_owned());

        // if let Some(Action::LoadDialog) = action_event_reader.iter().next() {

        info!("load dialog AHHHH");
        // let maybe_dialog_path = open_file_dialog("save_name", None, "point");
        if let Some(file_path) = rfd::FileDialog::new()
            // .set_file_name(&save_prepath)
            .set_directory(&save_prepath)
            .pick_file()
        {
            // get mesh meta info using the .points extension
            //

            let mut file = std::fs::File::open(file_path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let loaded_mesh_params: SaveMeshMeta = serde_json::from_str(&contents).unwrap();

            let poly_color = globals.polygon_color;

            let mat_handle = fill_materials.add(FillMesh2dMaterial {
                color: poly_color.into(),
                show_com: 0.0,
                selected: 0.0,
                is_intersecting: 0.0,
            });

            //
            //
            //
            // build the polygon
            //
            let mut path: NoAttributes<BuilderImpl> = Path::builder();

            for (idx, pos) in loaded_mesh_params.points.iter().enumerate() {
                //
                if idx == 0 {
                    path.begin(Point::new(pos.x, pos.y));
                } else {
                    path.line_to(Point::new(pos.x, pos.y));
                };
            }

            path.close();

            let built_path: Path = path.clone().build();

            let (mesh, center_of_mass) = make_polygon_mesh(&built_path, true);

            // // Sets path center of mass to the origin
            let path_translation =
                lyon::geom::Translation::new(-center_of_mass.x, -center_of_mass.y);
            let transformed_path = built_path.transformed(&path_translation);

            let mut rng = thread_rng();
            let id = rng.gen::<u64>();
            let z = rng.gen::<f32>();

            let mut transform =
                Transform::from_translation(loaded_mesh_params.translation.extend(z));
            transform.rotate_axis(Vec3::Z, loaded_mesh_params.rotation);

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
                .insert(Polygon)
                .insert(MeshMeta {
                    id,
                    path: transformed_path.clone(),
                    points: loaded_mesh_params.points, //TODO
                    previous_transform: transform,
                    is_intersecting: false,
                })
                .id();

            let ghost_mat_handle = fill_materials.add(FillMesh2dMaterial {
                color: globals.ghost_color.into(),
                show_com: 0.0,
                selected: 0.0,
                is_intersecting: 0.0,
            });

            let mut ghost_transform = transform;
            ghost_transform.translation.z = -10.0;

            let _ghost_entity = commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(mesh_handle.clone()),
                    material: ghost_mat_handle,
                    transform: ghost_transform,
                    ..default()
                })
                .insert(Ghost)
                .id();
        }
    }
}
