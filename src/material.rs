use nalgebra::na::{Vec2, Vec4};
use ncollide::math::{Scalar, Vect};
use ray_with_energy::RayWithEnergy;
use scene::Scene;

pub trait Material {
    fn ambiant(&self, pt: &Vect, normal: &Vect, uv: &Option<Vec2<Scalar>>) -> Vec4<f32>;
    fn compute(&self,
               _:      &RayWithEnergy,
               pt:     &Vect,
               normal: &Vect,
               uv:     &Option<Vec2<Scalar>>,
               _:      &Scene)
               -> Vec4<f32> {
        self.ambiant(pt, normal, uv)
    }
}
