use na::{Point2, Point4};
use math::{Scalar, Point, Vect};
use ray_with_energy::RayWithEnergy;
use scene::Scene;

pub trait Material {
    fn ambiant(&self, pt: &Point, normal: &Vect, uv: &Option<Point2<Scalar>>) -> Point4<f32>;
    fn compute(&self,
               _:      &RayWithEnergy,
               pt:     &Point,
               normal: &Vect,
               uv:     &Option<Point2<Scalar>>,
               _:      &Scene)
               -> Point4<f32> {
        self.ambiant(pt, normal, uv)
    }
}
