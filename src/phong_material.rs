use std::num;
use nalgebra::na::{Vec3, Vec4, Norm};
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
    texture:            Option<Texture2d>,
    alpha:              Option<Texture2d>,
    shininess:          f32 // FIXME: rename that
}

impl PhongMaterial {
    pub fn new(ambiant_color:  Vec3<f32>,
               diffuse_color:  Vec3<f32>,
               specular_color: Vec3<f32>,
               texture:        Option<Texture2d>,
               alpha:          Option<Texture2d>,
               shininess:      f32)
               -> PhongMaterial {
        PhongMaterial {
            diffuse_color:  diffuse_color,
            ambiant_color:  ambiant_color,
            specular_color: specular_color,
            texture:        texture,
            alpha:          alpha,
            shininess:      shininess
        }
    }
}

impl Material for PhongMaterial {
    fn ambiant(&self, _: &V, _: &V, uvs: &Option<Vec3<N>>) -> Vec4<f32> {
        // initialize with the ambiant color
        
        if na::dim::<V>() == 3 && uvs.is_some() {
            let mut tex_color = Vec4::new(1.0, 1.0, 1.0, 1.0);

            let uvs   = uvs.as_ref().unwrap();

            if self.texture.is_some() {
                let tex     = self.texture.as_ref().unwrap();
                tex_color   = tex.sample(uvs);
                tex_color.w = 1.0;
            }

            if self.alpha.is_some() {
                let alpha   = self.alpha.as_ref().unwrap();
                tex_color.w = alpha.sample(uvs).w;
            }

            let a = self.ambiant_color;

            Vec4::new(a.x, a.y, a.z, 1.0) * tex_color
        }
        else {
            let a = self.ambiant_color;
            Vec4::new(a.x, a.y, a.z, 1.0)
        }
    }

    fn compute(&self,
               ray:    &RayWithEnergy,
               point:  &V,
               normal: &V,
               uvs:    &Option<Vec3<N>>,
               scene:  &Scene)
               -> Vec4<f32> {
        // initialize with the ambiant color
        let mut res;
        let tex_color;
        let alpha;
        let mut nb_active_lights = 0.0f32;

        if na::dim::<V>() == 3 && uvs.is_some() && self.texture.is_some() {
            let uvs     = uvs.as_ref().unwrap();
            let tex     = self.texture.as_ref().unwrap();
            let texture = tex.sample(uvs);
            tex_color   = texture
        }
        else {
            tex_color = Vec4::new(1.0f32, 1.0, 1.0, 1.0)
        }

        if na::dim::<V>() == 3 && uvs.is_some() && self.alpha.is_some() {
            let uvs = uvs.as_ref().unwrap();
            let tex = self.alpha.as_ref().unwrap();
            alpha   = tex.sample(uvs).w;
        }
        else {
            alpha = 1.0;
        }

        let tex_color = Vec3::new(tex_color.x, tex_color.y, tex_color.z);
        // res = self.ambiant_color * tex_color;
        res = Vec3::new(0.0f32, 0.0, 0.0);

        // compute the contribution of each light
        for light in scene.lights().iter() {
            let mut acc = Vec3::new(0.0f32, 0.0, 0.0);
            light.sample(|pos| {
                let mut ldir = pos - *point;
                let     dist = ldir.normalize() - na::cast(0.001);

                match scene.intersects_ray(&Ray::new(point + ldir * na::cast::<f32, N>(0.001), ldir.clone()), dist) {
                    None         => { },
                    Some(filter) => {
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

                            acc = acc + light.color * filter * (diffuse);
                        }
                        else {
                            acc = acc + light.color * filter * diffuse;
                        }
                        nb_active_lights = nb_active_lights + 1.0f32;
                    }
                }
            });

            res = res + acc / ((light.racsample * light.racsample) as f32);
        }

        Vec4::new(res.x, res.y, res.z, alpha)
    }
}
