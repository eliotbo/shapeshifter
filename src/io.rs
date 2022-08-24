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
use std::fs::File;
use std::io::Read;
use std::io::Write;

use rand::{thread_rng, Rng};

use std::path::PathBuf;

use lyon::tessellation::math::Point;
use lyon::tessellation::path::{builder::NoAttributes, path::BuilderImpl, Path};

pub struct QuickLoad;
pub struct Load(pub String);

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(quick_load_mesh)
            .add_system(quick_save)
            .add_system(quick_load_target);
    }
}

pub fn quick_load_target(
    // mut commands: Commands,
    // asset_server: Res<AssetServer>,
    // mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    // mut quickload_event_reader: EventReader<QuickLoad>,
    // mut load_event_reader: EventReader<Load>,
    // globals: Res<Globals>,
    // mut poly_order: ResMut<PolyOrder>,
    mut loaded_target_event: EventWriter<LoadedTarget>,
    mut action_event_reader: EventReader<Action>,
) {
    //
    //
    if let Some(Action::QuickLoadTarget) = action_event_reader.iter().next() {
        let mut save_path = std::env::current_dir().unwrap();
        save_path.push("assets/meshes/my_target.points".to_owned());

        let mut file = std::fs::File::open(save_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let loaded_mesh_params: SaveMeshMeta = serde_json::from_str(&contents).unwrap();
        loaded_target_event.send(LoadedTarget {
            save_mesh_meta: loaded_mesh_params,
        });
    }
}

// either loads the "assets/meshes/my_mesh0" folder with the QuickLoad event
// or loads the "assets/meshes/<name>" folder with the Load event.
//
// Groups of polygons are not tagged as a group.
//
//
pub fn quick_load_mesh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut quickload_event_reader: EventReader<QuickLoad>,
    mut load_event_reader: EventReader<Load>,
    globals: Res<Globals>,
) {
    let mut load_names = Vec::new();
    let mut save_prepath = std::env::current_dir().unwrap();
    save_prepath.push("assets/meshes/".to_owned());

    let prefix = "my_mesh".to_string();
    let extension = "obj".to_string();

    for _ in quickload_event_reader.iter() {
        //
        //
        // to change the folder that QuickLoad will load, change the string here
        load_names.push("my_mesh0".to_string());
    }

    for load in load_event_reader.iter() {
        load_names.push(load.0.clone());
    }

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
            let mesh_handle: Handle<Mesh> = asset_server.load(save_path.to_str().unwrap());

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

            let mut rng = thread_rng();
            let id = rng.gen::<u64>();
            let z = rng.gen::<f32>();

            let mut transform =
                Transform::from_translation(loaded_mesh_params.translation.extend(z));
            transform.rotate_axis(Vec3::Z, loaded_mesh_params.rotation);

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
                    path: built_path.clone(),
                    points: loaded_mesh_params.points, //TODO
                    previous_transform: transform,
                })
                .id();

            let ghost_mat_handle = fill_materials.add(FillMesh2dMaterial {
                color: globals.ghost_color.into(),
                show_com: 0.0,
                selected: 0.0,
            });

            let mut ghost_transform = transform;
            ghost_transform.translation.z = -10.0;

            let _ghost_entity = commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(mesh_handle),
                    material: ghost_mat_handle,
                    transform: ghost_transform,
                    ..default()
                })
                .insert(Ghost)
                .id();
        }
    }
}

// pub fn open_file_dialog(save_name: &str, folder: &str, extension: &str) -> Option<PathBuf> {
//     let mut k = 0;

//     let mut default_path = std::env::current_dir().unwrap();
//     default_path.push("saved");
//     default_path.push(folder.to_string());
//     let mut default_name: String;

//     loop {
//         default_name = save_name.to_string();
//         default_name.push_str(&(k.to_string()));
//         default_name.push_str(extension);

//         default_path.push(&default_name);

//         if !default_path.exists() {
//             break;
//         }
//         default_path.pop();

//         k += 1;
//     }

//     let res = rfd::FileDialog::new()
//         .set_file_name(&default_name)
//         .set_directory(&default_path)
//         .save_file();
//     println!("The user chose: {:#?}", &res);

//     return res;
// }

pub struct SaveMeshEvent;

pub fn quick_save(
    mesh_query: Query<(&Mesh2dHandle, &Transform, &MeshMeta), With<Polygon>>,
    meshes: Res<Assets<Mesh>>,
    mut action_event_reader: EventReader<SaveMeshEvent>,
) {
    for _ in action_event_reader.iter() {
        //
        //
        //
        // get the first free directory with name "my_mesh#" in assets/meshes
        let (mut save_path, _i) = create_save_dir("my_mesh".to_string());
        save_path.push("dummy.obj");
        //
        //
        println!("saving to {:?}", save_path);
        //
        //
        // Save all individual meshes and meta data for the meshes in the same folder
        for (mesh_handle, transform, mesh_meta) in mesh_query.iter() {
            //
            //
            // get the first free file name with name "my_mesh#_#" in assets/meshes/"my_mesh#/"
            let (free_save_path_obj, _k) =
                get_free_save_name(save_path.clone(), "my_mesh".to_string(), "obj".to_string());
            //
            //
            // println!("free_save_path : {:?}", free_save_path_obj);

            let free_save_path_points = free_save_path_obj.with_extension("points");

            save_mesh(&mesh_handle.0, &meshes, free_save_path_obj);

            let (axis, transform_rotation_angle) = transform.rotation.to_axis_angle();
            let angle = axis.z * transform_rotation_angle;

            let save_mesh_meta: SaveMeshMeta = SaveMeshMeta {
                points: mesh_meta.points.clone(),
                translation: transform.translation.truncate(),
                rotation: angle,
            };
            let serialized = serde_json::to_string_pretty(&save_mesh_meta).unwrap();
            let mut output = File::create(free_save_path_points).unwrap();
            let _group_write_result = output.write(serialized.as_bytes());
        }
    }
}

pub fn create_save_dir(folder_prefix_name: String) -> (std::path::PathBuf, u64) {
    let mut save_path = std::env::current_dir().unwrap();
    save_path.push("assets/meshes/".to_string());
    let mut k = 0;
    loop {
        let name = folder_prefix_name.to_string() + &k.to_string() + "/";
        save_path.push(&name);

        if !save_path.exists() {
            break;
        } else {
            save_path = save_path.parent().unwrap().to_path_buf();
        }

        k += 1;
    }
    // create the directory
    std::fs::create_dir(&save_path).unwrap();

    (save_path, k)
}

// files don't get overwritten, which is why the name # is incremented
pub fn get_free_save_name(
    mut path_buf: PathBuf,
    prefix: String,
    extension: String,
) -> (std::path::PathBuf, u64) {
    // let mut save_path = std::env::current_dir().unwrap();
    // save_path.push("assets/meshes/my_mesh0.obj");
    let mut k = 0;
    loop {
        let mut name = prefix.to_string();
        name.push_str(&(k.to_string()));
        path_buf = path_buf.with_file_name(&name);
        path_buf = path_buf.with_extension(&extension);

        if !path_buf.exists() {
            break;
        }

        k += 1;
    }
    (path_buf, k)
}

// pub fn get_free_save_name(prefix: String, extension: String) -> (std::path::PathBuf, u64) {
//     let mut save_path = std::env::current_dir().unwrap();
//     save_path.push("assets/meshes/my_mesh0.obj");
//     let mut k = 0;
//     loop {
//         let mut name = prefix.to_string();
//         name.push_str(&(k.to_string()));
//         save_path = save_path.with_file_name(&name);
//         save_path = save_path.with_extension(&extension);

//         if !save_path.exists() {
//             break;
//         }

//         k += 1;
//     }
//     (save_path, k)
// }

pub fn save_mesh(mesh_handle: &Handle<Mesh>, meshes: &Res<Assets<Mesh>>, path: PathBuf) {
    let mesh = meshes.get(mesh_handle).unwrap();
    let vertex_attributes = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
    let indices_u32 = mesh.indices().unwrap();

    match (vertex_attributes, indices_u32) {
        (
            bevy::render::mesh::VertexAttributeValues::Float32x3(vertices),
            bevy::render::mesh::Indices::U32(indices),
        ) => {
            let obj_vertices = vertices
                .clone()
                .iter()
                .map(|arr| obj_exporter::Vertex {
                    x: arr[0] as f64,
                    y: arr[1] as f64,
                    z: arr[2] as f64,
                })
                .collect::<Vec<obj_exporter::Vertex>>();

            // let mut obj_inds_vecs: Vec<Vec<u32>> =
            // indices.chunks(3).map(|x| x.to_vec()).collect();
            let obj_inds_vecs: Vec<(usize, usize, usize)> = indices
                .chunks_exact(3)
                .map(|z| {
                    let mut x = z.iter();
                    return (
                        *x.next().unwrap() as usize,
                        *x.next().unwrap() as usize,
                        *x.next().unwrap() as usize,
                    );
                })
                .collect();

            let normals = vec![obj_exporter::Vertex {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            }];
            // let olen = obj_vertices.len();
            let olen = 1;

            // let tvert =

            let set = obj_exporter::ObjSet {
                material_library: None,
                objects: vec![obj_exporter::Object {
                    name: "My_mesh".to_owned(),
                    vertices: obj_vertices,
                    tex_vertices: vec![(0.0, 0.0); olen]
                        .into_iter()
                        .map(|(u, v)| obj_exporter::TVertex { u, v, w: 0.0 })
                        .collect(),
                    normals,
                    geometry: vec![obj_exporter::Geometry {
                        material_name: None,
                        shapes: obj_inds_vecs
                            .into_iter()
                            .map(|(x, y, z)| obj_exporter::Shape {
                                primitive: obj_exporter::Primitive::Triangle(
                                    (x, None, None),
                                    (y, None, None),
                                    (z, None, None),
                                ),
                                groups: vec![],
                                smoothing_groups: vec![],
                            })
                            .collect(),
                    }],
                }],
            };

            obj_exporter::export_to_file(&set, path).unwrap();
        }
        _ => {}
    }
}
