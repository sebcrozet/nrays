use extra::arc::Arc;
use nalgebra::na::Transform;
use ncollide::ray::{RayCast, Ray, RayIntersection};
use ncollide::bounding_volume::{HasAABB, AABB};
use ncollide::math::M;
use material::Material;
use reflective_material::ReflectiveMaterial;
use texture2d::Texture2d;

#[cfg(dim3)]
use nalgebra::na;

pub struct SceneNode {
    refl:      ReflectiveMaterial,
    material:  Arc<~Material:Freeze+Send>,
    transform: M,
    geometry:  ~RayCast:Freeze+Send,
    aabb:      AABB,
    nmap:      Option<Texture2d>
}

impl SceneNode {
    pub fn new<G: 'static + Send + Freeze + RayCast + HasAABB>(
               material:  Arc<~Material:Freeze+Send>,
               refl:      ReflectiveMaterial,
               transform: M,
               geometry:  ~G,
               nmap:      Option<Texture2d>)
               -> SceneNode {
        SceneNode {
            refl:      refl, 
            material:  material,
            aabb:      geometry.aabb(&transform),
            geometry:  geometry as ~RayCast:Freeze+Send,
            transform: transform,
            nmap:      nmap
        }
    }

}

impl SceneNode {
    #[cfg(dim3)]
    pub fn cast(&self, r: &Ray) -> Option<RayIntersection> {
        let res = self.geometry.toi_and_normal_and_uv_with_transform_and_ray(&self.transform, r);

        if res.is_none() {
            return None;
        }

        match self.nmap {
            None           => res,
            Some(ref nmap) => {
                let mut inter = res.unwrap();

                match inter.uvs {
                    None          => Some(inter),
                    Some(ref uvs) => {
                        let cn = (na::normalize(&nmap.sample(uvs)) - 0.5f32) * 2.0f32;

                        inter.normal.x = na::cast(cn.x); 
                        inter.normal.y = na::cast(cn.y); 
                        inter.normal.z = na::cast(cn.z); 

                        Some(inter)
                    }
                }
            }
        }
    }

    #[cfg(dim4)]
    pub fn cast(&self, r: &Ray) -> Option<RayIntersection> {
        self.geometry.toi_and_normal_with_transform_and_ray(&self.transform, r)
    }
}
