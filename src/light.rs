use std::rand;
use nalgebra::na::Vec3;
use nalgebra::na::Indexable;
use nalgebra::na;
use ncollide::math::{Scalar, Vect};

pub struct Light {
    pub pos:       Vect,
    pub radius:    Scalar,
    pub racsample: uint,
    pub color:     Vec3<f32>
}

impl Light {
    pub fn new(pos: Vect, radius: Scalar, nsample: uint, color: Vec3<f32>) -> Light {
        Light {
            pos:     pos,
            radius:  radius,
            racsample: ((nsample as f32).sqrt()) as uint,
            color:   color
        }
    }
}

#[dim3]
impl Light {
    pub fn sample<T>(&self, f: |Vect| -> T) {
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

                v.set(0, self.radius * cphi * stheta);
                v.set(1, self.radius * sphi * stheta);
                v.set(2, self.radius * ctheta);

                f(v + self.pos);
            }
        }
    }
}

#[not_dim3]
impl Light {
    pub fn sample<T>(&self, f: |Vect| -> T) {
        for _ in range(0u, self.racsample * self.racsample) {
            f(rand::random::<Vect>() * self.radius + self.pos);
        }
    }
}
