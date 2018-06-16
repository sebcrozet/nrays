use image::Image;
use num::Zero;
use num_cpus;
use rand::random;
use std::cmp;
use std::iter;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

use na::{self, Matrix4, Point2, Point3, Point4, Vector2, Vector3};
use ncollide3d::bounding_volume::AABB;
use ncollide3d::partitioning::{BVTCostFn, BVT};
use ncollide3d::query::{Ray, RayCast, RayIntersection};

use light::Light;
use math::{Isometry, Point, Scalar, Vect};
use ray_with_energy::RayWithEnergy;
use scene_node::SceneNode;

pub struct Scene {
    background: Vector3<f32>,
    lights: Vec<Light>,
    world: BVT<Arc<SceneNode>, AABB<Scalar>>,
}

pub type Vless = Vector2<Scalar>;

pub fn render(
    scene: &Arc<Scene>,
    resolution: &Vless,
    ray_per_pixel: usize,
    window_width: Scalar,
    camera_eye: Point,
    projection: Matrix4<Scalar>,
) -> Image {
    assert!(ray_per_pixel > 0);

    let npixels = (resolution.x * resolution.y) as usize;
    let pixels = iter::repeat(na::zero::<Vector3<f32>>())
        .take(npixels)
        .collect();
    let pixels: Arc<RwLock<Vec<Vector3<f32>>>> = Arc::new(RwLock::new(pixels));

    let nrays = resolution.y * resolution.x * (ray_per_pixel as f64);

    println!("Tracing {} rays.", nrays as i32);

    let num_thread = num_cpus::get();
    let resx = resolution.x as usize;
    let resy = resolution.y as usize;

    let mpixels = pixels.clone();
    let mscene = scene.clone();

    let mut children = Vec::new();

    for i in 0..num_thread {
        let pixels = mpixels.clone();
        let scene = mscene.clone();
        let parts = npixels / num_thread + 1;
        let low_limit = parts * i;
        let up_limit = cmp::min(parts * (i + 1), npixels);

        children.push(thread::spawn(move || {
            let mut pxs = Vec::with_capacity(up_limit - low_limit);
            for ipt in low_limit..up_limit {
                let j = ipt / resx;
                let i = ipt - j * resx;

                let mut tot_c: Vector3<f32> = na::zero();

                for _ in 0usize..ray_per_pixel {
                    let shift = Vless::from_element(0.5);
                    let perturbation = (random::<Vless>() - shift) * window_width;
                    let orig = Vector2::new(i as f64, j as f64) + perturbation;

                    /*
                     * unproject
                     */
                    let device_x = (orig.x / (resx as f64) - 0.5) * 2.0;
                    let device_y = -(orig.y / (resy as f64) - 0.5) * 2.0;
                    let start = Point4::new(device_x, device_y, -1.0, 1.0);
                    let h_eye = projection * start;
                    let eye = Point3::from_homogeneous(h_eye.coords).unwrap();
                    let ray = Ray::new(camera_eye, na::normalize(&(eye - camera_eye)));

                    let c: Vector3<f32> =
                        scene.trace(&RayWithEnergy::new(ray.origin.clone(), ray.dir));

                    tot_c = tot_c + c;
                }

                pxs.push(tot_c / (ray_per_pixel as f32));
            }

            {
                let pxs = &pxs[..];
                let mut bpixels = pixels.write().unwrap();
                for ipt in low_limit..up_limit {
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
    pub fn new(nodes: Vec<Arc<SceneNode>>, lights: Vec<Light>, background: Vector3<f32>) -> Scene {
        let mut nodes_w_bvs = Vec::new();

        for n in nodes.into_iter() {
            nodes_w_bvs.push((n.clone(), n.aabb.clone()));
        }

        let bvt = BVT::new_balanced(nodes_w_bvs);

        Scene {
            lights: lights,
            world: bvt,
            background: background,
        }
    }

    #[inline]
    pub fn set_background(&mut self, background: Vector3<f32>) {
        self.background = background
    }

    #[inline]
    pub fn lights(&self) -> &[Light] {
        &self.lights[..]
    }
}

impl Scene {
    pub fn intersects_ray(&self, ray: &Ray<Scalar>, maxtoi: Scalar) -> Option<Vector3<f32>> {
        let mut filter = Vector3::new(1.0, 1.0, 1.0);

        let inter;
        {
            let mut shadow_caster = TransparentShadowsRayTOICostFn::new(ray, maxtoi, &mut filter);
            inter = self.world.best_first_search(&mut shadow_caster);
        }

        if inter.is_none() {
            Some(filter)
        } else {
            None
        }
    }

    pub fn trace(&self, ray: &RayWithEnergy) -> Vector3<f32> {
        let cast = self
            .world
            .best_first_search(&mut ClosestRayTOICostFn::new(&ray.ray));

        match cast {
            None => self.background.clone(),
            Some((sn, inter)) => {
                let pt = ray.ray.origin + ray.ray.dir * inter.toi;
                let obj = sn
                    .material
                    .compute(ray, &pt, &inter.normal, &uvs(&inter), self);
                let refl =
                    self.trace_reflection(sn.refl_mix, sn.refl_atenuation, ray, &pt, &inter.normal);

                let alpha = obj.w * sn.alpha;
                let obj_color =
                    Vector3::new(obj.x, obj.y, obj.z) * (1.0 - sn.refl_mix) + refl * sn.refl_mix;
                let refr = self.trace_refraction(alpha, sn.refr_coeff, ray, &pt, &inter.normal);

                if alpha == 1.0 {
                    Vector3::new(obj_color.x, obj_color.y, obj_color.z)
                } else {
                    let obj_color = Vector3::new(obj_color.x, obj_color.y, obj_color.z);
                    let refr = Vector3::new(refr.x, refr.y, refr.z);

                    obj_color * alpha + refr * (1.0 - alpha)
                }
            }
        }
    }

    #[inline]
    fn trace_reflection(
        &self,
        mix: f32,
        attenuation: f32,
        ray: &RayWithEnergy,
        pt: &Point,
        normal: &Vect,
    ) -> Vector3<f32> {
        if !mix.is_zero() && ray.energy > 0.1 {
            let nproj = *normal * na::dot(&ray.ray.dir, normal);
            let rdir = ray.ray.dir - nproj * 2.0;
            let new_energy = ray.energy - attenuation;

            self.trace(&RayWithEnergy::new_with_energy(
                *pt + rdir * 0.001,
                rdir,
                ray.refr.clone(),
                new_energy,
            ))
        } else {
            na::zero()
        }
    }

    #[inline]
    fn trace_refraction(
        &self,
        alpha: f32,
        coeff: Scalar,
        ray: &RayWithEnergy,
        pt: &Point,
        normal: &Vect,
    ) -> Vector3<f32> {
        if alpha != 1.0 {
            let n1;
            let n2;

            if ray.refr == 1.0 {
                n1 = 1.0;
                n2 = coeff;
            } else {
                n1 = coeff;
                n2 = 1.0;
            }

            let dir_along_normal = *normal * na::dot(&ray.ray.dir, normal);
            let tangent = ray.ray.dir - dir_along_normal;
            let new_dir = na::normalize(&(dir_along_normal + tangent * (n2 / n1)));
            let new_pt = *pt + new_dir * 0.001f64;

            self.trace(&RayWithEnergy::new_with_energy(
                new_pt, new_dir, n2, ray.energy,
            ))
        } else {
            na::zero()
        }
    }
}

fn uvs(i: &RayIntersection<Scalar>) -> Option<Point2<Scalar>> {
    i.uvs.clone()
}

/*
 * Define our own cost functions as the Arc<SceneNode> does not implement the `RayCast` trait.
 */
pub struct ClosestRayTOICostFn<'a> {
    ray: &'a Ray<Scalar>,
}

impl<'a> ClosestRayTOICostFn<'a> {
    pub fn new(ray: &'a Ray<Scalar>) -> ClosestRayTOICostFn<'a> {
        ClosestRayTOICostFn { ray: ray }
    }
}
impl<'a> BVTCostFn<Scalar, Arc<SceneNode>, AABB<Scalar>> for ClosestRayTOICostFn<'a> {
    type UserData = RayIntersection<Scalar>;

    #[inline]
    fn compute_bv_cost(&mut self, bv: &AABB<Scalar>) -> Option<Scalar> {
        bv.toi_with_ray(&Isometry::identity(), self.ray, true)
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &Arc<SceneNode>) -> Option<(Scalar, RayIntersection<Scalar>)> {
        b.cast(self.ray).map(|inter| (inter.toi, inter))
    }
}

pub struct TransparentShadowsRayTOICostFn<'a> {
    ray: &'a Ray<Scalar>,
    maxtoi: Scalar,
    filter: &'a mut Vector3<f32>,
}

impl<'a> TransparentShadowsRayTOICostFn<'a> {
    pub fn new(
        ray: &'a Ray<Scalar>,
        maxtoi: Scalar,
        filter: &'a mut Vector3<f32>,
    ) -> TransparentShadowsRayTOICostFn<'a> {
        TransparentShadowsRayTOICostFn {
            ray: ray,
            maxtoi: maxtoi,
            filter: filter,
        }
    }
}
impl<'a> BVTCostFn<Scalar, Arc<SceneNode>, AABB<Scalar>> for TransparentShadowsRayTOICostFn<'a> {
    type UserData = ();

    #[inline]
    fn compute_bv_cost(&mut self, bv: &AABB<Scalar>) -> Option<Scalar> {
        bv.toi_with_ray(&Isometry::identity(), self.ray, true)
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &Arc<SceneNode>) -> Option<(Scalar, ())> {
        match b.cast(self.ray) {
            Some(t) => {
                if t.toi <= self.maxtoi {
                    let color = b.material.ambiant(
                        &(self.ray.origin + self.ray.dir * t.toi),
                        &t.normal,
                        &uvs(&t),
                    );
                    let alpha = color.w * b.alpha;

                    if alpha < 1.0 {
                        let rgb = Vector3::new(color.x, color.y, color.z);
                        *self.filter = self.filter.component_mul(&rgb) * (1.0 - alpha);

                        None
                    } else {
                        Some((t.toi, ()))
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
