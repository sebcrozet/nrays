use nalgebra::na::Vec3;
use ncollide::math::N;
use ray_with_energy::RayWithEnergy;
use scene::Scene;
use material::Material;

pub struct NormalMaterial;

impl Material for NormalMaterial {
    #[inline]
    fn compute(&self,
               _:      &RayWithEnergy,
               _:      &Vec3<N>,
               normal: &Vec3<N>,
               _:      &Option<Vec3<N>>,
               _:      &Scene)
               -> Vec3<f32> {
        Vec3::new((1.0f32 + NumCast::from(normal.x.clone()).expect("Conversion failed.")) / 2.0,
                  (1.0f32 + NumCast::from(normal.y.clone()).expect("Conversion failed.")) / 2.0,
                  (1.0f32 + NumCast::from(normal.z.clone()).expect("Conversion failed.")) / 2.0)
    }
}

impl NormalMaterial {
    #[inline]
    pub fn new() -> NormalMaterial {
        NormalMaterial
    }
}
