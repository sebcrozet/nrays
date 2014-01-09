#[crate_id = "sphere4d"];
#[crate_type = "bin"];
#[warn(non_camel_case_types)];
#[feature(managed_boxes)];

extern mod nalgebra;
extern mod ncollide = "ncollide4df64";
extern mod nrays    = "nrays4d";

use std::io::fs::File;
use nalgebra::na::{Iso4, Vec3, Vec4};
use nalgebra::na;
use ncollide::ray::Ray;
use ncollide::geom::{Ball, Box, Cone, Cylinder};
use nrays::scene_node::SceneNode;
use nrays::material::Material;
use nrays::phong_material::PhongMaterial;
use nrays::reflective_material::ReflectiveMaterial;
use nrays::scene::Scene;
use nrays::light::Light;

fn main() {
    let resolution = Vec3::new(100.0f64, 100.0, 100.0);

    let mut lights = ~[];

    {
        lights.push(Light::new(Vec4::new(0.0f64, 10.0, 10.0, 1.0),
                               0.0,
                               1,
                               Vec3::new(1.0, 1.0, 1.0)));
        lights.push(Light::new(Vec4::new(0.0f64, 10.0, 10.0, 1.0),
                               0.0,
                               1,
                               Vec3::new(1.0, 1.0, 1.0)));
        /*
        lights.push(Light::new(Vec4::new(0.0f64, -10.0, -10.0, 1.0),
                               Vec3::new(1.0, 1.0, 1.0)));
        lights.push(Light::new(Vec4::new(0.0f64, -10.0, 10.0, 1.0),
                               Vec3::new(1.0, 1.0, 1.0)));
        lights.push(Light::new(Vec4::new(0.0f64, 10.0, -10.0, 1.0),
                               Vec3::new(1.0, 1.0, 1.0)));
        lights.push(Light::new(Vec4::new(0.0f64, 10.0, 10.0, 1.0),
                               Vec3::new(1.0, 1.0, 1.0)));
                               */
    }

    let refl = ReflectiveMaterial::new(0.4, 0.2);
    let blue = @PhongMaterial::new(
        Vec3::new(0.1, 0.1, 0.1),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
        None,
        100.0
    ) as @Material;

    let transform: Iso4<f64> = na::one();

    let box_shape = @Box::new_with_margin(Vec4::new(0.25, 0.25, 0.25, 0.00000001), 0.000000001);
    let ball      = @Ball::new(0.25);
    let cone      = @Box::new_with_margin(Vec4::new(0.25, 0.25, 0.25, 0.00000001), 0.00000001); // Cone::new_with_margin(0.25, 0.25, 0.0);
    let cylinder  = @Cylinder::new_with_margin(0.25, 0.25, 0.0);

    let pos  = na::append_translation(&transform, &Vec4::new(0.0, 0.0, 0.0,    4.0));
    let pos2 = na::append_translation(&transform, &Vec4::new(0.75, 0.75, 0.0,  4.0));
    let pos3 = na::append_translation(&transform, &Vec4::new(0.0, 0.75, 0.75,  4.0));
    let pos4 = na::append_translation(&transform, &Vec4::new(0.0, 0.75, -0.75, 4.0));

    let mut nodes = ~[];
    nodes.push(@SceneNode::new(blue, refl, pos,  ball, None));
    nodes.push(@SceneNode::new(blue, refl, pos2, box_shape, None));
    nodes.push(@SceneNode::new(blue, refl, pos3, cone, None));
    nodes.push(@SceneNode::new(blue, refl, pos4, cylinder, None));
    // nodes.push(@SceneNode::new(~[white],  na::append_translation(&transform, &Vec4::new(0.0f64, 0.0f64, 0.0, 4.0)), plane));

    let scene = Scene::new(nodes, lights);
    let pixels = scene.render(&resolution, |pt| {
        let x = (pt.x / resolution.x - 0.5) * 2.0;
        let y = (pt.y / resolution.y - 0.5) * 2.0;
        let z = (pt.z / resolution.z - 0.5) * 2.0;
        // Ray::new(eye, na::normalize(&(Vec4::new(pt.x, pt.y, pt.z, 1.0) - eye)))

        Ray::new(Vec4::new(x, y, z, 0.0), Vec4::new(0.0, 0.0, 0.0, 1.0))
    });

    let path = "out.4d";
    let mut file = File::create(&Path::new(path)).expect("Cannot create the file: " + path);
    // let mut file = BufferedWriter::new(file);
    pixels.to_file(&mut file);
}
