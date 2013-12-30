use nalgebra::na::{Cast, VecExt, AlgebraicVecExt, AbsoluteRotate, Dim, Transform, Rotate,
                   Translation, Vec3};
use nalgebra::na;
use ncollide::ray::Ray;
use ray_with_energy::RayWithEnergy;
use scene::Scene;
use material::Material;
use texture2d::Texture2d;


pub struct PhongMaterial {
    diffuse_color:      Vec3<f32>,
    ambiant_color:      Vec3<f32>,
    texture:            Option<Texture2d>, // FIXME: put this on an ARC?
    ka:                 f32,
    kd:                 f32,
    ks:                 f32,
    alpha:              f32 // FIXME: rename that
}

impl PhongMaterial {
    pub fn new(ambiant_color:      Vec3<f32>,
               diffuse_color:      Vec3<f32>,
               ka:                 f32,
               kd:                 f32,
               ks:                 f32,
               texture:            Option<Texture2d>,
               alpha:              f32)
               -> PhongMaterial {
        PhongMaterial {
            diffuse_color:      diffuse_color,
            ambiant_color:      ambiant_color,
            ka:                 ka,
            kd:                 kd,
            ks:                 ks,
            texture:            texture,
            alpha:              alpha
        }
    }
}

// FIXME: there might be too many bounds here…
impl<N:     'static + Cast<f32> + Send + Freeze + NumCast + Primitive + Algebraic + Signed + Float,
     V:     'static + AlgebraicVecExt<N> + Send + Freeze + Clone,
     Vless: VecExt<N> + Dim + Clone,
     M:     Translation<V> + Rotate<V> + Send + Freeze + Transform<V> + Mul<M, M> + AbsoluteRotate<V> + Dim>
Material<N, V, Vless, M> for PhongMaterial {
    fn compute(&self,
               ray:    &RayWithEnergy<V>,
               point:  &V,
               normal: &V,
               uvs:    &Option<(N, N, N)>,
               scene:  &Scene<N, V, Vless, M>)
               -> Vec3<f32> {
        // initialize with the ambiant color
        let mut res = self.ambiant_color * self.ka;

        // compute the contribution of each light
        for light in scene.lights().iter() {
            let mut ldir = light.pos - *point;
            let     dist = ldir.normalize() - na::cast(0.001);

            if !scene.intersects_ray(&Ray::new(point + ldir * na::cast(0.001), ldir.clone()), dist) {
                let dot_ldir_norm = na::dot(&ldir, normal);

                // diffuse
                let dcoeff: f32   = NumCast::from(dot_ldir_norm.clone()).expect("Conversion failed.");
                let dcoeff        = dcoeff.max(&0.0);
                let diffuse_color;

                if na::dim::<V>() == 3 && uvs.is_some() && self.texture.is_some() {
                    let uvs       = uvs.as_ref().unwrap();
                    let tex       = self.texture.as_ref().unwrap();
                    let texture   = tex.sample(uvs);
                    diffuse_color = texture / 2.0f32
                }
                else {
                    diffuse_color = self.diffuse_color;
                }

                let diffuse = (light.color * diffuse_color) * dcoeff;

                // specular
                let lproj = normal * dot_ldir_norm;
                let rldir = na::normalize(&(-ldir + lproj * na::cast(2.0)));

                let scoeff: f32 = NumCast::from(-na::dot(&rldir, &ray.ray.dir)).expect("Conversion failed.");
                if scoeff > na::zero() {
                    let scoeff   = scoeff.pow(&self.alpha);
                    let specular = light.color * scoeff;

                    res = res + diffuse * self.kd + specular * self.ks;
                }
                else {
                    res = res + diffuse * self.kd;
                }
            }
        }

        res
    }
}
