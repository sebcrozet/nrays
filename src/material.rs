use nalgebra::na::{Vec2, Vec3, Vec4, Iso3};
use scene::Scene;

pub type Material3d<N> = @Material<N, Vec3<N>, Vec2<uint>, Iso3<N>>;

pub trait Material<N, V, Vlessi, M> {
    fn compute(&self, pt: &V, normal: &V, scene: &Scene<N, V, Vlessi, M>) -> Vec4<f32>;
}
