use ncollide::ray::Ray;
use ncollide::math::V;
use nalgebra::na::Vec3;
use nalgebra::na;

pub struct Intersection {
    normal: V,
    color: Vec3<f32>,
    pos: V
}

impl Intersection {
    pub fn new(normal: V, pos: V, col: Vec3<f32>) -> LightPath {
        LightPath {
            normal: normal,
            pos: pos,
            color: col
        }
    }
}
