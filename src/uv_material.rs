use nalgebra::na::{Vec3, Vec4};
use nalgebra::na;
use ncollide::math::N;
use material::Material;

pub struct UVMaterial;

impl Material for UVMaterial {
    #[inline]
    fn ambiant(&self, _: &Vec3<N>, _: &Vec3<N>, uv: &Option<Vec3<N>>) -> Vec4<f32> {
        match *uv {
            Some(ref uvs) => {
                let ux = NumCast::from(uvs.x).expect("Conversion failed.");
                let uy = NumCast::from(uvs.y).expect("Conversion failed.");
                let uz = NumCast::from(uvs.z).expect("Conversion failed.");

                Vec4::new(ux, uy, uz, 1.0)
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
