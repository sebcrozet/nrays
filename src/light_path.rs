use ncollide::ray::Ray;
use ncollide::math::V;
use nalgebra::na::Vec3;
use nalgebra::na;

pub struct LightPath {
    ray:    Ray,
    energy: f32,
    color: Vec3<f32>,
    last_intersection: V,
    mix_coef: f32

}

impl LightPath {
    pub fn new(orig: V, dir: V, col: Vec3<f32>) -> LightPath {
        LightPath {
            ray: Ray::new(orig, dir),
            energy: 1.0f32,
            color: col,
            last_intersection: orig,
            mix_coef: 1.0f32
        }
    }
}
