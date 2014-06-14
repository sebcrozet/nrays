use std::io::Writer;
use nalgebra::na::Vec3;
use nalgebra::na;
use ncollide::math::Scalar;

#[cfg(dim3)]
use png;
#[cfg(dim3)]
use nalgebra::na::Vec2;

#[cfg(dim3)]
pub type Vless = Vec2<Scalar>;

#[cfg(dim4)]
pub type Vless = Vec3<Scalar>;

pub struct Image {
    extents: Vless, // extents of the rendering cube
    pixels:  Vec<Vec3<f32>>
}

impl Image {
    pub fn new(extents: Vless, pixels: Vec<Vec3<f32>>) -> Image {
        Image {
            extents: extents,
            pixels:  pixels
        }
    }
}

#[cfg(dim3)]
impl Image {
    pub fn to_ppm<W: Writer>(&self, w: &mut W) {
        // XXX: there is something weird here…
        let width  = self.extents.x as uint;
        let height = self.extents.y as uint;

        let _ = w.write("P3\n".as_bytes());

        let _ = w.write_uint(width);
        let _ = w.write(" ".as_bytes());
        let _ = w.write_uint(height);
        let _ = w.write("\n".as_bytes());
        let _ = w.write("255\n".as_bytes());

        for i in range(0u, height) {
            for j in range(0u, width) {
                let c:     Vec3<f32> = self.pixels.get(i * width + j).clone();
                let color: Vec3<f32> = na::cast(c * 255.0f32);
                let white            = Vec3::new(255.0, 255.0, 255.0);
                let valid_color      = na::inf(&na::sup(&white, &color), &white);
                let px: Vec3<uint>   = na::cast(valid_color);

                let _ = w.write_uint(px.x);
                let _ = w.write(" ".as_bytes());
                let _ = w.write_uint(px.y);
                let _ = w.write(" ".as_bytes());
                let _ = w.write_uint(px.z);
                let _ = w.write(" ".as_bytes());
            }

            let _ = w.write("\n".as_bytes());
        }
    }

    pub fn to_png(&self, path: &Path) {
        let width  = self.extents.x as uint;
        let height = self.extents.y as uint;

        let mut data: Vec<u8> = Vec::new();
        for i in range(0u, height) {
            for j in range(0u, width) {
                let c:     Vec3<f32> = self.pixels.get(i * width + j).clone();
                let color: Vec3<f32> = na::cast(c * 255.0f32);
                let white            = Vec3::new(255.0, 255.0, 255.0);
                let valid_color      = na::inf(&na::sup(&color, &na::zero()), &white);
                let px: Vec3<uint>   = na::cast(valid_color);

                data.push(px.x as u8);
                data.push(px.y as u8);
                data.push(px.z as u8);
            }
        }

        let img = png::Image {
            width:      width  as u32,
            height:     height as u32,
            color_type: png::RGB8,
            pixels:     data
        };

        let res = png::store_png(&img, path);

        if !res.is_ok() {
            fail!("Failed to save the output image.")
        }
    }
}

#[cfg(dim4)]
impl Image {
    pub fn to_file<W: Writer>(&self, w: &mut W) {
        let wx = self.extents.x as uint;
        let wy = self.extents.y as uint;
        let wz = self.extents.z as uint;

        for x in range(0u, wx) {
            for y in range(0u, wy) {
                for z in range(0u, wz) {
                    let c:     Vec3<f32> = self.pixels.get(z * wx * wy + y * wx + x).clone();
                    let color: Vec3<f32> = na::cast(c * 255.0f32);
                    let white            = Vec3::new(255.0, 255.0, 255.0);
                    let valid_color      = na::inf(&na::sup(&color, &na::zero()), &white);
                    let _: Vec3<uint>    = na::cast(valid_color);

                    let _ = w.write_le_f32((c.x + c.y + c.z) / 3.0f32);
                    // w.write_le_uint(px.y);
                    // w.write_le_uint(px.z);
                }
            }
        }
    }
}
