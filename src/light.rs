use std::rand;
use nalgebra::na::Vec3;
use ncollide::math::{N, V};

#[cfg(dim3)]
use nalgebra::na::Indexable;
#[cfg(dim3)]
use nalgebra::na;

pub struct Light {
    pos:       V,
    radius:    N,
    racsample: uint,
    color:     Vec3<f32>
}

impl Light {
    pub fn new(pos: V, radius: N, nsample: uint, color: Vec3<f32>) -> Light {
        Light {
            pos:     pos,
            radius:  radius,
            racsample: ((nsample as f32).sqrt()) as uint,
            color:   color
        }
    }

    #[cfg(dim3)]
    pub fn sample<T>(&self, f: |V| -> T) {
        for i in range(0u, self.racsample) {
            for j in range(0u, self.racsample) {
                let iracsample = na::one::<N>() / na::cast(self.racsample);
                let parttheta  = iracsample * Real::pi();
                let partphi    = iracsample * Real::two_pi();

                let phi   = (rand::random::<N>() + na::cast(i)) * partphi;
                let theta = (rand::random::<N>() + na::cast(j)) * parttheta;

                let mut v = na::zero::<V>();

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

    #[cfg(not(dim3))]
    pub fn sample<T>(&self, f: |V| -> T) {
        for _ in range(0u, self.racsample * self.racsample) {
            f(rand::random::<V>() * self.radius + self.pos);
        }
    }
}
