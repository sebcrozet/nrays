use na::{Pnt2, Pnt4};
use math::{Scalar, Point, Vect};
use material::Material;

pub struct NormalMaterial;

impl Material for NormalMaterial {
    #[inline]
    fn ambiant(&self, _: &Point, normal: &Vect, _: &Option<Pnt2<Scalar>>) -> Pnt4<f32> {
        Pnt4::new((1.0f32 + NumCast::from(normal.x.clone()).expect("Conversion failed.")) / 2.0,
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
