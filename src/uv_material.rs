use std::num::{One, Zero};
use nalgebra::na::Vec3;
use nalgebra::na;
use ray_with_energy::RayWithEnergy;
use scene::Scene;
use material::Material;

pub struct UVMaterial;

impl<N: Clone + One + Zero + ToPrimitive, Vlessi, M> Material<N, Vec3<N>, Vlessi, M>
for UVMaterial {
    #[inline]
    fn compute(&self,
               _:  &RayWithEnergy<Vec3<N>>,
               _:  &Vec3<N>,
               _:  &Vec3<N>,
               uv: &Option<(N, N, N)>,
               _:  &Scene<N, Vec3<N>, Vlessi, M>)
               -> Vec3<f32> {
        match *uv {
            Some(ref uvs) => {
                let (ux, uy, uz) = uvs.clone();
                let ux = NumCast::from(ux).expect("Conversion failed.");
                let uy = NumCast::from(uy).expect("Conversion failed.");
                let uz = NumCast::from(uz).expect("Conversion failed.");

                Vec3::new(ux, uy, uz)
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
