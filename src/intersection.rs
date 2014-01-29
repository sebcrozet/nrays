use nalgebra::na;
use ncollide::ray::Ray;
use ncollide::math::{N, V};
use nalgebra::na::{Vec3, Vec4, Norm};

pub struct Intersection {
    pos: V,
    normal: V,
    intensity: N,
    color: Vec4<f32>
}

impl Intersection {
    pub fn new(pos: V, normal: V, intensity: N, c: Vec4<f32>) -> Intersection {
        Intersection {
            pos: pos,
            normal: normal,
            intensity: intensity,
            color: c
        }
    }
}
