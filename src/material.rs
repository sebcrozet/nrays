use nalgebra::na::{Vec2, Vec3, Vec4, Iso3, Iso4};
use ray_with_energy::RayWithEnergy;
use scene::Scene;

pub type Material3d<N> = @Material<N, Vec3<N>, Vec2<N>, Iso3<N>>;
pub type Material4d<N> = @Material<N, Vec4<N>, Vec3<N>, Iso4<N>>;

pub trait Material<N, V, Vless, M> {
    fn compute(&self,
               ray:    &RayWithEnergy<V>,
               pt:     &V,
               normal: &V,
               scene:  &Scene<N, V, Vless, M>) -> Vec4<f32>;
}
