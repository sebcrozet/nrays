use nalgebra::na::{Rotate, Transform, VecExt, Cast, Vec4, Vec3, Vec2, Iso3, Iso4};
use nalgebra::na;
use ncollide::ray::{RayCastWithTransform, Ray};
use ncollide::bounding_volume::{HasAABB, AABB};
use material::Material;
use reflective_material::ReflectiveMaterial;
use texture2d::Texture2d;

pub type SceneNode3d<N> = SceneNode<N, Vec3<N>, Vec2<N>, Iso3<N>>;
pub type SceneNode4d<N> = SceneNode<N, Vec4<N>, Vec3<N>, Iso4<N>>;

pub struct SceneNode<N, V, Vlessi, M> {
    refl:      ReflectiveMaterial,
    material:  @Material<N, V, Vlessi, M>,
    transform: M,
    geometry:  @RayCastWithTransform<N, V, M>,
    aabb:      AABB<N, V>,
    nmap:      Option<Texture2d>

}

impl<N, V, Vlessi, M> SceneNode<N, V, Vlessi, M> {
    pub fn new<G: 'static + RayCastWithTransform<N, V, M> + HasAABB<N, V, M>>(
               material:  @Material<N, V, Vlessi, M>,
               refl:      ReflectiveMaterial,
               transform: M,
               geometry:  @G,
               nmap:      Option<Texture2d>)
               -> SceneNode<N, V, Vlessi, M> {
        SceneNode {
            refl:      refl, 
            material:  material,
            geometry:  geometry as @RayCastWithTransform<N, V, M>,
            aabb:      geometry.aabb(&transform),
            transform: transform,
            nmap:      nmap
        }
    }

}

impl<N: Cast<f32> + NumCast + Clone, V: VecExt<N>, Vlessi, M: Transform<V> + Rotate<V>>
SceneNode<N, V, Vlessi, M> {
    pub fn cast(&self, r: &Ray<V>) -> Option<(N, V, Option<(N, N, N)>)> {
        let res = self.geometry.toi_and_normal_and_uv_with_transform_and_ray(&self.transform, r);

        if na::dim::<V>() != 3 {
            return res;
        }

        if res.is_none() {
            return None;
        }

        match self.nmap {
            None           => res,
            Some(ref nmap) => {
                let (t, n, uvs) = res.unwrap();

                match uvs {
                    None          => Some((t, n, None)),
                    Some(ref uvs) => {
                        let mut n = na::zero::<V>();
                        let cn    = (na::normalize(&nmap.sample(uvs)) - 0.5f32) * 2.0f32;

                        n.set(0, na::cast(cn.x)); 
                        n.set(1, na::cast(cn.y)); 
                        n.set(2, na::cast(cn.z)); 

                        Some((t, n, Some(uvs.clone())))
                    }
                }
            }
        }
    }
}
