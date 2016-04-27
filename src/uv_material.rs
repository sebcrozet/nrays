use na::{Point2, Point3, Point4, Vector3};
use na;
use math::Scalar;
use material::Material;

pub struct UVMaterial;

impl Material for UVMaterial {
    #[inline]
    fn ambiant(&self, _: &Point3<Scalar>, _: &Vector3<Scalar>, uv: &Option<Point2<Scalar>>) -> Point4<f32> {
        match *uv {
            Some(ref uvs) => {
                let ux = na::cast(uvs.x);
                let uy = na::cast(uvs.y);

                Point4::new(ux, uy, na::zero(), 1.0)
            },
            None => na::origin()
        }
    }
}

impl UVMaterial {
    #[inline]
    pub fn new() -> UVMaterial {
        UVMaterial
    }
}
