use std::rand;
use na::Pnt3;
use ncollide::math::{Scalar, Point, Vect};

#[cfg(feature = "3d")]
use na;

pub struct Light {
    pub pos:       Point,
    pub radius:    Scalar,
    pub racsample: uint,
    pub color:     Pnt3<f32>
}

impl Light {
    pub fn new(pos: Point, radius: Scalar, nsample: uint, color: Pnt3<f32>) -> Light {
        Light {
            pos:     pos,
            radius:  radius,
            racsample: ((nsample as f32).sqrt()) as uint,
            color:   color
        }
    }
}

#[cfg(feature = "3d")]
impl Light {
    pub fn sample<T>(&self, f: |Point| -> T) {
        for i in range(0u, self.racsample) {
            for j in range(0u, self.racsample) {
                let iracsample = na::one::<Scalar>() / na::cast(self.racsample);
                let parttheta  = iracsample * Float::pi();
                let partphi    = iracsample * Float::two_pi();

                let phi   = (rand::random::<Scalar>() + na::cast(i)) * partphi;
                let theta = (rand::random::<Scalar>() + na::cast(j)) * parttheta;

                let mut v = na::zero::<Vect>();

                let cphi   = phi.cos();
                let sphi   = phi.sin();
                let ctheta = theta.cos();
                let stheta = theta.sin();

                v[0] = self.radius * cphi * stheta;
                v[1] = self.radius * sphi * stheta;
                v[2] = self.radius * ctheta;

                f(self.pos + v);
            }
        }
    }
}

#[cfg(not(feature = "3d"))]
impl Light {
    pub fn sample<T>(&self, f: |Point| -> T) {
        for _ in range(0u, self.racsample * self.racsample) {
            f(self.pos + rand::random::<Vect>() * self.radius);
        }
    }
}
