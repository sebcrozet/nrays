use std::sync::Arc;
use nalgebra::na::Transform;
use ncollide::ray::{RayCast, Ray, RayIntersection};
use ncollide::bounding_volume::{HasAABB, AABB};
use ncollide::math::{Scalar, Matrix};
use material::Material;
use texture2d::Texture2d;

#[cfg(dim3)]
use nalgebra::na;

pub struct SceneNode {
    pub refl_mix:        f32,
    pub refl_atenuation: f32,
    pub refr_coeff:      Scalar,
    pub alpha:           f32,
    pub solid:           bool,
    pub material:        Arc<Box<Material:Share+Send>>,
    pub transform:       Matrix,
    pub geometry:        Box<RayCast:Share+Send>,
    pub aabb:            AABB,
    pub nmap:            Option<Texture2d>
}

impl SceneNode {
    pub fn new<G: 'static + Send + Share + RayCast + HasAABB>(
               material:        Arc<Box<Material:Share+Send>>,
               refl_mix:        f32,
               refl_atenuation: f32,
               alpha:           f32,
               refr_coeff:      Scalar,
               transform:       Matrix,
               geometry:        Box<G>,
               nmap:            Option<Texture2d>,
               solid:           bool)
               -> SceneNode {
        SceneNode {
            refl_mix:        refl_mix,
            refl_atenuation: refl_atenuation,
            alpha:           alpha,
            refr_coeff:      refr_coeff,
            material:        material,
            aabb:            geometry.aabb(&transform),
            geometry:        geometry as Box<RayCast:Share+Send>,
            transform:       transform,
            nmap:            nmap,
            solid:           solid
        }
    }

}

impl SceneNode {
    #[cfg(dim3)]
    pub fn cast(&self, r: &Ray) -> Option<RayIntersection> {
        let res = self.geometry.toi_and_normal_and_uv_with_transform_and_ray(&self.transform, r, self.solid);

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
                        let shift_color = nmap.sample(uvs);
                        let shift       = (shift_color.x + shift_color.y + shift_color.z) / 3.0;

                        inter.toi = inter.toi - na::cast(shift);
                        // inter.normal.x = na::cast(cn.x); 
                        // inter.normal.y = na::cast(cn.y); 
                        // inter.normal.z = na::cast(cn.z); 

                        Some(inter)
                    }
                }
            }
        }
    }

    #[cfg(dim4)]
    pub fn cast(&self, r: &Ray) -> Option<RayIntersection> {
        self.geometry.toi_and_normal_with_transform_and_ray(
            &self.transform,
            r,
            self.solid)
    }
}
