use nalgebra::na::{Vec3, Vec4};
use ncollide::math::{N, V};
use material::Material;

pub struct NormalMaterial;

impl Material for NormalMaterial {
    #[inline]
    fn ambiant(&self, _: &V, normal: &V, _: &Option<Vec3<N>>) -> Vec4<f32> {
        Vec4::new((1.0f32 + NumCast::from(normal.x.clone()).expect("Conversion failed.")) / 2.0,
                  (1.0f32 + NumCast::from(normal.y.clone()).expect("Conversion failed.")) / 2.0,
                  (1.0f32 + NumCast::from(normal.z.clone()).expect("Conversion failed.")) / 2.0,
                  1.0)
    }
}

impl NormalMaterial {
    #[inline]
    pub fn new() -> NormalMaterial {
        NormalMaterial
    }
}
