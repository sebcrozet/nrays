use std::num::Zero;
use nalgebra::na::Vec3;
use nalgebra::na;
use ncollide::math::{N, V};
use ray_with_energy::RayWithEnergy;
use scene::Scene;
use material::Material;

pub struct ReflectiveMaterial {
    mix:        f32,
    atenuation: f32
}

impl ReflectiveMaterial {
    pub fn new(mix: f32,atenuation: f32) -> ReflectiveMaterial {
        ReflectiveMaterial {
            atenuation: atenuation,
            mix:        mix,
        }
    }
}

impl Material for ReflectiveMaterial {
    #[inline]
    fn compute(&self,
               ray:    &RayWithEnergy,
               pt:     &V,
               normal: &V,
               _:      &Option<(N, N, N)>,
               scene:  &Scene)
               -> Vec3<f32> {
        if !self.mix.is_zero() && ray.energy > 0.1 {
            let nproj      = normal * na::dot(&ray.ray.dir, normal);
            let rdir       = ray.ray.dir - nproj * na::cast::<f32, N>(2.0);
            let new_energy = ray.energy - self.atenuation;

            scene.trace(
                &RayWithEnergy::new_with_energy(
                    pt + rdir * na::cast::<f32, N>(0.001),
                    rdir,
                    new_energy))
        }
        else {
            na::zero()
        }
    }
}
