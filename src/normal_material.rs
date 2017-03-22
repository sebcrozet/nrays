use na::{Point2, Point4};
use math::{Scalar, Point, Vect};
use material::Material;

pub struct NormalMaterial;

impl Material for NormalMaterial {
    #[inline]
    fn ambiant(&self, _: &Point, normal: &Vect, _: &Option<Point2<Scalar>>) -> Point4<f32> {
        Point4::new((1.0f32 + normal.x as f32) / 2.0,
                    (1.0f32 + normal.y as f32) / 2.0,
                    (1.0f32 + normal.z as f32) / 2.0,
                    1.0)
    }
}

impl NormalMaterial {
    #[inline]
    pub fn new() -> NormalMaterial {
        NormalMaterial
    }
}
