use na;
use ncollide::ray::Ray;
use ncollide::math::{Scalar, Point, Vect};

pub struct RayWithEnergy {
    pub ray:    Ray,
    pub refr:   Scalar,
    pub energy: f32
}

impl RayWithEnergy {
    pub fn new(orig: Point, dir: Vect) -> RayWithEnergy {
        RayWithEnergy::new_with_energy(orig, dir, na::cast(1.0f64), 1.0)
    }

    pub fn new_with_energy(orig: Point, dir: Vect, refr: Scalar, energy: f32) -> RayWithEnergy {
        RayWithEnergy {
            ray:    Ray::new(orig, dir),
            refr:   refr,
            energy: energy
        }
    }
}
