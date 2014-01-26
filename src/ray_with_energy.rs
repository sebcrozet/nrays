use nalgebra::na;
use ncollide::ray::Ray;
use ncollide::math::{N, V};

pub struct RayWithEnergy {
    ray:    Ray,
    refr:   N,
    energy: f32
}

impl RayWithEnergy {
    pub fn new(orig: V, dir: V) -> RayWithEnergy {
        RayWithEnergy::new_with_energy(orig, dir, na::cast(1.0), 1.0)
    }

    pub fn new_with_energy(orig: V, dir: V, refr: N, energy: f32) -> RayWithEnergy {
        RayWithEnergy {
            ray:    Ray::new(orig, dir),
            refr:   refr,
            energy: energy
        }
    }
}
