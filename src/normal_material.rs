use nalgebra::na::Vec3;
use ncollide::math::N;
use ray_with_energy::RayWithEnergy;
use light_path::LightPath;
use scene::Scene;
use material::Material;

pub struct NormalMaterial;

impl Material for NormalMaterial {
    #[inline]
    fn compute(&self,
               _:      &RayWithEnergy,
               _:      &Vec3<N>,
               normal: &Vec3<N>,
               _:      &Option<(N, N, N)>,
               _:      &Scene)
               -> Vec3<f32> {
        Vec3::new((1.0f32 + NumCast::from(normal.x.clone()).expect("Conversion failed.")) / 2.0,
                  (1.0f32 + NumCast::from(normal.y.clone()).expect("Conversion failed.")) / 2.0,
                  (1.0f32 + NumCast::from(normal.z.clone()).expect("Conversion failed.")) / 2.0)
    }

    #[inline]
    fn compute_for_light_path(&self,
               path:      &mut LightPath,
               _:      &Vec3<N>,
               normal: &Vec3<N>,
               _:      &Option<(N, N, N)>,
               _:      &Scene) {
        path.color = path.color * (1.0f32 - path.mix_coef) +
                     Vec3::new((1.0f32 + NumCast::from(normal.x.clone()).expect("Conversion failed.")) / 2.0,
                  (1.0f32 + NumCast::from(normal.y.clone()).expect("Conversion failed.")) / 2.0,
                  (1.0f32 + NumCast::from(normal.z.clone()).expect("Conversion failed.")) / 2.0) *
                     path.mix_coef;
    }
}

impl NormalMaterial {
    #[inline]
    pub fn new() -> NormalMaterial {
        NormalMaterial
    }
}
