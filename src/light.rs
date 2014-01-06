use std::rand;
use nalgebra::na::Vec3;
use ncollide::math::{N, V};

pub struct Light {
    pos:     V,
    radius:  N,
    nsample: uint,
    color:   Vec3<f32>
}

impl Light {
    pub fn new(pos: V, radius: N, nsample: uint, color: Vec3<f32>) -> Light {
        Light {
            pos:     pos,
            radius:  radius,
            nsample: nsample,
            color:   color
        }
    }

    pub fn sample<T>(&self, n: uint, f: |V| -> T) {
        for _ in range(0u, n) {
            f(rand::random::<V>() * self.radius + self.pos);
        }
    }
}
