use std::sync::Arc;
use na::Transform;
use ncollide::ray::{RayCast, Ray, RayIntersection};
use ncollide::bounding_volume::{HasAABB, AABB};
use math::{Scalar, Point, Vect, Matrix};
use material::Material;
use texture2d::Texture2d;

#[cfg(feature = "3d")]
use na;

pub struct SceneNode {
    pub refl_mix:        f32,
    pub refl_atenuation: f32,
    pub refr_coeff:      Scalar,
    pub alpha:           f32,
    pub solid:           bool,
    pub material:        Arc<Box<Material + Sync + Send>>,
    pub transform:       Matrix,
    pub geometry:        Box<RayCast<Scalar, Point, Vect, Matrix> + Sync + Send>,
    pub aabb:            AABB<Point>,
    pub nmap:            Option<Texture2d>
}

impl SceneNode {
    pub fn new<G: 'static + Send + Sync + RayCast<Scalar, Point, Vect, Matrix> + HasAABB<Point, Matrix>>(
               material:        Arc<Box<Material + Sync + Send>>,
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
            geometry:        geometry as Box<RayCast<Scalar, Point, Vect, Matrix> + Sync + Send>,
            transform:       transform,
            nmap:            nmap,
            solid:           solid
        }
    }

}

#[cfg(feature = "3d")]
impl SceneNode {
    pub fn cast(&self, r: &Ray<Point, Vect>) -> Option<RayIntersection<Scalar, Vect>> {
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
}

#[cfg(feature = "4d")]
impl SceneNode {
    pub fn cast(&self, r: &Ray<Point, Vect>) -> Option<RayIntersection<Scalar, Vect>> {
        self.geometry.toi_and_normal_with_transform_and_ray(
            &self.transform,
            r,
            self.solid)
    }
}
