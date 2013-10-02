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
use nalgebra::vec::Vec3;
use nalgebra::types::Iso3f64;
use nrays::scene::Scene;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    let scene = Scene::new(~[]);

    let eye        = Vec3::new(0.0f64, 0.0, 0.0);
    let at         = Vec3::new(0.0f64, 0.0, 1.0);
    let extents    = Vec3::new(100.0f64, 100.0, 100.0);
    let resolution = Vec3::new(10u, 10, 10);
    let projection: Iso3f64 = One::one();

    scene.render(&eye, &at, &extents, &resolution, &projection);
}
