use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::input::*;
use crate::material::FillMesh2dMaterial;
use crate::poly::Polygon;
use crate::util::*;

use serde::Deserialize;
use serde::Serialize;

use std::collections::HashMap;
use std::collections::HashSet;

use std::fs::File;
use std::io::Read;
use std::io::Write;

use rand::{thread_rng, Rng};

use std::path::PathBuf;

use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::{builder::NoAttributes, path::BuilderImpl, Path};

pub struct QuickLoad;
pub struct Load(pub String);

// only loads groups
pub fn quick_load_mesh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut quickload_event_reader: EventReader<QuickLoad>,
    mut load_event_reader: EventReader<Load>,
    globals: Res<Globals>,
) {
    let mut load_names = Vec::new();

    for _ in quickload_event_reader.iter() {
        load_names.push("my_mesh7".to_string());
    }

    for load in load_event_reader.iter() {
        load_names.push(load.0.clone());
    }

    for name in load_names {
        info!("quick loading mesh");

        let filename = "assets/meshes/".to_owned() + &name + ".obj";

        let mut save_path = std::env::current_dir().unwrap();
        save_path.push(filename);

        let mesh_handle: Handle<Mesh> = asset_server.load(save_path.to_str().unwrap());

        // get mesh info using the .points extension
        let saved_mesh_data = save_path.with_extension("points");
        let mut file = std::fs::File::open(saved_mesh_data).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let loaded_mesh_params: SaveMeshMeta = serde_json::from_str(&contents).unwrap();

        let mat_handle = fill_materials.add(FillMesh2dMaterial {
            color: globals.polygon_color.into(),
            show_com: 0.0,
        });

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

        let transform = Transform::from_translation(Vec3::new(0.0, 0.0, rng.gen::<f32>()));

        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(mesh_handle),
                material: mat_handle,
                transform,
                ..default()
            })
            .insert(Polygon)
            .insert(MeshMeta {
                id,
                path: built_path.clone(),
                points: loaded_mesh_params.points, //TODO
            });

        // let mat_handle = fill_materials.add(FillMesh2dMaterial {
        //     color: globals.polygon_color.into(),
        //     show_com: 0.0, // show center of mass
        // });

        // let id = rng.gen::<u64>();
        // let entity = commands
        //     .spawn_bundle(MaterialMesh2dBundle {
        //         mesh: Mesh2dHandle(meshes.add(mesh)),
        //         material: mat_handle,
        //         transform: fill_transform,
        //         ..default()
        //     })
        //     .insert(Polygon)
        //     .insert(MeshMeta {
        //         id,
        //         path: path.clone(),
        //         points: poly.all_points.clone(),
        //     })
        //     .id();
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
    // polygon_query: Query<&MakingPolygon>,
    mesh_query: Query<(&Mesh2dHandle, &MeshMeta), With<Polygon>>,
    // fill_mats: Res<Assets<FillMesh2dMaterial>>,
    meshes: Res<Assets<Mesh>>,
    // globals: ResMut<Globals>,
    mut action_event_reader: EventReader<SaveMeshEvent>,
) {
    for _ in action_event_reader.iter() {
        //
        for (mesh_handle, mesh_meta) in mesh_query.iter() {
            let (free_save_path_obj, k) =
                get_free_save_name("my_mesh".to_string(), "obj".to_string());
            println!("free_save_path : {:?}", free_save_path_obj);

            let free_save_path_points = free_save_path_obj.with_extension("points");

            save_mesh(&mesh_handle.0, &meshes, free_save_path_obj);

            let save_mesh_meta: SaveMeshMeta = mesh_meta.into();
            let serialized = serde_json::to_string_pretty(&save_mesh_meta).unwrap();
            let mut output = File::create(free_save_path_points).unwrap();
            let _group_write_result = output.write(serialized.as_bytes());
        }
    }
}

pub fn get_free_save_name(prefix: String, extension: String) -> (std::path::PathBuf, u64) {
    let mut save_path = std::env::current_dir().unwrap();
    save_path.push("assets/meshes/my_mesh0.obj");
    let mut k = 0;
    loop {
        let mut name = prefix.to_string();
        name.push_str(&(k.to_string()));
        save_path = save_path.with_file_name(&name);
        save_path = save_path.with_extension(&extension);

        if !save_path.exists() {
            break;
        }

        k += 1;
    }
    (save_path, k)
}

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
