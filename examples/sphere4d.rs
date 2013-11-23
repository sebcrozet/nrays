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
use ncollide::geom::{Ball, Box, Plane, Cone, Cylinder};
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
        lights.push(Light::new(Vec4::new(10.0f64, -10.0, 10.0, 1.0),
                               Vec3::new(1.0, 1.0, 1.0)));
        lights.push(Light::new(Vec4::new(-10.0f64, -10.0, 10.0, 1.0),
                               Vec3::new(1.0, 1.0, 1.0)));
        lights.push(Light::new(Vec4::new(10.0f64, 10.0, 10.0, 1.0),
                               Vec3::new(1.0, 1.0, 1.0)));
        lights.push(Light::new(Vec4::new(-10.0f64, 10.0, 10.0, 1.0),
                               Vec3::new(1.0, 1.0, 1.0)));
    }

    let refl = @ReflectiveMaterial::new(0.2) as Material4d<f64>;
    let blue = @PhongMaterial::new(
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
        0.6,
        0.0,
        0.4,
        100.0
    ) as Material4d<f64>;
    let white = @PhongMaterial::new(
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
        0.0,
        0.0,
        1.0,
        100.0
    ) as Material4d<f64>;

    let transform: Iso4<f64> = na::one();

    let box      = @Box::new_with_margin(Vec4::new(0.25, 0.25, 0.25, 0.25), 0.0);
    let ball     = @Ball::new(0.25);
    let cone     = @Cone::new_with_margin(0.25, 0.25, 0.0);
    let cylinder = @Cylinder::new_with_margin(0.25, 0.25, 0.0);
    let plane    = @Plane::new(Vec4::new(0.0, 0.0, 0.0, -1.0));

    let pos  = na::append_translation(&transform, &Vec4::new(0.0, 0.0, 0.0,    4.0));
    let pos2 = na::append_translation(&transform, &Vec4::new(0.75, 0.75, 0.0,  4.0));
    let pos3 = na::append_translation(&transform, &Vec4::new(0.0, 0.75, 0.75,  4.0));
    let pos4 = na::append_translation(&transform, &Vec4::new(0.0, 0.75, -0.75, 4.0));

    let mut nodes = ~[];
    nodes.push(@SceneNode::new(~[refl, blue], pos,  ball));
    nodes.push(@SceneNode::new(~[refl, blue], pos2, box));
    nodes.push(@SceneNode::new(~[refl, blue], pos3, cone));
    nodes.push(@SceneNode::new(~[refl, blue], pos4, cylinder));
    // nodes.push(@SceneNode::new(~[white],  na::append_translation(&transform, &Vec4::new(0.0f64, 0.0f64, 0.0, 4.0)), plane));

    let scene = Scene::new(nodes, lights);
    let pixels = do scene.render(&resolution) |pt| {
        let x = (pt.x / resolution.x - 0.5) * 2.0;
        let y = (pt.y / resolution.y - 0.5) * 2.0;
        let z = (pt.z / resolution.z - 0.5) * 2.0;
        // Ray::new(eye, na::normalize(&(Vec4::new(pt.x, pt.y, pt.z, 1.0) - eye)))

        Ray::new(Vec4::new(x, y, z, 0.0), Vec4::new(0.0, 0.0, 0.0, 1.0))
    };

    let path = "out.4d";
    let mut file = File::create(&Path::new(path)).expect("Cannot create the file: " + path);
    // let mut file = BufferedWriter::new(file);
    pixels.to_file(&mut file);
}
