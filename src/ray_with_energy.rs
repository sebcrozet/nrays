use math::{Point, Scalar, Vect};
use ncollide3d::query::Ray;

pub struct RayWithEnergy {
    pub ray: Ray<Scalar>,
    pub refr: Scalar,
    pub energy: f32,
}

impl RayWithEnergy {
    pub fn new(orig: Point, dir: Vect) -> RayWithEnergy {
        RayWithEnergy::new_with_energy(orig, dir, 1.0, 1.0)
    }

    pub fn new_with_energy(orig: Point, dir: Vect, refr: Scalar, energy: f32) -> RayWithEnergy {
        RayWithEnergy {
            ray: Ray::new(orig, dir),
            refr: refr,
            energy: energy,
        }
    }
}
