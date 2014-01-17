use nalgebra::na::Vec3;
use ncollide::math::{N, V};
use ray_with_energy::RayWithEnergy;
use light_path::LightPath;
use scene::Scene;

pub trait Material {
    fn compute(&self,
               ray:    &RayWithEnergy,
               pt:     &V,
               normal: &V,
               uv:     &Option<(N, N, N)>,
               scene:  &Scene) -> Vec3<f32>;

    // Returns the position of the light's avatar
    fn compute_for_light_path(&self,
               path:   &mut LightPath,
               pt:     &V,
               normal: &V,
               uv:     &Option<(N, N, N)>,
               scene:  &Scene);
}
