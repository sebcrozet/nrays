use std::num::{One, Zero};
use nalgebra::na::{Vec3, Vec4};
use scene::Scene;
use material::Material;

pub struct NormalMaterial;

impl<N: Clone + One + Zero + ToPrimitive, Vlessi, M> Material<N, Vec3<N>, Vlessi, M>
for NormalMaterial {
    fn compute(&self,
               _:      &Vec3<N>,
               normal: &Vec3<N>,
               _:      &Scene<N, Vec3<N>, Vlessi, M>)
               -> Vec4<f32> {
        Vec4::new((1.0f32 + NumCast::from(normal.x.clone()).expect("(n) Conversion failed.")) / 2.0,
                  (1.0f32 + NumCast::from(normal.y.clone()).expect("(n) Conversion failed.")) / 2.0,
                  (1.0f32 + NumCast::from(normal.z.clone()).expect("(n) Conversion failed.")) / 2.0,
                  1.0)
    }
}

impl NormalMaterial {
    pub fn new() -> NormalMaterial {
        NormalMaterial
    }
}
