use nalgebra::vec::Vec4;

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
