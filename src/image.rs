use std::io::Writer;
use std::num::Zero;
use nalgebra::na::{Indexable, Vec2, Vec3, Vec4};
use nalgebra::na;

pub struct Image<V> {
    priv extents: V, // extents of the rendering cube
    priv pixels:  ~[Vec4<f32>]
}

impl<V> Image<V> {
    pub fn new(extents: V, pixels: ~[Vec4<f32>]) -> Image<V> {
        Image {
            extents: extents,
            pixels:  pixels
        }
    }
}

impl Image<Vec2<f64>> {
    pub fn to_ppm<W: Writer>(&self, w: &mut W) {
        // XXX: there is something weird here…
        let width  = self.extents.at(0) as uint;
        let height = self.extents.at(1) as uint;

        w.write("P3\n".as_bytes());

        w.write_uint(width);
        w.write(" ".as_bytes());
        w.write_uint(height);
        w.write("\n".as_bytes());
        w.write("255\n".as_bytes());

        for i in range(0u, height) {
            for j in range(0u, width) {
                let h_c              = &self.pixels[i * width + j];
                let c:     Vec3<f32> = na::from_homogeneous(h_c);
                let color: Vec3<f32> = na::cast(c * 255.0f32);
                let white            = Vec3::new(255.0, 255.0, 255.0);
                let valid_color      = color.clamp(&na::zero(), &white);
                let px: Vec3<uint>   = na::cast(valid_color);

                w.write_uint(px.x);
                w.write(" ".as_bytes());
                w.write_uint(px.y);
                w.write(" ".as_bytes());
                w.write_uint(px.z);
                w.write(" ".as_bytes());
            }

            w.write("\n".as_bytes());
        }
    }
}

impl Image<Vec3<f64>> {
    pub fn to_file<W: Writer>(&self, w: &mut W) {
        let wx = self.extents.at(0) as uint;
        let wy = self.extents.at(1) as uint;
        let wz = self.extents.at(2) as uint;

        for x in range(0u, wx) {
            for y in range(0u, wy) {
                for z in range(0u, wz) {
                    let h_c = &self.pixels[z * wx * wy + y * wx + x];
                    let c:     Vec3<f32> = na::from_homogeneous(h_c);
                    let color: Vec3<f32> = na::cast(c * 255.0f32);
                    let white            = Vec3::new(255.0, 255.0, 255.0);
                    let valid_color      = color.clamp(&na::zero(), &white);
                    let px: Vec3<uint>   = na::cast(valid_color);

                    w.write_le_f32((c.x + c.y + c.z) / 3.0f32);
                    // w.write_le_uint(px.y);
                    // w.write_le_uint(px.z);
                }
            }
        }
    }
}
