use nalgebra::na::{Cast, VecExt, AlgebraicVecExt, AbsoluteRotate, Dim, Transform, Rotate,
                   Translation, Vec4, Vec3};
use nalgebra::na;
use ncollide::ray::Ray;
use ray_with_energy::RayWithEnergy;
use scene::Scene;
use material::Material;

pub struct PhongMaterial {
    diffuse_color:  Vec3<f32>,
    ambiant_color:  Vec3<f32>,
    diffuse_intensity:  f32,
    specular_intensity: f32,
    ambiant_intensity:  f32,
    alpha:              f32 // FIXME: rename that
}

impl PhongMaterial {
    pub fn new(diffuse_color:      Vec3<f32>,
               ambiant_color:      Vec3<f32>,
               diffuse_intensity:  f32,
               specular_intensity: f32,
               ambiant_intensity:  f32,
               alpha:              f32)
               -> PhongMaterial {
        PhongMaterial {
            diffuse_color:      diffuse_color,
            ambiant_color:      ambiant_color,
            diffuse_intensity:  diffuse_intensity,
            specular_intensity: specular_intensity,
            ambiant_intensity:  ambiant_intensity,
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
               scene:  &Scene<N, V, Vless, M>) -> Vec4<f32> {
        // initialize with the ambiant color
        let mut res = self.ambiant_color * self.ambiant_intensity;

        // compute the contribution of each light
        for light in scene.lights().iter() {
            let ldir = na::normalize(&(light.pos - *point));

            if !scene.intersects_ray(&Ray::new(point + ldir * na::cast(0.001), ldir.clone())) {
                let dot_ldir_norm = na::dot(&ldir, normal);

                // diffuse
                let dcoeff: f32 = NumCast::from(dot_ldir_norm.clone()).unwrap();
                let dcoeff = dcoeff.max(&0.0);
                let diffuse = (light.color * self.diffuse_color) * self.diffuse_intensity * dcoeff;

                // specular
                let lproj = normal * dot_ldir_norm;
                let rldir = na::normalize(&(-ldir + lproj * na::cast(2.0)));

                let scoeff: f32 = NumCast::from(-na::dot(&rldir, &ray.ray.dir)).unwrap();
                if scoeff > na::zero() {
                    let scoeff   = scoeff.pow(&self.alpha);
                    let specular = light.color * self.specular_intensity * scoeff;
                    res = res + diffuse + specular;
                }
                else {
                    res = res + diffuse;
                }
            }
        }

        na::to_homogeneous(&res)
    }
}
