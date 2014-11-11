use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use stb_image::image::ImageU8;
use stb_image::image;
use na::{Pnt2, Pnt4, Vec2};
use na;
use math::Scalar;

pub struct ImageData {
    pixels: Vec<Pnt4<f32>>,
    dims:   Vec2<uint>
}

impl ImageData {
    pub fn new(pixels: Vec<Pnt4<f32>>, dims: Vec2<uint>) -> ImageData {
        assert!(pixels.len() == dims.x * dims.y);
        assert!(dims.x >= 1);
        assert!(dims.y >= 1);

        ImageData {
            pixels: pixels,
            dims:   dims
        }
    }
}

local_data_key!(KEY_TEXTURE_MANAGER: RefCell<TextureManager>)

struct TextureManager {
    loaded_opaque:      HashMap<String, Arc<ImageData>>,
    loaded_transparent: HashMap<String, Arc<ImageData>>
}

impl TextureManager {
    pub fn new() -> TextureManager {
        TextureManager {
            loaded_opaque:      HashMap::new(),
            loaded_transparent: HashMap::new()
        }
    }
}

/// Gets the texture manager.
fn get_texture_manager<T>(f: |&mut TextureManager| -> T) -> T {
    if KEY_TEXTURE_MANAGER.get().is_none() {
        let _ = KEY_TEXTURE_MANAGER.replace(Some(RefCell::new(TextureManager::new())));
    }

    f(KEY_TEXTURE_MANAGER.get().unwrap().borrow_mut().deref_mut())
}

// FIXME: move this to its own file
pub enum Interpolation {
    Bilinear,
    Nearest
}

pub enum Overflow {
    ClampToEdges,
    Wrap
}

pub struct Texture2d {
    data:     Arc<ImageData>,
    interpol: Interpolation,
    overflow: Overflow
}

impl Texture2d {
    pub fn new(data:          Arc<ImageData>,
               interpolation: Interpolation,
               overflow:      Overflow)
               -> Texture2d {
        Texture2d {
            data:     data,
            interpol: interpolation,
            overflow: overflow 
        }
    }

    pub fn from_png(path: &Path, opacity: bool, interpolation: Interpolation, overflow: Overflow) -> Option<Texture2d> {
        let data = get_texture_manager(|tm| {
            let res;
            
            {
                let found;

                if opacity {
                   found = tm.loaded_transparent.get(&path.as_str().unwrap().to_string());
                }
                else {
                   found = tm.loaded_opaque.get(&path.as_str().unwrap().to_string());
                }

                res = match found {
                    Some(data) => Some(data.clone()),
                    None => {
                        match image::load(&Path::new(path.as_str().unwrap().to_string())) {
                            ImageU8(mut image) => {
                                let mut data = Vec::new();

                                // Flip the y axis
                                let elt_per_row = image.width * image.depth;
                                for j in range(0u, image.height / 2) {
                                    for i in range(0u, elt_per_row) {
                                        image.data.as_mut_slice().swap(
                                            (image.height - j - 1) * elt_per_row + i,
                                            j * elt_per_row + i)
                                    }
                                }

                                if image.depth == 1 {
                                    for p in image.data.iter() {
                                        let g = *p as f32 / 255.0;

                                        if opacity {
                                            data.push(Pnt4::new(1.0, 1.0, 1.0, g));
                                        }
                                        else {
                                            data.push(Pnt4::new(g, g, g, 1.0));
                                        }
                                    }

                                    Some(Arc::new(ImageData::new(data,
                                    Vec2::new(image.width as uint, image.height as uint))))
                                }
                                else if image.depth == 2 {
                                    for p in image.data.as_slice().chunks(2) {
                                        let r = p[0] as f32 / 255.0;
                                        let g = p[1] as f32 / 255.0;

                                        if opacity {
                                            data.push(Pnt4::new(1.0, 1.0, 1.0, g * r));
                                        }
                                        else {
                                            data.push(Pnt4::new(r * g, r * g, r * g, 1.0));
                                        }
                                    }

                                    Some(Arc::new(ImageData::new(data,
                                    Vec2::new(image.width as uint, image.height as uint))))
                                }
                                else if image.depth == 3 {
                                    for p in image.data.as_slice().chunks(3) {
                                        let r = p[0] as f32 / 255.0;
                                        let g = p[1] as f32 / 255.0;
                                        let b = p[2] as f32 / 255.0;

                                        if opacity {
                                            data.push(Pnt4::new(1.0, 1.0, 1.0, r));
                                        }
                                        else {
                                            data.push(Pnt4::new(r, g, b, 1.0));
                                        }
                                    }

                                    Some(Arc::new(ImageData::new(data,
                                    Vec2::new(image.width as uint, image.height as uint))))
                                }
                                else if image.depth == 4 {
                                    for p in image.data.as_slice().chunks(4) {
                                        let r = p[0] as f32 / 255.0;
                                        let g = p[1] as f32 / 255.0;
                                        let b = p[2] as f32 / 255.0;
                                        let a = p[3] as f32 / 255.0;

                                        if opacity {
                                            data.push(Pnt4::new(1.0, 1.0, 1.0, a));
                                        }
                                        else {
                                            data.push(Pnt4::new(r, g, b, 1.0));
                                        }
                                    }

                                    Some(Arc::new(ImageData::new(data,
                                    Vec2::new(image.width as uint, image.height as uint))))
                                }
                                else {
                                    panic!("Image depth {} not suported.", image.depth);
                                }
                            },
                            _ => {
                                None
                            }
                        }
                    }
                };
            }

            let data = res.clone();
            data.map(|data| {
                if opacity {
                    tm.loaded_transparent.insert(path.as_str().unwrap().to_string(), data)
                }
                else {
                    tm.loaded_opaque.insert(path.as_str().unwrap().to_string(), data)
                }
            });

            res
        });

        data.map(|data| Texture2d::new(data, interpolation, overflow))
    }

    pub fn at<'a>(&'a self, x: uint, y: uint) -> &'a Pnt4<f32> {
        &self.data.pixels[y * self.data.dims.x + x]
    }

    pub fn sample(&self, coords: &Pnt2<Scalar>) -> Pnt4<f32> {
        let mut ux: f32 = NumCast::from(coords.x).expect("Conversion of sampling coordinates failed.");
        let mut uy: f32 = NumCast::from(coords.y).expect("Conversion of sampling coordinates failed.");

        match self.overflow {
            ClampToEdges => {
                ux = na::clamp(ux, 0.0, 1.0);
                uy = na::clamp(uy, 0.0, 1.0);
            }
            Wrap => {
                ux = ux % 1.0;
                uy = uy % 1.0;

                if ux < 0.0 { ux = 1.0 + ux };
                if uy < 0.0 { uy = 1.0 + uy };
            }
        }

        ux = ux * ((self.data.dims.x - 1) as f32);
        uy = uy * ((self.data.dims.y - 1) as f32);

        match self.interpol {
            Nearest => {
                let ux = ux.round() as uint;
                let uy = uy.round() as uint;

                self.at(ux, uy).clone()
            },
            Bilinear => {
                let low_ux = ux.floor() as uint;
                let low_uy = uy.floor() as uint;

                let hig_ux = low_ux + 1;
                let hig_uy = low_uy + 1;

                let shift_ux = ux - (low_ux as f32);
                let shift_uy = uy - (low_uy as f32);

                let ul = self.at(low_ux, hig_uy);
                let ur = self.at(hig_ux, hig_uy);
                let dr = self.at(hig_ux, low_uy);
                let dl = self.at(low_ux, low_uy);

                let u_interpol = *ul * (1.0 - shift_ux) + *ur.as_vec() * shift_ux;
                let d_interpol = *dl * (1.0 - shift_ux) + *dr.as_vec() * shift_ux;

                u_interpol * shift_uy + *d_interpol.as_vec() * (1.0 - shift_uy)
            }
        }
    }
}
