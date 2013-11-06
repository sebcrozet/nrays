use nalgebra::na::{Indexable, Vec2, Vec3, Vec4};
use nalgebra::na;
use std::rt::io::Writer;

pub struct Image<Vi> {
    priv extents: Vi, // extents of the rendering cube
    priv pixels:  ~[Vec4<f64>]
}

impl<Vi> Image<Vi> {
    pub fn new(extents: Vi, pixels: ~[Vec4<f64>]) -> Image<Vi> {
        Image {
            extents: extents,
            pixels:  pixels
        }
    }
}

impl Image<Vec2<uint>> {
    pub fn to_ppm<W: Writer>(&self, w: &mut W) {
        let width  = self.extents.at(1);
        let height = self.extents.at(0);

        w.write("P3\n".as_bytes());

        let s = width.to_str() + " " + height.to_str() + "\n";
        w.write(s.as_bytes());
        w.write("255\n".as_bytes());

        for i in range(0u, height) {
            for j in range(0u, width) {
                let h_c              = &self.pixels[i * width + j];
                let c:     Vec3<f64> = na::from_homogeneous(h_c);
                let color: Vec3<f64> = na::cast(c * 255.0);
                let white            = Vec3::new(255.0, 255.0, 255.0);
                let valid_color      = color.clamp(&na::zero(), &white);
                let px: Vec3<uint> = na::cast(valid_color);

                let s = px.x.to_str() + " " + px.y.to_str() + " " + px.z.to_str() + " ";
                w.write(s.as_bytes());
            }

            w.write("\n".as_bytes());
        }
    }
}
