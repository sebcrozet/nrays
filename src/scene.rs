#[cfg(feature = "3d")]
use rand::random;
#[cfg(feature = "3d")]
use std::sync::RwLock;
#[cfg(feature = "3d")]
use std::cmp;
#[cfg(feature = "3d")]
use std::thread;
#[cfg(feature = "3d")]
use std::iter;
#[cfg(feature = "3d")]
use na::{Pnt3, Pnt4, Vec2, Mat4};
#[cfg(feature = "3d")]
use num_cpus::{self};
use num::Zero;
use std::sync::Arc;
use na::{Identity, Pnt2, Vec3};
use na;
use ncollide::bounding_volume::AABB;
use ncollide::partitioning::{BVT, BVTCostFn};
// use ncollide::partitioning::bvt_visitor::RayInterferencesCollector;
use ncollide::ray::{Ray, RayIntersection, RayCast};
use math::{Scalar, Point, Vect};
use material::Material;
use ray_with_energy::RayWithEnergy;
use scene_node::SceneNode;
use image::Image;
use light::Light;

#[cfg(feature = "4d")]
use na::Iterable;

pub struct Scene {
    background: Vec3<f32>,
    lights:     Vec<Light>,
    world:      BVT<Arc<SceneNode>, AABB<Point>>
}

#[cfg(feature = "3d")]
pub type Vless = Vec2<Scalar>;

#[cfg(feature = "4d")]
pub type Vless = Vec3<Scalar>;

#[cfg(feature = "3d")]
pub fn render(scene:         &Arc<Scene>,
              resolution:    &Vless,
              ray_per_pixel: usize,
              window_width:  Scalar,
              camera_eye:    Point,
              projection:    Mat4<Scalar>)
              -> Image {
    assert!(ray_per_pixel > 0);

    let npixels: usize = na::cast(resolution.x * resolution.y);
    let pixels         = iter::repeat(na::zero::<Vec3<f32>>()).take(npixels).collect();
    let pixels: Arc<RwLock<Vec<Vec3<f32>>>> = Arc::new(RwLock::new(pixels));

    let nrays = resolution.y * resolution.x * (ray_per_pixel as f64);

    println!("Tracing {} rays.", nrays as i32);

    let num_thread = num_cpus::get();
    let resx       = resolution.x as usize;
    let resy       = resolution.y as usize;

    let mpixels = pixels.clone();
    let mscene  = scene.clone();

    let mut children = Vec::new();

    for i in 0 .. num_thread {
        let pixels    = mpixels.clone();
        let scene     = mscene.clone();
        let parts     = npixels / num_thread + 1;
        let low_limit = parts * i;
        let up_limit  = cmp::min(parts * (i + 1), npixels);

        children.push(thread::spawn(move || {
            let mut pxs = Vec::with_capacity(up_limit - low_limit);
            for ipt in low_limit .. up_limit {
                let j = ipt / resx;
                let i = ipt - j * resx;

                let mut tot_c: Vec3<f32> = na::zero();

                for _ in 0usize .. ray_per_pixel {
                    let perturbation  = (random::<Vless>() - na::cast::<f32, Scalar>(0.5)) * window_width;
                    let orig: Vec2<Scalar> = Vec2::new(na::cast::<usize, Scalar>(i), na::cast::<usize, Scalar>(j)) + perturbation;

                    /*
                     * unproject
                     */
                    let device_x = (orig.x / (resx as f64) - 0.5) * 2.0;
                    let device_y = -(orig.y / (resy as f64) - 0.5) * 2.0;
                    let start = Pnt4::new(device_x, device_y, -1.0, 1.0);
                    let h_eye = projection * start;
                    let eye: Pnt3<f64> = na::from_homogeneous(&h_eye);
                    let ray = Ray::new(camera_eye, na::normalize(&(eye - camera_eye)));

                    let c: Vec3<f32> = scene.trace(&RayWithEnergy::new(ray.orig.clone(), ray.dir));

                    tot_c = tot_c + c;
                }

                pxs.push(tot_c / na::cast::<usize, f32>(ray_per_pixel));
            }

            {
                let pxs = &pxs[..];
                let mut bpixels = pixels.write().unwrap();
                for ipt in low_limit .. up_limit {
                    let j = ipt / resx;
                    let i = ipt - j * resx;

                    bpixels[i + j * resx as usize] = pxs[ipt - low_limit];
                }
            }

        }));
    }

    for child in children.into_iter() {
        let _ = child.join();
    }

    let out_pixels = pixels.read().unwrap().clone();
    Image::new(resolution.clone(), out_pixels)
}

impl Scene {
    pub fn new(nodes: Vec<Arc<SceneNode>>, lights: Vec<Light>, background: Vec3<f32>) -> Scene {
        let mut nodes_w_bvs = Vec::new();

        for n in nodes.into_iter() {
            nodes_w_bvs.push((n.clone(), n.aabb.clone()));
        }

        let bvt = BVT::new_balanced(nodes_w_bvs);

        Scene {
            lights:     lights,
            world:      bvt,
            background: background
        }
    }

    #[inline]
    pub fn set_background(&mut self, background: Vec3<f32>) {
        self.background = background
    }

    #[inline]
    pub fn lights(&self) -> &[Light] {
        &self.lights[..]
    }
}

#[cfg(feature = "4d")]
impl Scene {
    pub fn render<F>(&self, resolution: &Vless, unproject: F) -> Image
        where F: Fn(&Vless) -> Ray<Point> {
        let mut npixels: Scalar = na::one();

        for i in resolution.iter() {
            npixels = npixels * *i;
        }

        let mut curr: Vless = na::zero();

        // Sample a rectangular n-1 surface (with n the rendering dimension):
        //   * a rectangle for 3d rendering.
        //   * a cube for 4d rendering.
        //   * an hypercube for 5d rendering.
        //   * etc
        let mut pixels = Vec::with_capacity(na::cast(npixels.clone()));

        for _ in 0usize .. na::cast(npixels) {
            // curr contains the index of the current sample point.
            let ray = unproject(&curr);
            let c   = self.trace(&RayWithEnergy::new(ray.orig.clone(), ray.dir));
            pixels.push(c);

            for j in 0 .. na::dim::<Vless>() {
                let inc = curr[j] + na::one::<Scalar>();

                if inc == resolution[j] {
                    curr[j] = na::zero();
                }
                else {
                    curr[j] = inc;
                    break;
                }
            }
        }

        Image::new(resolution.clone(), pixels)
    }
}

impl Scene {
    pub fn intersects_ray(&self, ray: &Ray<Point>, maxtoi: Scalar) -> Option<Vec3<f32>> {
        let mut filter        = Vec3::new(1.0, 1.0, 1.0);

        let inter;
        {
            let mut shadow_caster = TransparentShadowsRayTOICostFn::new(ray, maxtoi, &mut filter);
            inter = self.world.best_first_search(&mut shadow_caster);
        }

        if inter.is_none() {
            Some(filter)
        }
        else {
            None
        }
    }

    pub fn trace(&self, ray: &RayWithEnergy) -> Vec3<f32> {
        let cast = self.world.best_first_search(&mut ClosestRayTOICostFn::new(&ray.ray));

        match cast {
            None                 => self.background.clone(),
            Some((sn, inter)) => {
                let pt        = ray.ray.orig + ray.ray.dir * inter.toi;
                let obj       = sn.material.compute(ray, &pt, &inter.normal, &uvs(&inter), self);
                let refl      = self.trace_reflection(sn.refl_mix, sn.refl_atenuation, ray, &pt, &inter.normal);

                let alpha     = obj.w * sn.alpha;
                let obj_color = Vec3::new(obj.x, obj.y, obj.z) * (1.0 - sn.refl_mix) + refl * sn.refl_mix;
                let refr      = self.trace_refraction(alpha, sn.refr_coeff, ray, &pt, &inter.normal);

                if alpha == 1.0 {
                    Vec3::new(obj_color.x, obj_color.y, obj_color.z)
                }
                else {
                    let obj_color = Vec3::new(obj_color.x, obj_color.y, obj_color.z);
                    let refr      = Vec3::new(refr.x, refr.y, refr.z);

                    obj_color * alpha + refr * (1.0 - alpha)
                }
            }
        }
    }

    #[inline]
    fn trace_reflection(&self, mix: f32, attenuation: f32, ray: &RayWithEnergy, pt: &Point, normal: &Vect) -> Vec3<f32> {
        if !mix.is_zero() && ray.energy > 0.1 {
            let nproj      = *normal * na::dot(&ray.ray.dir, normal);
            let rdir       = ray.ray.dir - nproj * na::cast::<f32, Scalar>(2.0);
            let new_energy = ray.energy - attenuation;

            self.trace(
                &RayWithEnergy::new_with_energy(
                    *pt + rdir * na::cast::<f32, Scalar>(0.001),
                    rdir,
                    ray.refr.clone(),
                    new_energy))
        }
        else {
            na::zero()
        }
    }

    #[inline]
    fn trace_refraction(&self, alpha: f32, coeff: Scalar, ray: &RayWithEnergy, pt: &Point, normal: &Vect) -> Vec3<f32>  {
        if alpha != 1.0 {
            let n1;
            let n2;

            if ray.refr == na::cast(1.0f64) {
                n1 = na::cast(1.0f64);
                n2 = coeff;
            }
            else {
                n1 = coeff;
                n2 = na::cast(1.0f64);
            }

            let dir_along_normal = *normal * na::dot(&ray.ray.dir, normal);
            let tangent = ray.ray.dir - dir_along_normal;
            let new_dir = na::normalize(&(dir_along_normal + tangent * (n2 / n1)));
            let new_pt  = *pt + new_dir * 0.001f64;

            self.trace(&RayWithEnergy::new_with_energy(new_pt, new_dir, n2, ray.energy))
        }
        else {
            na::zero()
        }
    }
}

#[cfg(feature = "3d")]
fn uvs(i: &RayIntersection<Vect>) -> Option<Pnt2<Scalar>> {
    i.uvs.clone()
}

#[cfg(not(feature = "3d"))]
fn uvs(_: &RayIntersection<Vect>) -> Option<Pnt2<Scalar>> {
    None
}

/*
 * Define our own cost functions as the Arc<SceneNode> does not implement the `RayCast` trait.
 */
pub struct ClosestRayTOICostFn<'a> {
    ray:   &'a Ray<Point>
}

impl<'a> ClosestRayTOICostFn<'a> {
    pub fn new(ray: &'a Ray<Point>) -> ClosestRayTOICostFn<'a> {
        ClosestRayTOICostFn {
            ray: ray
        }
    }
}
impl<'a> BVTCostFn<Scalar, Arc<SceneNode>, AABB<Point>, RayIntersection<Vect>> for ClosestRayTOICostFn<'a> {
    #[inline]
    fn compute_bv_cost(&mut self, bv: &AABB<Point>) -> Option<Scalar> {
        bv.toi_with_ray(&Identity::new(), self.ray, true)
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &Arc<SceneNode>) -> Option<(Scalar, RayIntersection<Vect>)> {

        b.cast(self.ray).map(|inter| (inter.toi, inter))
    }
}

pub struct TransparentShadowsRayTOICostFn<'a> {
    ray:    &'a Ray<Point>,
    maxtoi: Scalar,
    filter: &'a mut Vec3<f32>
}

impl<'a> TransparentShadowsRayTOICostFn<'a> {
    pub fn new(ray: &'a Ray<Point>, maxtoi: Scalar, filter: &'a mut Vec3<f32>)
        -> TransparentShadowsRayTOICostFn<'a> {
        TransparentShadowsRayTOICostFn {
            ray:    ray,
            maxtoi: maxtoi,
            filter: filter
        }
    }
}
impl<'a> BVTCostFn<Scalar, Arc<SceneNode>, AABB<Point>, ()>
for TransparentShadowsRayTOICostFn<'a> {
    #[inline]
    fn compute_bv_cost(&mut self, bv: &AABB<Point>) -> Option<Scalar> {
        bv.toi_with_ray(&Identity::new(), self.ray, true)
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &Arc<SceneNode>) -> Option<(Scalar, ())> {
        match b.cast(self.ray) {
            Some(t) => {
                if t.toi <= self.maxtoi {
                    let color = b.material.ambiant(&(self.ray.orig + self.ray.dir * t.toi), &t.normal, &uvs(&t));
                    let alpha = color.w * b.alpha;

                    if alpha < 1.0 {
                        *self.filter = *self.filter * Vec3::new(color.x, color.y, color.z) * (1.0 - alpha);

                        None
                    }
                    else {
                        Some((t.toi, ()))
                    }
                }
                else {
                    None
                }
            }
            _ => None
        }
    }
}
