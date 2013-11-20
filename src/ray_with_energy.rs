use nalgebra::na::Vec;
use ncollide::ray::Ray;

pub struct RayWithEnergy<V> {
    ray:    Ray<V>,
    energy: f32
}

impl<N, V: Vec<N>> RayWithEnergy<V> {
    pub fn new(orig: V, dir: V) -> RayWithEnergy<V> {
        RayWithEnergy::new_with_energy(orig, dir, 1.0)
    }

    pub fn new_with_energy(orig: V, dir: V, energy: f32) -> RayWithEnergy<V> {
        RayWithEnergy {
            ray:    Ray::new(orig, dir),
            energy: energy
        }
    }
}
