use nalgebra::na::Vec3;
use nalgebra::na;
use ncollide::math::N;
use light_path::LightPath;
use ray_with_energy::RayWithEnergy;
use scene::Scene;
use material::Material;

pub struct UVMaterial;

impl Material for UVMaterial {
    #[inline]
    fn compute(&self,
               _:  &RayWithEnergy,
               _:  &Vec3<N>,
               _:  &Vec3<N>,
               uv: &Option<(N, N, N)>,
               _:  &Scene)
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

    fn compute_for_light_path(&self,
               path:  &mut LightPath,
               _:  &Vec3<N>,
               _:  &Vec3<N>,
               uv: &Option<(N, N, N)>,
               _:  &Scene) {
        match *uv {
            Some(ref uvs) => {
                let (ux, uy, uz) = uvs.clone();
                let ux = NumCast::from(ux).expect("Conversion failed.");
                let uy = NumCast::from(uy).expect("Conversion failed.");
                let uz = NumCast::from(uz).expect("Conversion failed.");

                path.color = path.color * (1.0f32 - path.mix_coef) + Vec3::new(ux, uy, uz) *
                             path.mix_coef;
            },
            None => ()
        };
    }
}

impl UVMaterial {
    #[inline]
    pub fn new() -> UVMaterial {
        UVMaterial
    }
}
