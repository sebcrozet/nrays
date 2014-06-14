use nalgebra::na::{Vec2, Vec3, Vec4};
use nalgebra::na;
use ncollide::math::Scalar;
use material::Material;

pub struct UVMaterial;

impl Material for UVMaterial {
    #[inline]
    fn ambiant(&self, _: &Vec3<Scalar>, _: &Vec3<Scalar>, uv: &Option<Vec2<Scalar>>) -> Vec4<f32> {
        match *uv {
            Some(ref uvs) => {
                let ux = NumCast::from(uvs.x).expect("Conversion failed.");
                let uy = NumCast::from(uvs.y).expect("Conversion failed.");

                Vec4::new(ux, uy, na::zero(), 1.0)
            },
            None => na::zero()
        }
    }
}

impl UVMaterial {
    #[inline]
    pub fn new() -> UVMaterial {
        UVMaterial
    }
}
