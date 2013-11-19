use nalgebra::na::{Dim, Transform, Rotate, AbsoluteRotate, Translation, AlgebraicVecExt, VecExt, Cast, Vec4};
use nalgebra::na;
use ncollide::ray::Ray;
use scene::Scene;
use material::Material;

pub struct ReflectiveMaterial;

impl<N:     'static + Cast<f32> + Send + Freeze + NumCast + Primitive + Algebraic + Signed + Float,
     V:     'static + AlgebraicVecExt<N> + Send + Freeze + Clone,
     Vless: VecExt<N> + Dim + Clone,
     M:     Translation<V> + Rotate<V> + Send + Freeze + Transform<V> + Mul<M, M> + AbsoluteRotate<V> + Dim>
Material<N, V, Vless, M> for ReflectiveMaterial {
    #[inline]
    fn compute(&self,
               ray:    &Ray<V>,
               pt:     &V,
               normal: &V,
               scene:  &Scene<N, V, Vless, M>)
               -> Vec4<f32> {
        let nproj = normal * na::dot(&ray.dir, normal);
        let rdir  = -ray.dir + nproj * na::cast(2.0);

        scene.trace(&Ray::new(pt.clone(), rdir))
    }
}
