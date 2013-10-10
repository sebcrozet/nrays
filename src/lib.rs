#[link(name     = "nrays"
       , vers   = "0.0"
       , author = "Sébastien Crozet"
       , uuid   = "3d5ca519-5558-4ab7-8880-e7a4b62ad2c4")];
#[crate_type = "lib"];
#[warn(non_camel_case_types)]

extern mod extra;
extern mod nalgebra;
extern mod ncollide;

pub mod scene_node;
pub mod scene;
pub mod image;
pub mod light;
