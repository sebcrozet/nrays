use na::{Point2, Point3, Point4, Vector3, Norm, Axpy};
use na;
use ncollide::ray::Ray;
use math::{Scalar, Point, Vect};
use ray_with_energy::RayWithEnergy;
use scene::Scene;
use material::Material;
use texture2d::Texture2d;


pub struct PhongMaterial {
    diffuse_color:      Point3<f32>,
    ambiant_color:      Point3<f32>,
    specular_color:     Point3<f32>,
    texture:            Option<Texture2d>,
    alpha:              Option<Texture2d>,
    shininess:          f32 // FIXME: rename that
}

impl PhongMaterial {
    pub fn new(ambiant_color:  Point3<f32>,
               diffuse_color:  Point3<f32>,
               specular_color: Point3<f32>,
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
    fn ambiant(&self, _: &Point, _: &Vect, uvs: &Option<Point2<Scalar>>) -> Point4<f32> {
        // initialize with the ambiant color
        
        if uvs.is_some() {
            let mut tex_color = Point4::new(1.0, 1.0, 1.0, 1.0);

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

            Point4::new(a.x * tex_color.x, a.y * tex_color.y, a.z * tex_color.z, 1.0 * tex_color.w)
        }
        else {
            let a = self.ambiant_color;
            Point4::new(a.x, a.y, a.z, 1.0)
        }
    }

    fn compute(&self,
               ray:    &RayWithEnergy,
               point:  &Point,
               normal: &Vect,
               uvs:    &Option<Point2<Scalar>>,
               scene:  &Scene)
               -> Point4<f32> {
        // initialize with the ambiant color
        let mut res;
        let tex_color;
        let alpha;
        
        if uvs.is_some() && self.texture.is_some() {
            let uvs     = uvs.as_ref().unwrap();
            let tex     = self.texture.as_ref().unwrap();
            let texture = tex.sample(uvs);
            tex_color   = texture
        }
        else {
            tex_color = Point4::new(1.0f32, 1.0, 1.0, 1.0)
        }

        if uvs.is_some() && self.alpha.is_some() {
            let uvs = uvs.as_ref().unwrap();
            let tex = self.alpha.as_ref().unwrap();
            alpha   = tex.sample(uvs).w;
        }
        else {
            alpha = 1.0;
        }

        let tex_color = Point3::new(tex_color.x, tex_color.y, tex_color.z);
        res = *self.ambiant_color.as_vector() * *tex_color.as_vector();

        // compute the contribution of each light
        for light in scene.lights().iter() {
            let mut acc = Vector3::new(0.0f32, 0.0, 0.0);
            light.sample(&mut |pos| {
                let mut ldir = pos - *point;
                let     dist = ldir.normalize_mut() - na::cast::<f64, Scalar>(0.001f64);

                match scene.intersects_ray(&Ray::new(*point + ldir * na::cast::<f32, Scalar>(0.001), ldir.clone()), dist) {
                    None         => { },
                    Some(filter) => {
                        let dot_ldir_norm = na::dot(&ldir, normal);

                        // diffuse
                        let dcoeff: f32   = na::cast(dot_ldir_norm.clone());
                        let dcoeff        = dcoeff.max(0.0);
                        let diffuse_color = *self.diffuse_color.as_vector() * *tex_color.as_vector();

                        let diffuse = diffuse_color * dcoeff;

                        // specular
                        let lproj = *normal * dot_ldir_norm;
                        let rldir = na::normalize(&(-ldir + lproj * na::cast::<f32, Scalar>(2.0)));

                        let scoeff: f32 = na::cast(-na::dot(&rldir, &ray.ray.dir));
                        if scoeff > na::zero() {
                            let scoeff   = scoeff.clone().powf(self.shininess);
                            let specular = self.specular_color * scoeff;

                            acc = acc + *light.color.as_vector() * filter * (diffuse + *specular.as_vector());
                        }
                        else {
                            acc = acc + *light.color.as_vector() * filter * diffuse;
                        }
                    }
                }
            });

            res.axpy(&( 1.0 / (light.racsample * light.racsample) as f32), &acc);
        }

        Point4::new(res.x, res.y, res.z, alpha)
    }
}
