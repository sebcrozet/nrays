use nalgebra::na::Transform;
use ncollide::ray::{RayCast, Ray};
use ncollide::bounding_volume::{HasAABB, AABB};
use ncollide::math::{N, V, M};
use material::Material;
use reflective_material::ReflectiveMaterial;
use texture2d::Texture2d;

#[cfg(dim3)]
use nalgebra::na;

pub struct SceneNode {
    refl:      ReflectiveMaterial,
    material:  @Material,
    transform: M,
    geometry:  @RayCast,
    aabb:      AABB,
    nmap:      Option<Texture2d>

}

impl SceneNode {
    pub fn new<G: 'static + RayCast + HasAABB>(
               material:  @Material,
               refl:      ReflectiveMaterial,
               transform: M,
               geometry:  @G,
               nmap:      Option<Texture2d>)
               -> SceneNode {
        SceneNode {
            refl:      refl, 
            material:  material,
            geometry:  geometry as @RayCast,
            aabb:      geometry.aabb(&transform),
            transform: transform,
            nmap:      nmap
        }
    }

}

impl SceneNode {
    #[cfg(dim3)]
    pub fn cast(&self, r: &Ray) -> Option<(N, V, Option<(N, N, N)>)> {
        let res = self.geometry.toi_and_normal_and_uv_with_transform_and_ray(&self.transform, r);

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

                        n.x = na::cast(cn.x); 
                        n.y = na::cast(cn.y); 
                        n.z = na::cast(cn.z); 

                        Some((t, n, Some(uvs.clone())))
                    }
                }
            }
        }
    }

    #[cfg(dim4)]
    pub fn cast(&self, r: &Ray) -> Option<(N, V, Option<(N, N, N)>)> {
        self.geometry.toi_and_normal_with_transform_and_ray(&self.transform, r).map(|(n, v)|
            (n, v, None))
    }
}
