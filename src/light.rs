use nalgebra::na::Vec3;
use ncollide::math::V;

// FIXME: this is a point light
// Implemente other types of lights
pub struct Light {
    pos:   V,
    color: Vec3<f32>
}

impl Light {
    pub fn new(pos: V, color: Vec3<f32>) -> Light {
        Light {
            pos:   pos,
            color: color
        }
    }
}
