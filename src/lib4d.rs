#[crate_id = "nrays4d#0.1"];
#[crate_type = "lib"];
#[warn(non_camel_case_types)];
#[feature(managed_boxes)];

extern mod extra;
extern mod nalgebra;
extern mod ncollide = "ncollide4df64";
extern mod png;

pub mod scene_node;
pub mod scene;
pub mod image;
pub mod light;
pub mod material;
pub mod ray_with_energy;

pub mod phong_material;
pub mod reflective_material;
pub mod texture2d;

// NOTE: Those cannot be used on 4d ray cast.

// pub mod normal_material;
// pub mod uv_material;

pub mod obj;
pub mod mtl;
pub mod mesh;
