#[link(name     = "sphere4d"
       , vers   = "0.0"
       , author = "Sébastien Crozet"
       , uuid   = "cf8cfe5d-18ca-40cb-b596-d8790090a56d")];
#[crate_type = "bin"];
#[warn(non_camel_case_types)]

extern mod nalgebra;
extern mod ncollide;
extern mod nrays;

use std::io::buffered::BufferedWriter;
use std::io::fs::File;
use std::rand::Rng;
use std::rand;
use nalgebra::na::{Iso3, Iso4, Vec2, Vec3, Vec4, Mat4, Inv};
use nalgebra::na;
use ncollide::ray::Ray;
use ncollide::geom::{Ball, Box};
use nrays::scene_node::SceneNode;
use nrays::material::Material4d;
use nrays::normal_material::NormalMaterial;
use nrays::phong_material::PhongMaterial;
use nrays::reflective_material::ReflectiveMaterial;
use nrays::scene::Scene;
use nrays::light::Light;

fn main() {
    let eye        = Vec4::new(0.0f64, 0.0, 0.0, 0.0);
    let at         = Vec4::new(0.0f64, 0.0, 0.0, 1.0);
    let resolution = Vec3::new(100.0f64, 100.0, 100.0);

    let mut lights = ~[];

    {
        lights.push(Light::new(Vec4::new(10.0f64, -10.0, 10.0, 10.0),
                               Vec3::new(1.0, 1.0, 1.0)));
    }

    let blue = @PhongMaterial::new(
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        0.0,
        0.0,
        1.0,
        100.0
    ) as Material4d<f64>;

    let transform: Iso4<f64> = na::one();

    let ball = @Box::new_with_margin(Vec4::new(5.0, 5.0, 5.0, 1.0), 0.0);

    let pos = na::append_translation(&transform, &Vec4::new(50.0, 50.0, 50.0, 1.0));

    let mut nodes = ~[];
    nodes.push(@SceneNode::new(~[blue], pos, ball));

    let scene = Scene::new(nodes, lights);
    let pixels = do scene.render(&resolution) |pt| {
        // Ray::new(eye, na::normalize(&(Vec4::new(pt.x, pt.y, pt.z, 1.0) - eye)))
        Ray::new(Vec4::new(pt.x, pt.y, pt.z, 0.0), na::normalize(&(Vec4::new(pt.x, pt.y, pt.z, 1.0) - eye)))
    };

    let path = "out.4d";
    let mut file = File::create(&Path::new(path)).expect("Cannot create the file: " + path);
    // let mut file = BufferedWriter::new(file);
    pixels.to_file(&mut file);
}
