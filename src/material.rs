use nalgebra::na::{Vec3, Vec4};
use ncollide::math::{N, V};
use ray_with_energy::RayWithEnergy;
use scene::Scene;

pub trait Material {
    fn ambiant(&self, pt: &V, normal: &V, uv: &Option<Vec3<N>>) -> Vec4<f32>;
    fn compute(&self,
               _:      &RayWithEnergy,
               pt:     &V,
               normal: &V,
               uv:     &Option<Vec3<N>>,
               _:      &Scene)
               -> Vec4<f32> {
        self.ambiant(pt, normal, uv)
    }
}
