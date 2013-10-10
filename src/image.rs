use nalgebra::na::{Indexable, Vec2, Vec3, Vec4};
use nalgebra::na;
use std::io::Writer;

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
    pub fn to_ppm(&self, w: @Writer) {
        let width  = self.extents.at(1);
        let height = self.extents.at(0);

        w.write_str("P3\n");
        w.write_str(width.to_str() + " " + height.to_str() + "\n");
        w.write_str("255\n");

        for i in range(0u, height) {
            for j in range(0u, width) {
                let h_c            = &self.pixels[i * width + j];
                let c:  Vec3<f64>  = na::from_homogeneous(h_c);
                let px: Vec3<uint> = na::cast(c * 255.0);

                w.write_str(px.x.to_str() + " " + px.y.to_str() + " " + px.z.to_str() + " ");
            }

            w.write_char('\n');
        }
    }
}
