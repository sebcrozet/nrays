use std::sync::Arc;
use na::Transform;
use ncollide::ray::{RayCast, Ray, RayIntersection};
use ncollide::bounding_volume::{AABB, HasBoundingVolume};
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
    pub geometry:        Box<RayCast<Point, Matrix> + Sync + Send>,
    pub aabb:            AABB<Point>,
    pub nmap:            Option<Texture2d>
}

impl SceneNode {
    pub fn new<G: 'static + Send + Sync + RayCast<Point, Matrix> + HasBoundingVolume<Matrix, AABB<Point>>> (
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
            aabb:            geometry.bounding_volume(&transform),
            geometry:        geometry as Box<RayCast<Point, Matrix> + Sync + Send>,
            transform:       transform,
            nmap:            nmap,
            solid:           solid
        }
    }

}

#[cfg(feature = "3d")]
impl SceneNode {
    pub fn cast(&self, r: &Ray<Point>) -> Option<RayIntersection<Vect>> {
        let res = self.geometry.toi_and_normal_and_uv_with_ray(&self.transform, r, self.solid);

        if res.is_none() {
            return None;
        }

        match self.nmap {
            None           => res,
            Some(ref nmap) => {
                let mut inter = res.unwrap();

                if let Some(ref uvs) = inter.uvs {
                    let shift_color = nmap.sample(uvs);
                    let shift       = (shift_color.x + shift_color.y + shift_color.z) / 3.0;

                    inter.toi = inter.toi - na::cast::<f32, Scalar>(shift);
                }

                Some(inter)
            }
        }
    }
}

#[cfg(feature = "4d")]
impl SceneNode {
    pub fn cast(&self, r: &Ray<Point>) -> Option<RayIntersection<Vect>> {
        self.geometry.toi_and_normal_with_ray(&self.transform, r, self.solid)
    }
}
