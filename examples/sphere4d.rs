#[link(name     = "sphere4d"
       , vers   = "0.0"
       , author = "Sébastien Crozet"
       , uuid   = "cf8cfe5d-18ca-40cb-b596-d8790090a56d")];
#[crate_type = "bin"];
#[warn(non_camel_case_types)]

extern mod nalgebra;
extern mod ncollide;
extern mod nrays;

// use nalgebra::na::{Vec4, Vec3, Iso4};
// use nalgebra::na;
// use ncollide::ray::Ray;
// use nrays::scene::Scene;

fn main() {
    // let scene: Scene<f64, Vec4<f64>, Vec3<uint>, Iso4<f64>> = Scene::new(~[], ~[]);

    // let eye        = na::vec4(0.0f64, 0.0, 0.0, 0.0);
    // let at         = na::vec4(0.0f64, 0.0, 0.0, 1.0);
    // let resolution = na::vec3(10u, 10, 10);

    // scene.render(&resolution, |pt| { println(pt.to_str()); Ray::new(eye, at) });
}
