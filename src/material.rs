use nalgebra::na::Vec3;
use ncollide::math::{N, V};
use ray_with_energy::RayWithEnergy;
use scene::Scene;

pub trait Material {
    fn compute(&self,
               ray:    &RayWithEnergy,
               pt:     &V,
               normal: &V,
               uv:     &Option<Vec3<N>>,
               scene:  &Scene)
               -> Vec3<f32>;
}
