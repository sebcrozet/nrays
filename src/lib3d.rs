#[crate_id = "nrays3d#0.1"];
#[crate_type = "lib"];
#[warn(non_camel_case_types)];
#[feature(managed_boxes)];

extern mod native;
extern mod extra;
extern mod nalgebra;
extern mod ncollide = "ncollide3df64";
extern mod png;
extern mod stb_image;

pub mod scene_node;
pub mod scene;
pub mod image;
pub mod light;
pub mod material;
pub mod ray_with_energy;
pub mod intersection;

pub mod phong_material;
pub mod texture2d;

pub mod normal_material;
pub mod uv_material;

pub mod obj;
pub mod mtl;
pub mod mesh;
