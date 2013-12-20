#[link(name     = "sphere3d"
       , vers   = "0.0"
       , author = "Sébastien Crozet"
       , uuid   = "cf8cfe5d-18ca-40cb-b596-d8790090a56d")];
#[crate_type = "bin"];
#[warn(non_camel_case_types)];
#[feature(managed_boxes)];
#[feature(globs)];

extern mod nalgebra;
extern mod ncollide;
extern mod nrays;

use std::num;
use std::io::buffered::BufferedWriter;
use std::io::fs::File;
// XXX: globing is bad
use nalgebra::na::*; // {Iso3, Vec2, Vec3, Vec4, Mat4, Inv, Identity};
use nalgebra::na;
use ncollide::ray::{Ray, RayCast, RayCastWithTransform};
use ncollide::ray::ray_implicit::gjk_toi_and_normal_with_ray;
use ncollide::bounding_volume::{AABB, HasAABB, implicit_shape_aabb};
use ncollide::geom::{Geom, Box, Ball, Plane, Cone, Cylinder, MinkowskiSum, Implicit, HasMargin};
use ncollide::narrow::algorithm::johnson_simplex::JohnsonSimplex;
use nrays::scene_node::SceneNode;
use nrays::material::Material3d;
use nrays::normal_material::NormalMaterial;
use nrays::phong_material::PhongMaterial;
use nrays::reflective_material::ReflectiveMaterial;
use nrays::scene::Scene;
use nrays::light::Light;

fn main() {
    let resolution = Vec2::new(1024.0, 1024.0);
    let mut lights = ~[];
    let mut nodes  = ~[];

    {
        lights.push(Light::new(Vec3::new(10.0f64, -10.0, 10.0),
                               Vec3::new(1.0, 1.0, 1.0)));
        lights.push(Light::new(Vec3::new(-10.0f64, -10.0, 10.0),
                               Vec3::new(1.0, 1.0, 1.0)));
        lights.push(Light::new(Vec3::new(10.0f64, 10.0, 10.0),
                               Vec3::new(1.0, 1.0, 1.0)));
        lights.push(Light::new(Vec3::new(-10.0f64, 10.0, 10.0),
                               Vec3::new(1.0, 1.0, 1.0)));
    }

    {
        // let white_material = Material::new(Vec4::new(1.0f64, 1.0, 1.0, 1.0));
        // let red_material = Material::new(Vec4::new(1.0f64, 0.0, 0.0, 1.0));
        // let blue_material = Material::new(Vec4::new(0.0f64, 0.0, 1.0, 1.0));
        // let green_material = Material::new(Vec4::new(0.0f64, 1.0, 0.0, 1.0));
        let blue = @PhongMaterial::new(
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            0.3,
            0.6,
            0.1,
            100.0
        ) as Material3d<f64>;

        let red = @PhongMaterial::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            0.3,
            0.6,
            0.1,
            100.0
        ) as Material3d<f64>;

        let green = @PhongMaterial::new(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            0.3,
            0.6,
            0.1,
            100.0
        ) as Material3d<f64>;

        let white = @PhongMaterial::new(
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            0.3,
            0.6,
            0.1,
            100.0
        ) as Material3d<f64>;

        let refl = @ReflectiveMaterial::new(0.2) as Material3d<f64>;

        let normal_material = @NormalMaterial::new() as Material3d<f64>;
        let transform: Iso3<f64> = na::one();

        let margin       = 0.0f64;
        let ball         = @Ball::new(1.0f64);
        let box_shape    = @Box::new_with_margin(Vec3::new(0.5f64, 0.5, 2.0), margin);
        let cone         = @Cone::new_with_margin(2.0f64, 0.5f64, margin);
        let cylinder     = @Cylinder::new_with_margin(1.0f64, 1.0, margin);
        let plane        = @Plane::new(Vec3::new(0.0f64, -2.0, -0.5));
        let cylinder_box = @ManagedMinkowskiSum::new(na::append_rotation(&transform, &Vec3::new(0.2, 0.0, 0.4)), cone, transform, cone);
        // let cone_cylinder_box = @ManagedMinkowskiSum::new(transform, cylinder_box, transform, cone);
        // FIXME: new_capsule is missing from ncollide
        // let capsule: G = Geom::new_capsule(Capsule::new(1.0f64, 1.0f64));

        let pi: f64 = num::Real::pi();
        let tcb = na::append_rotation(&transform, &Vec3::new(0.0f64, pi / 4.0, 0.0));
        let tcb = na::append_translation(&tcb, &Vec3::new(0.0f64, -2.0, 15.0));
        nodes.push(@SceneNode::new(~[blue, refl], tcb, cylinder_box));
        nodes.push(@SceneNode::new(~[refl, blue], na::append_translation(&transform, &Vec3::new(-4.0f64, 0.0, 15.0)), box_shape));
        nodes.push(@SceneNode::new(~[refl, green], na::append_translation(&transform, &Vec3::new(4.0f64, 0.0, 15.0)), cone));
        nodes.push(@SceneNode::new(~[refl, red], na::append_translation(&transform, &Vec3::new(0.0f64, -4.0f64, 15.0)), cylinder));
        nodes.push(@SceneNode::new(~[refl, white],  na::append_translation(&transform, &Vec3::new(0.0f64, 1.5f64, 15.0)), plane));
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
    let pixels = scene.render(&resolution, |pt| {
        let device_x = (pt.x / resolution.x - 0.5) * 2.0;
        let device_y = (pt.y / resolution.y - 0.5) * 2.0;
        let start = Vec4::new(device_x, device_y, -1.0, 1.0);
        let end   = Vec4::new(device_x, device_y, 1.0, 1.0);
        let h_eye = perspective * start;
        let h_at  = perspective * end;
        let eye: Vec3<f64> = na::from_homogeneous(&h_eye);
        let at:  Vec3<f64> = na::from_homogeneous(&h_at);
        Ray::new(eye, na::normalize(&(at - eye)))
    });

    let path = "out.ppm";
    let file = File::create(&Path::new(path)).expect("Cannot create the file: " + path);
    let mut file = BufferedWriter::new(file);
    pixels.to_ppm(&mut file);
}

struct ManagedMinkowskiSum<M, G1, G2> {
    // FIXME: explicitly use @ ?
    m1: M,
    g1: @G1,
    m2: M,
    g2: @G2
}

impl<M, G1, G2> ManagedMinkowskiSum<M, G1, G2> {
    pub fn new(m1: M, g1: @G1, m2: M, g2: @G2) -> ManagedMinkowskiSum<M, G1, G2> {
        ManagedMinkowskiSum {
            m1: m1,
            g1: g1,
            m2: m2,
            g2: g2
        }
    }
}

impl<N: Num + Algebraic,
     V: AlgebraicVec<N>,
     M,
     Id: Rotate<V> + Transform<V>,
     G1: Implicit<N, V, M>,
     G2: Implicit<N, V, M>>
Implicit<N, V, Id> for ManagedMinkowskiSum<M, G1, G2> {
    #[inline]
    // FIXME: using M here is incorrect
    fn support_point(&self, m: &Id, dir: &V) -> V {
        let ldir = m.inv_rotate(dir);
        m.transform(&(self.g1.support_point(&self.m1, &ldir) + self.g2.support_point(&self.m2, &ldir)))
    }

    #[inline]
    fn support_point_without_margin(&self, m: &Id, dir: &V) -> V {
        let ldir = m.inv_rotate(dir);
        m.transform(&(self.g1.support_point_without_margin(&self.m1, &ldir) +
                      self.g2.support_point_without_margin(&self.m2, &ldir)))
    }
}

impl<N: Algebraic + Num,
     V: AlgebraicVecExt<N>,
     M:  Rotate<V> + Transform<V>,
     G1: Implicit<N, V, M>,
     G2: Implicit<N, V, M>>
HasAABB<N, V, M> for ManagedMinkowskiSum<M, G1, G2> {
    fn aabb(&self, m: &M) -> AABB<N, V> {
        implicit_shape_aabb(m, self)
    }
}

impl<N:  Ord + Num + Float + Cast<f32> + Clone,
     V:  AlgebraicVecExt<N> + Clone,
     G1: Implicit<N, V, M>,
     G2: Implicit<N, V, M>,
     M>
RayCast<N, V> for ManagedMinkowskiSum<M, G1, G2> {
    fn toi_and_normal_with_ray(&self, ray: &Ray<V>) -> Option<(N, V)> {
        gjk_toi_and_normal_with_ray(&Identity::new(), self, &mut JohnsonSimplex::<N, V>::new_w_tls(), ray)
    }
}

impl<N: Num, M, G1: HasMargin<N>, G2: HasMargin<N>> HasMargin<N> for ManagedMinkowskiSum<M, G1, G2> {
    fn margin(&self) -> N {
        self.g1.margin() + self.g2.margin()
    }
}

impl<N:  Ord + Num + Float + Cast<f32> + Clone,
     V:  AlgebraicVecExt<N> + Clone,
     G1: Implicit<N, V, M>,
     G2: Implicit<N, V, M>,
     M:  Rotate<V> + Transform<V>>
RayCastWithTransform<N, V, M> for ManagedMinkowskiSum<M, G1, G2> { }
