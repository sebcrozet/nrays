use rand::random;
use na::Point3;
use math::{Scalar, Point, Vect};

#[cfg(feature = "3d")]
use na::{self, BaseFloat};

pub struct Light {
    pub pos:       Point,
    pub radius:    Scalar,
    pub racsample: usize,
    pub color:     Point3<f32>
}

impl Light {
    pub fn new(pos: Point, radius: Scalar, nsample: usize, color: Point3<f32>) -> Light {
        Light {
            pos:     pos,
            radius:  radius,
            racsample: ((nsample as f32).sqrt()) as usize,
            color:   color
        }
    }
}

#[cfg(feature = "3d")]
impl Light {
    pub fn sample<T, F: FnMut(Point) -> T>(&self, f: &mut F) {
        for i in 0usize .. self.racsample {
            for j in 0usize .. self.racsample {
                let iracsample: Scalar = 1.0 / (self.racsample as f64);
                let pi: Scalar         = BaseFloat::pi();
                let parttheta: Scalar  = iracsample * pi;
                let partphi: Scalar    = iracsample * (pi + pi);

                let phi: Scalar   = (random::<f64>() + i as f64) * partphi;
                let theta: Scalar = (random::<f64>() + j as f64) * parttheta;

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
    pub fn sample<T, F: FnMut(Point) -> T>(&self, f: &mut F) {
        for _ in 0 .. self.racsample * self.racsample {
            f(self.pos + random::<Vect>() * self.radius);
        }
    }
}
