use std::hashmap::HashMap;
use std::local_data;
use extra::arc::Arc;
use stb_image::image::ImageU8;
use stb_image::image;
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
    loaded: HashMap<~str, Arc<ImageData>>
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

    pub fn from_png(path: &Path, interpolation: Interpolation, overflow: Overflow) -> Option<Texture2d> {

        let data = get_texture_manager(|tm| {
            let res = match tm.loaded.find(&path.as_str().unwrap().to_owned()) {
                Some(data) => Some(data.clone()),
                None => {
                    match image::load(path.as_str().unwrap().to_owned()) {
                        ImageU8(mut image) => {
                            let mut data = ~[];

                            // Flip the y axis
                            let elt_per_row = image.width * image.depth;
                            for j in range(0u, image.height / 2) {
                                for i in range(0u, elt_per_row) {
                                    image.data.swap(
                                        (image.height - j - 1) * elt_per_row + i,
                                        j * elt_per_row + i)
                                }
                            }

                            if image.depth == 1 {
                                for p in image.data.iter() {
                                    let g = *p as f32 / 255.0;

                                    data.push(Vec3::new(g, g, g));
                                }

                                Some(Arc::new(ImageData::new(data,
                                Vec2::new(image.width as uint, image.height as uint))))
                            }

                            else if image.depth == 3 {
                                for p in image.data.chunks(3) {
                                    let r = p[0] as f32 / 255.0;
                                    let g = p[1] as f32 / 255.0;
                                    let b = p[2] as f32 / 255.0;

                                    data.push(Vec3::new(r, g, b));
                                }

                                Some(Arc::new(ImageData::new(data,
                                Vec2::new(image.width as uint, image.height as uint))))
                            }
                            else if image.depth == 4 {
                                for p in image.data.chunks(4) {
                                    let r = p[0] as f32 / 255.0;
                                    let g = p[1] as f32 / 255.0;
                                    let b = p[2] as f32 / 255.0;

                                    data.push(Vec3::new(r, g, b));
                                }

                                Some(Arc::new(ImageData::new(data,
                                Vec2::new(image.width as uint, image.height as uint))))
                            }
                            else {
                                fail!("Image depth {} not suported.", image.depth);
                            }
                        },
                        _ => {
                            None
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
        let res = &'a self.data.get().pixels[y * self.data.get().dims.x + x];

        res
    }

    pub fn sample(&self, coords: &Vec3<N>) -> Vec3<f32> {
        let mut ux: f32 = NumCast::from(coords.x).expect("Conversion of sampling coordinates failed.");
        let mut uy: f32 = NumCast::from(coords.y).expect("Conversion of sampling coordinates failed.");

        match self.overflow {
            ClampToEdges => {
                ux = ux.clamp(&0.0, &1.0);
                uy = uy.clamp(&0.0, &1.0);
            }
            Wrap => {
                ux = ux % 1.0;
                uy = uy % 1.0;

                if ux < 0.0 { ux = 1.0 + ux };
                if uy < 0.0 { uy = 1.0 + uy };
            }
        }

        ux = ux * ((self.data.get().dims.x - 1) as f32);
        uy = uy * ((self.data.get().dims.y - 1) as f32);

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
