use std::rc::Rc;
use std::hashmap::HashMap;
use std::local_data;
use png;
use nalgebra::na::{Vec3, Vec2};
use ncollide::math::N;

struct ImageData {
    pixels: ~[Vec3<f32>],
    dims:   Vec2<uint>
}

impl ImageData {
    pub fn new(pixels: ~[Vec3<f32>], dims: Vec2<uint>) -> ImageData {
        assert!(pixels.len() == dims.x * dims.y);
        assert!(dims.x >= 1);
        assert!(dims.y >= 1);

        ImageData {
            pixels: pixels,
            dims:   dims
        }
    }
}

local_data_key!(KEY_TEXTURE_MANAGER: TexturesManager)

struct TexturesManager {
    loaded: HashMap<~str, Rc<ImageData>>
}

impl TexturesManager {
    pub fn new() -> TexturesManager {
        TexturesManager {
            loaded: HashMap::new()
        }
    }
}

/// Gets the texture manager.
pub fn get_texture_manager<T>(f: |&mut TexturesManager| -> T) -> T {
    if local_data::get(KEY_TEXTURE_MANAGER, |tm| tm.is_none()) {
        local_data::set(KEY_TEXTURE_MANAGER, TexturesManager::new())
    }

    local_data::get_mut(KEY_TEXTURE_MANAGER, |tm| f(tm.unwrap()))
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
    data:     Rc<ImageData>,
    interpol: Interpolation,
    overflow: Overflow
}

impl Texture2d {
    pub fn new(data:          Rc<ImageData>,
               interpolation: Interpolation,
               overflow:      Overflow)
               -> Texture2d {
        Texture2d {
            data:     data,
            interpol: interpolation,
            overflow: overflow 
        }
    }

    pub fn from_png(path: &Path, interpolation: Interpolation, overflow: Overflow) -> Option<Texture2d> {

        let data = get_texture_manager(|tm| {
            let res = match tm.loaded.find(&path.as_str().unwrap().to_owned()) {
                Some(data) => Some(data.clone()),
                None => {
                    let img = png::load_png(path);
                    if !img.is_ok() {
                        None
                    }
                    else {
                        let     img  = img.unwrap();
                        let mut data = ~[];

                        match img.color_type {
                            png::RGB8 => {
                                for p in img.pixels.chunks(3) {
                                    let r = p[0] as f32 / 255.0;
                                    let g = p[1] as f32 / 255.0;
                                    let b = p[2] as f32 / 255.0;

                                    data.push(Vec3::new(r, g, b));
                                }

                                Some(Rc::new(ImageData::new(data,
                                             Vec2::new(img.width as uint, img.height as uint))))
                            },
                            png::RGBA8 => {
                                for p in img.pixels.chunks(4) {
                                    let r = p[0] as f32 / 255.0;
                                    let g = p[1] as f32 / 255.0;
                                    let b = p[2] as f32 / 255.0;

                                    data.push(Vec3::new(r, g, b));
                                }

                                Some(Rc::new(ImageData::new(data,
                                             Vec2::new(img.width as uint, img.height as uint))))
                            },
                            _         => {
                                fail!("Unsuported data type.")
                            }
                        }
                    }
                }
            };

            let data = res.clone();
            data.map(|data| tm.loaded.insert(path.as_str().unwrap().to_owned(), data));

            res
        });

        data.map(|data| Texture2d::new(data, interpolation, overflow))
    }

    pub fn at<'a>(&'a self, x: uint, y: uint) -> &'a Vec3<f32> {
        let res = &'a self.data.borrow().pixels[y * self.data.borrow().dims.x + x];

        res
    }

    pub fn sample(&self, coords: &(N, N, N)) -> Vec3<f32> {
        let (ux, uy, _) = coords.clone();
        let mut ux: f32 = NumCast::from(ux).expect("Conversion of sampling coordinates failed.");
        let mut uy: f32 = NumCast::from(uy).expect("Conversion of sampling coordinates failed.");

        match self.overflow {
            ClampToEdges => {
                ux = ux.clamp(&0.0, &1.0);
                uy = 1.0 - uy.clamp(&0.0, &1.0);
            }
            Wrap => {
                uy = -uy;
                ux = ux % 1.0;
                uy = uy % 1.0;

                if ux < 0.0 { ux = 1.0 + ux };
                if uy < 0.0 { uy = 1.0 + uy };
            }
        }

        ux = ux * ((self.data.borrow().dims.x - 1) as f32);
        uy = uy * ((self.data.borrow().dims.y - 1) as f32);

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

                let u_interpol = ul * (1.0 - shift_ux) + ur * shift_ux;
                let d_interpol = dl * (1.0 - shift_ux) + dr * shift_ux;

                u_interpol * (1.0 - shift_uy) + d_interpol * shift_uy
            }
        }
    }
}
