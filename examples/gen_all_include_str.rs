use std::path::*;

pub fn main() {
    let mut save_prepath = std::env::current_dir().unwrap();
    save_prepath.push("assets/meshes/");
    println!("save_prepath: {:?}", save_prepath);

    for (idx, entry) in std::fs::read_dir(save_prepath).unwrap().enumerate() {
        // if idx < 2 {
        let entry = entry.unwrap();
        let path = entry.path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".pts") {
            // let save_mesh_meta = read_save_mesh_meta(path_str);
            let name_of_file = path_str.split("/").last().unwrap();
            let name_of_file = name_of_file.split(".").next().unwrap();
            println!("{:?}", name_of_file);
            // println!("polygon_map.insert(\"{}\".to_string(), serde_json::from_str(&include_str!(\"polygons/{}.pts\")).unwrap()); ", name_of_file, name_of_file);
        }
        // }
    }
}
