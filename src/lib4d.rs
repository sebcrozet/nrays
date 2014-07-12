#![warn(non_camel_case_types)]
#![feature(managed_boxes)]
#![feature(phase)]

extern crate rustrt;
extern crate native;
extern crate nalgebra;
extern crate ncollide = "ncollide4df64";
extern crate png;
extern crate stb_image;

#[phase(plugin)] extern crate dim4;

pub mod scene_node;
pub mod scene;
pub mod image;
pub mod light;
pub mod material;
pub mod ray_with_energy;

pub mod phong_material;
pub mod texture2d;

// NOTE: Those cannot be used on 4d ray cast.

// pub mod normal_material;
// pub mod uv_material;

pub mod obj;
pub mod mtl;
pub mod mesh;
