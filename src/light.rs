use nalgebra::na::Vec3;

// FIXME: this is a point light
// Implemente other types of lights
pub struct Light<V> {
    pos:   V,
    color: Vec3<f32>
}

impl<V> Light<V> {
    pub fn new(pos: V, color: Vec3<f32>) -> Light<V> {
        Light {
            pos:   pos,
            color: color
        }
    }
}
