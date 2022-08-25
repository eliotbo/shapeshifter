use bevy::prelude::*;

use crate::input::Action;
use crate::poly::Polygon;
use crate::util::*;

// use std::fs::create_dir;
use std::fs::File;
use std::io::Write;

use std::path::PathBuf;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(quick_save).add_system(save_one_selected);
    }
}

////////////////// ////////////////// ////////////////// ////////////////// //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////
/////////////      comment everything below before building for wasm        //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////
////////////////// ////////////////// ////////////////// ////////////////// //////////////////

pub fn save_one_selected(
    mesh_query: Query<(&Transform, &MeshMeta), (With<Polygon>, With<Selected>)>,
    mesh_query_no_select: Query<(&Transform, &MeshMeta), (With<Polygon>, Without<Selected>)>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::SaveOneDialog) = action_event_reader.iter().next() {
        // //
        // //
        // // opens up file dialog
        let mut save_prepath = std::env::current_dir().unwrap();
        save_prepath.push("assets/meshes/");

        if let Some(chosen_path) = rfd::FileDialog::new()
            .set_directory(&save_prepath)
            .save_file()
        {
            //
            //
            if let Some((transform, mesh_meta)) = mesh_query.iter().next() {
                let (axis, transform_rotation_angle) = transform.rotation.to_axis_angle();
                let angle = axis.z * transform_rotation_angle;

                let save_mesh_meta: SaveMeshMeta = SaveMeshMeta {
                    points: mesh_meta.points.clone(),
                    translation: transform.translation.truncate(),
                    rotation: angle,
                };
                let serialized = serde_json::to_string_pretty(&save_mesh_meta).unwrap();
                let mut output = File::create(chosen_path).unwrap();
                let _group_write_result = output.write(serialized.as_bytes());
            } else {
                if let Some((transform, mesh_meta)) = mesh_query_no_select.iter().next() {
                    let (axis, transform_rotation_angle) = transform.rotation.to_axis_angle();
                    let angle = axis.z * transform_rotation_angle;

                    let save_mesh_meta: SaveMeshMeta = SaveMeshMeta {
                        points: mesh_meta.points.clone(),
                        translation: transform.translation.truncate(),
                        rotation: angle,
                    };
                    let serialized = serde_json::to_string_pretty(&save_mesh_meta).unwrap();
                    let mut output = File::create(chosen_path).unwrap();
                    let _group_write_result = output.write(serialized.as_bytes());
                }
            }
        }
    }
}

pub fn quick_save(
    mesh_query: Query<(&Transform, &MeshMeta), With<Polygon>>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::QuickSave) = action_event_reader.iter().next() {
        //
        //
        //
        // get the first free directory with name "my_mesh#" in assets/meshes
        let folder_name = "my_mesh".to_string();

        let (mut save_path, _i) = create_save_dir(folder_name);
        save_path.push("dummy.point");
        //
        //
        println!("saving to {:?}", save_path);
        //
        //
        // Save all individual meshes and meta data for the meshes in the same folder
        for (transform, mesh_meta) in mesh_query.iter() {
            //
            //
            // get the first free file name with name "my_mesh#_#" in assets/meshes/"my_mesh#/"
            let (free_save_path_obj, _k) = get_free_save_name(
                save_path.clone(),
                "my_mesh".to_string(),
                "point".to_string(),
            );
            //
            //
            // println!("free_save_path : {:?}", free_save_path_obj);

            let free_save_path_points = free_save_path_obj.with_extension("points");

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
