use na::{Pnt2, Pnt3, Pnt4, Vec3, Norm, Axpy};
use na;
use ncollide::ray::Ray;
use math::{Scalar, Point, Vect};
use ray_with_energy::RayWithEnergy;
use scene::Scene;
use material::Material;
use texture2d::Texture2d;


pub struct PhongMaterial {
    diffuse_color:      Pnt3<f32>,
    ambiant_color:      Pnt3<f32>,
    specular_color:     Pnt3<f32>,
    texture:            Option<Texture2d>,
    alpha:              Option<Texture2d>,
    shininess:          f32 // FIXME: rename that
}

impl PhongMaterial {
    pub fn new(ambiant_color:  Pnt3<f32>,
               diffuse_color:  Pnt3<f32>,
               specular_color: Pnt3<f32>,
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
    fn ambiant(&self, _: &Point, _: &Vect, uvs: &Option<Pnt2<Scalar>>) -> Pnt4<f32> {
        // initialize with the ambiant color
        
        if na::dim::<Vect>() == 3 && uvs.is_some() {
            let mut tex_color = Pnt4::new(1.0, 1.0, 1.0, 1.0);

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

            Pnt4::new(a.x * tex_color.x, a.y * tex_color.y, a.z * tex_color.z, 1.0 * tex_color.w)
        }
        else {
            let a = self.ambiant_color;
            Pnt4::new(a.x, a.y, a.z, 1.0)
        }
    }

    fn compute(&self,
               ray:    &RayWithEnergy,
               point:  &Point,
               normal: &Vect,
               uvs:    &Option<Pnt2<Scalar>>,
               scene:  &Scene)
               -> Pnt4<f32> {
        // initialize with the ambiant color
        let mut res;
        let tex_color;
        let alpha;
        
        if na::dim::<Vect>() == 3 && uvs.is_some() && self.texture.is_some() {
            let uvs     = uvs.as_ref().unwrap();
            let tex     = self.texture.as_ref().unwrap();
            let texture = tex.sample(uvs);
            tex_color   = texture
        }
        else {
            tex_color = Pnt4::new(1.0f32, 1.0, 1.0, 1.0)
        }

        if na::dim::<Vect>() == 3 && uvs.is_some() && self.alpha.is_some() {
            let uvs = uvs.as_ref().unwrap();
            let tex = self.alpha.as_ref().unwrap();
            alpha   = tex.sample(uvs).w;
        }
        else {
            alpha = 1.0;
        }

        let tex_color = Pnt3::new(tex_color.x, tex_color.y, tex_color.z);
        res = *self.ambiant_color.as_vec() * *tex_color.as_vec();

        // compute the contribution of each light
        for light in scene.lights().iter() {
            let mut acc = Vec3::new(0.0f32, 0.0, 0.0);
            light.sample(|pos| {
                let mut ldir = pos - *point;
                let     dist = ldir.normalize() - na::cast(0.001f64);

                match scene.intersects_ray(&Ray::new(point + ldir * na::cast::<f32, Scalar>(0.001), ldir.clone()), dist) {
                    None         => { },
                    Some(filter) => {
                        let dot_ldir_norm = na::dot(&ldir, normal);

                        // diffuse
                        let dcoeff: f32   = NumCast::from(dot_ldir_norm.clone()).expect("[0] Conversion failed.");
                        let dcoeff        = dcoeff.max(0.0);
                        let diffuse_color = *self.diffuse_color.as_vec() * *tex_color.as_vec();

                        let diffuse = diffuse_color * dcoeff;

                        // specular
                        let lproj = normal * dot_ldir_norm;
                        let rldir = na::normalize(&(-ldir + lproj * na::cast::<f32, Scalar>(2.0)));

                        let scoeff: f32 = NumCast::from(-na::dot(&rldir, &ray.ray.dir)).expect("[1] Conversion failed.");
                        if scoeff > na::zero() {
                            let scoeff   = scoeff.clone().powf(self.shininess);
                            let specular = self.specular_color * scoeff;

                            acc = acc + light.color.as_vec() * filter * (diffuse + *specular.as_vec());
                        }
                        else {
                            acc = acc + light.color.as_vec() * filter * diffuse;
                        }
                    }
                }
            });

            res.axpy(&( 1.0 / (light.racsample * light.racsample) as f32), &acc);
        }

        Pnt4::new(res.x, res.y, res.z, alpha)
    }
}
