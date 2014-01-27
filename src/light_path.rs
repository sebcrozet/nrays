use ncollide::ray::Ray;
use ncollide::math::V;
use nalgebra::na::Vec3;
use nalgebra::na;

pub struct LightPath {
    ray:    Ray,
    energy: f32,
    color: Vec3<f32>,
    total_color: Vec3<f32>,
    normal_contact: V,
    last_intersection: V,
    no_hit: bool,
    total_weight: f32,
    mix_coef: f32

}

impl LightPath {
    pub fn new(orig: V, dir: V, col: Vec3<f32>) -> LightPath {
        LightPath {
            ray: Ray::new(orig, dir),
            energy: 1.0f32,
            color: col,
            total_color: col,
            normal_contact: dir,
            last_intersection: orig,
            total_weight: 0.0f32,
            no_hit: true,
            mix_coef: 1.0f32
        }
    }
}
