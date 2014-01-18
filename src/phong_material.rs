use std::num;
use nalgebra::na::{Vec3, Norm};
use nalgebra::na;
use ncollide::math::{N, V};
use ncollide::ray::Ray;
use ray_with_energy::RayWithEnergy;
use scene::Scene;
use material::Material;
use texture2d::Texture2d;


pub struct PhongMaterial {
    diffuse_color:      Vec3<f32>,
    ambiant_color:      Vec3<f32>,
    specular_color:     Vec3<f32>,
    texture:            Option<Texture2d>, // FIXME: put this on an ARC?
    shininess:          f32 // FIXME: rename that
}

impl PhongMaterial {
    pub fn new(ambiant_color:      Vec3<f32>,
               diffuse_color:      Vec3<f32>,
               specular_color:     Vec3<f32>,
               texture:            Option<Texture2d>,
               shininess:          f32)
               -> PhongMaterial {
        PhongMaterial {
            diffuse_color:  diffuse_color,
            ambiant_color:  ambiant_color,
            specular_color: specular_color,
            texture:        texture,
            shininess:      shininess
        }
    }
}

impl Material for PhongMaterial {
    fn compute(&self,
               ray:    &RayWithEnergy,
               point:  &V,
               normal: &V,
               uvs:    &Option<(N, N, N)>,
               scene:  &Scene)
               -> Vec3<f32> {
        // initialize with the ambiant color
        let mut res;
        let tex_color;
        
        if na::dim::<V>() == 3 && uvs.is_some() && self.texture.is_some() {
            let uvs     = uvs.as_ref().unwrap();
            let tex     = self.texture.as_ref().unwrap();
            let texture = tex.sample(uvs);
            tex_color   = texture / 2.0f32
        }
        else {
            tex_color = Vec3::new(1.0f32, 1.0, 1.0)
        }

        res = self.ambiant_color * tex_color;

        // compute the contribution of each light
        for light in scene.lights().iter() {
            let mut acc = Vec3::new(0.0f32, 0.0, 0.0);
            light.sample(|pos| {
                let mut ldir = pos - *point;
                let     dist = ldir.normalize() - na::cast(0.001);

                if !scene.intersects_ray(&Ray::new(point + ldir * na::cast::<f32, N>(0.001), ldir.clone()), dist) {
                    let dot_ldir_norm = na::dot(&ldir, normal);

                    // diffuse
                    let dcoeff: f32   = NumCast::from(dot_ldir_norm.clone()).expect("[0] Conversion failed.");
                    let dcoeff        = dcoeff.max(&0.0);
                    let diffuse_color = self.diffuse_color * tex_color;

                    let diffuse = diffuse_color * dcoeff;

                    // specular
                    let lproj = normal * dot_ldir_norm;
                    let rldir = na::normalize(&(-ldir + lproj * na::cast::<f32, N>(2.0)));

                    let scoeff: f32 = NumCast::from(-na::dot(&rldir, &ray.ray.dir)).expect("[1] Conversion failed.");
                    if scoeff > na::zero() {
                        let scoeff   = num::powf(scoeff.clone(), self.shininess);
                        let specular = self.specular_color * scoeff;

                        acc = acc + light.color * (diffuse + specular);
                    }
                    else {
                        acc = acc + light.color * diffuse;
                    }
                }
            });

            res = res + acc / ((light.racsample * light.racsample) as f32);
        }

        res
    }
}
