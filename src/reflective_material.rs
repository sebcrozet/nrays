use std::num::Zero;
use nalgebra::na::{Dim, Transform, Rotate, AbsoluteRotate, Translation, AlgebraicVecExt, VecExt, Cast, Vec3};
use nalgebra::na;
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

impl<N:     'static + Cast<f32> + Send + Freeze + NumCast + Primitive + Algebraic + Signed + Float + Zero,
     V:     'static + AlgebraicVecExt<N> + Send + Freeze + Clone,
     Vless: VecExt<N> + Dim + Clone,
     M:     Translation<V> + Rotate<V> + Send + Freeze + Transform<V> + Mul<M, M> + AbsoluteRotate<V> + Dim>
Material<N, V, Vless, M> for ReflectiveMaterial {
    #[inline]
    fn compute(&self,
               ray:    &RayWithEnergy<V>,
               pt:     &V,
               normal: &V,
               _:      &Option<(N, N, N)>,
               scene:  &Scene<N, V, Vless, M>)
               -> Vec3<f32> {
        if !self.mix.is_zero() && ray.energy > 0.1 {
            let nproj      = normal * na::dot(&ray.ray.dir, normal);
            let rdir       = ray.ray.dir - nproj * na::cast(2.0);
            let new_energy = ray.energy - self.atenuation;

            scene.trace(
                &RayWithEnergy::new_with_energy(
                    pt + rdir * na::cast(0.001),
                    rdir,
                    new_energy))
        }
        else {
            na::zero()
        }
    }
}
