use ncollide::ray::Ray;
use ncollide::math::V;

pub struct RayWithEnergy {
    ray:    Ray,
    energy: f32
}

impl RayWithEnergy {
    pub fn new(orig: V, dir: V) -> RayWithEnergy {
        RayWithEnergy::new_with_energy(orig, dir, 1.0)
    }

    pub fn new_with_energy(orig: V, dir: V, energy: f32) -> RayWithEnergy {
        RayWithEnergy {
            ray:    Ray::new(orig, dir),
            energy: energy
        }
    }
}
