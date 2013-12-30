use std::num::{One, Zero};
use nalgebra::na::Vec3;
use ray_with_energy::RayWithEnergy;
use scene::Scene;
use material::Material;

pub struct NormalMaterial;

impl<N: Clone + One + Zero + ToPrimitive, Vlessi, M> Material<N, Vec3<N>, Vlessi, M>
for NormalMaterial {
    #[inline]
    fn compute(&self,
               _:      &RayWithEnergy<Vec3<N>>,
               _:      &Vec3<N>,
               normal: &Vec3<N>,
               _:      &Option<(N, N, N)>,
               _:      &Scene<N, Vec3<N>, Vlessi, M>)
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
