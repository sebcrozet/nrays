#[link(name     = "sphere3d"
       , vers   = "0.0"
       , author = "Sébastien Crozet"
       , uuid   = "cf8cfe5d-18ca-40cb-b596-d8790090a56d")];
#[crate_type = "bin"];
#[warn(non_camel_case_types)];

extern mod nalgebra;
extern mod ncollide;
extern mod nrays;

use std::io::buffered::BufferedWriter;
use std::io::fs::File;
use nalgebra::na::{Iso3, Vec2, Vec3, Vec4, Mat4, Inv};
use nalgebra::na;
use ncollide::ray::Ray;
use ncollide::geom::Geom;
use nrays::scene_node::SceneNode;
use nrays::material::Material3d;
use nrays::normal_material::NormalMaterial;
use nrays::phong_material::PhongMaterial;
use nrays::scene::Scene;
use nrays::light::Light;

fn main() {
    let resolution = Vec2::new(640.0, 640.0);
    let mut lights = ~[];
    let mut nodes  = ~[];

    {
        lights.push(Light::new(Vec3::new(10.0f64, -10.0, 10.0),
                               Vec3::new(0.0, 1.0, 0.0)));
        lights.push(Light::new(Vec3::new(-10.0f64, -10.0, 10.0),
                               Vec3::new(0.0, 1.0, 1.0)));
        lights.push(Light::new(Vec3::new(10.0f64, 10.0, 10.0),
                               Vec3::new(1.0, 1.0, 0.0)));
        lights.push(Light::new(Vec3::new(-10.0f64, 10.0, 10.0),
                               Vec3::new(1.0, 1.0, 1.0)));
    }

    {
        // let white_material = Material::new(Vec4::new(1.0f64, 1.0, 1.0, 1.0));
        // let red_material = Material::new(Vec4::new(1.0f64, 0.0, 0.0, 1.0));
        // let blue_material = Material::new(Vec4::new(0.0f64, 0.0, 1.0, 1.0));
        // let green_material = Material::new(Vec4::new(0.0f64, 1.0, 0.0, 1.0));
        let blue = @PhongMaterial::new(
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            0.5,
            0.3,
            0.2,
            10.0
        ) as Material3d<f64>;

        let red = @PhongMaterial::new(
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            0.5,
            0.3,
            0.2,
            10.0
        ) as Material3d<f64>;

        let normal_material = @NormalMaterial::new() as Material3d<f64>;
        let transform: Iso3<f64> = na::one();

        type G = Geom<f64, Vec3<f64>, Iso3<f64>>;
        let margin = 0.0f64;
        let ball: G = Geom::new_ball(1.0f64);
        let box:  G = Geom::new_box_with_margin(Vec3::new(1.0f64, 1.0, 1.0), margin);
        let cone: G = Geom::new_cone_with_margin(1.0f64, 1.0f64, margin);
        let cylinder: G = Geom::new_cylinder_with_margin(1.0f64, 1.0, margin);
        let plane:    G = Geom::new_plane(Vec3::new(0.0f64, -2.0, -0.5));
        // FIXME: new_capsule is missing from ncollide
        // let capsule: G = Geom::new_capsule(Capsule::new(1.0f64, 1.0f64));

        nodes.push(@SceneNode::new(blue, na::append_translation(&transform, &Vec3::new(0.0f64, 0.0, 15.0)), ball));
        nodes.push(@SceneNode::new(normal_material, na::append_translation(&transform, &Vec3::new(-4.0f64, 0.0, 15.0)), box));
        nodes.push(@SceneNode::new(normal_material, na::append_translation(&transform, &Vec3::new(4.0f64, 0.0, 15.0)), cone));
        nodes.push(@SceneNode::new(blue, na::append_translation(&transform, &Vec3::new(0.0f64, -4.0f64, 15.0)), cylinder));
        nodes.push(@SceneNode::new(red,  na::append_translation(&transform, &Vec3::new(0.0f64, 1.5f64, 15.0)), plane));
        // nodes.push(@SceneNode::new(green_material, transform.translated(&Vec3::new(0.0f64, 5.0f64, 15.0)), capsule));
    }

    // FIXME: new_perspective is _not_ accessible as a free function.
    let mut perspective = Mat4::new_perspective(
        resolution.x,
        resolution.y,
        45.0f64 * 3.14 / 180.0,
        1.0,
        100000.0);

    perspective.inv();

    let scene  = Scene::new(nodes, lights);
    let pixels = do scene.render(&resolution) |pt| {
        let device_x = (pt.x / resolution.x - 0.5) * 2.0;
        let device_y = (pt.y / resolution.y - 0.5) * 2.0;
        let start = Vec4::new(device_x, device_y, -1.0, 1.0);
        let end   = Vec4::new(device_x, device_y, 1.0, 1.0);
        let h_eye = perspective * start;
        let h_at  = perspective * end;
        let eye: Vec3<f64> = na::from_homogeneous(&h_eye);
        let at:  Vec3<f64> = na::from_homogeneous(&h_at);
        Ray::new(eye, na::normalize(&(at - eye)))
    };

    let path = "out.ppm";
    let file = File::create(&Path::new(path)).expect("Cannot create the file: " + path);
    let mut file = BufferedWriter::new(file);
    pixels.to_ppm(&mut file);
}
