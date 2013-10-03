#[link(name     = "sphere3d"
       , vers   = "0.0"
       , author = "Sébastien Crozet"
       , uuid   = "cf8cfe5d-18ca-40cb-b596-d8790090a56d")];
#[crate_type = "bin"];
#[warn(non_camel_case_types)]

extern mod nalgebra;
extern mod ncollide;
extern mod nrays;

use std::num::One;
use std::io;
use nalgebra::vec::*;
use nalgebra::mat::*;
use nalgebra::types::*;
use ncollide::ray::Ray;
use ncollide::geom::{Geom, Ball};
use nrays::scene_node::{Material, SceneNode};
use nrays::scene::Scene;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    let eye        = Vec3::new(5.0f64, 5.0, 0.0);
    let at         = Vec3::new(0.0f64, 0.0, 1.0);
    let resolution = Vec2::new(500u, 500);
    let mut nodes  = ~[];

    {
        let material = Material::new(Vec4::new(1.0f64, 0.0, 1.0, 1.0));

        let mut transform: Iso3f64 = One::one();
        transform.translate_by(&Vec3::new(250.0f64, 250.0, 10.0));

        let geometry: Geom<f64, Vec3f64, Iso3f64> = Geom::new_ball(Ball::new(50.0f64));

        nodes.push(@SceneNode::new(material, transform, geometry));
    }

    let scene  = Scene::new(nodes);
    let pixels = scene.render(&resolution, |pt| {
        Ray::new(Vec3::new(pt.x as f64, pt.y as f64, 0.0), at)
    });

    let file = io::buffered_file_writer(&PosixPath("out.ppm")).expect("Cannot open the output file.");
    pixels.to_ppm(file);
}
