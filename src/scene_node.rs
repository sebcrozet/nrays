use material::Material;
use math::{Isometry, Scalar};
use ncollide3d::bounding_volume::{HasBoundingVolume, AABB};
use ncollide3d::query::{Ray, RayCast, RayIntersection};
use std::sync::Arc;
use texture2d::Texture2d;

pub struct SceneNode {
    pub refl_mix: f32,
    pub refl_atenuation: f32,
    pub refr_coeff: Scalar,
    pub alpha: f32,
    pub solid: bool,
    pub material: Arc<Box<Material + Sync + Send>>,
    pub transform: Isometry,
    pub geometry: Box<RayCast<Scalar> + Sync + Send>,
    pub aabb: AABB<Scalar>,
    pub nmap: Option<Texture2d>,
}

impl SceneNode {
    pub fn new<
        G: 'static + Send + Sync + RayCast<Scalar> + HasBoundingVolume<Scalar, AABB<Scalar>>,
    >(
        material: Arc<Box<Material + Sync + Send>>,
        refl_mix: f32,
        refl_atenuation: f32,
        alpha: f32,
        refr_coeff: Scalar,
        transform: Isometry,
        geometry: Box<G>,
        nmap: Option<Texture2d>,
        solid: bool,
    ) -> SceneNode {
        SceneNode {
            refl_mix: refl_mix,
            refl_atenuation: refl_atenuation,
            alpha: alpha,
            refr_coeff: refr_coeff,
            material: material,
            aabb: geometry.bounding_volume(&transform),
            geometry: geometry as Box<RayCast<Scalar> + Sync + Send>,
            transform: transform,
            nmap: nmap,
            solid: solid,
        }
    }
}

impl SceneNode {
    pub fn cast(&self, r: &Ray<Scalar>) -> Option<RayIntersection<Scalar>> {
        let res = self
            .geometry
            .toi_and_normal_and_uv_with_ray(&self.transform, r, self.solid);

        if res.is_none() {
            return None;
        }

        match self.nmap {
            None => res,
            Some(ref nmap) => {
                let mut inter = res.unwrap();

                if let Some(ref uvs) = inter.uvs {
                    let shift_color = nmap.sample(uvs);
                    let shift = (shift_color.x + shift_color.y + shift_color.z) / 3.0;

                    inter.toi = inter.toi - shift as f64;
                }

                Some(inter)
            }
        }
    }
}
