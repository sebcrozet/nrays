use rustrt::bookkeeping;
use std::rand;
use std::sync::RWLock;
use std::cmp;
use std::rt;
use std::num::Zero;
use std::sync::Arc;
use nalgebra::na::{Vec4, Mat4};
use nalgebra::na::{Vec2, Vec3};
use nalgebra::na::{Dim, Indexable};
use nalgebra::na::Iterable;
use nalgebra::na;
use ncollide::bounding_volume::{AABB, HasAABB};
use ncollide::partitioning::BVT;
// use ncollide::partitioning::bvt_visitor::RayInterferencesCollector;
use ncollide::ray::{Ray, RayIntersection};
use ncollide::math::{Scalar, Vect};
use material::Material;
use ray_with_energy::RayWithEnergy;
use scene_node::SceneNode;
use image::Image;
use light::Light;

pub struct Scene {
    background: Vec3<f32>,
    lights:     Vec<Light>,
    world:      BVT<Arc<SceneNode>, AABB>
}

#[dim3]
pub type Vless = Vec2<Scalar>;

#[dim4]
pub type Vless = Vec3<Scalar>;

#[dim3]
pub fn render(scene:         &Arc<Scene>,
              resolution:    &Vless,
              ray_per_pixel: uint,
              window_width:  Scalar,
              camera_eye:    Vect,
              projection:    Mat4<Scalar>)
              -> Image {
    assert!(ray_per_pixel > 0);

    let npixels: uint = NumCast::from(resolution.x * resolution.y).unwrap();
    let pixels        = Vec::from_elem(npixels, na::zero());
    let pixels        = Arc::new(RWLock::new(pixels));

    let nrays = resolution.y * resolution.x * (ray_per_pixel as f64);

    println!("Tracing {} rays.", nrays as int);


    /*
     *
     * FIXME: we should use libgreen to do the repartition.
     * However, it does not work very well at the moment.
     *
     */
    let num_thread = rt::default_sched_threads();
    let resx       = resolution.x as uint;
    let resy       = resolution.y as uint;

    let mpixels = pixels.clone();
    let mscene  = scene.clone();

    for i in range(0u, num_thread) {
        let pixels    = mpixels.clone();
        let scene     = mscene.clone();
        let parts     = npixels / num_thread + 1;
        let low_limit = parts * i;
        let up_limit  = cmp::min(parts * (i + 1), npixels);

        spawn(proc() {
            let mut pxs = Vec::with_capacity(up_limit - low_limit);
            for ipt in range(low_limit, up_limit) {
                let j = ipt / resx;
                let i = ipt - j * resx;

                let mut tot_c: Vec3<f32> = na::zero();

                for _ in range(0u, ray_per_pixel) {
                    let perturbation  = (rand::random::<Vless>() - na::cast::<f32, Scalar>(0.5)) * window_width;
                    let orig: Vec2<Scalar> = Vec2::new(na::cast(i), na::cast(j)) + perturbation;

                    /*
                     * unproject
                     */
                    let device_x = (orig.x / (resx as f64) - 0.5) * 2.0;
                    let device_y = -(orig.y / (resy as f64) - 0.5) * 2.0;
                    let start = Vec4::new(device_x, device_y, -1.0, 1.0);
                    let h_eye = projection * start;
                    let eye: Vec3<f64> = na::from_homogeneous(&h_eye);
                    let ray = Ray::new(camera_eye, na::normalize(&(eye - camera_eye)));

                    let c: Vec3<f32> = scene.trace(&RayWithEnergy::new(ray.orig.clone(), ray.dir));

                    tot_c = tot_c + c;
                }

                pxs.push(tot_c / na::cast::<uint, f32>(ray_per_pixel));
            }

            {
                let pxs = pxs.as_slice();
                let mut bpixels = pixels.write();
                for ipt in range(low_limit, up_limit) {
                    let j = ipt / resx;
                    let i = ipt - j * resx;

                    *bpixels.get_mut(i + j * resx as uint) = pxs[ipt - low_limit];
                }
            }

        })
    }

    // FIXME: this might not be the canonical way of doing that…
    bookkeeping::wait_for_other_tasks();

    Image::new(resolution.clone(), pixels.read().clone())
}

impl Scene {
    pub fn new(nodes: Vec<Arc<SceneNode>>, lights: Vec<Light>, background: Vec3<f32>) -> Scene {
        let mut nodes_w_bvs = Vec::new();

        for n in nodes.move_iter() {
            nodes_w_bvs.push((n.clone(), n.aabb.clone()));
        }

        let bvt = BVT::new_kdtree(nodes_w_bvs);

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
    pub fn lights<'a>(&'a self) -> &'a [Light] {
        self.lights.as_slice()
    }
}

#[dim4]
impl Scene {
    pub fn render(&self, resolution: &Vless, unproject: |&Vless| -> Ray) -> Image {
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
        let mut pixels    = Vec::with_capacity(NumCast::from(npixels.clone()).unwrap());

        for _ in range(0u, NumCast::from(npixels).unwrap()) {
            // curr contains the index of the current sample point.
            let ray = unproject(&curr);
            let c   = self.trace(&RayWithEnergy::new(ray.orig.clone(), ray.dir));
            pixels.push(c);

            for j in range(0u, Dim::dim(None::<Vless>)) {
                let inc = curr.at(j) + na::one();

                if inc == resolution.at(j) {
                    curr.set(j, na::zero());
                }
                else {
                    curr.set(j, inc);
                    break;
                }
            }
        }

        Image::new(resolution.clone(), pixels)
    }
}

impl Scene {
    pub fn intersects_ray(&self, ray: &Ray, maxtoi: Scalar) -> Option<Vec3<f32>> {
        let mut filter = Vec3::new(1.0, 1.0, 1.0);

        let inter = self.world.cast_ray(ray, &mut |b, r| {
            // this will make a very coarce approximation of the transparent shadow.
            // (occluded objects might show up)
            match b.cast(r) {
                Some(t) => {
                    if t.toi <= maxtoi {
                        let color = b.material.ambiant(&(ray.orig + ray.dir * t.toi), &t.normal, &uvs(&t));

                        let alpha = color.w * b.alpha;

                        if alpha < 1.0 {
                            filter = filter * Vec3::new(color.x, color.y, color.z) * (1.0 - alpha);

                            None
                        }
                        else {
                            Some((t.toi, t.toi))
                        }
                    }
                    else {
                        None
                    }
                }
                _ => None
            }
        });

        if inter.is_none() {
            Some(filter)
        }
        else {
            None
        }
    }

    pub fn trace(&self, ray: &RayWithEnergy) -> Vec3<f32> {
        let cast = self.world.cast_ray(&ray.ray, &mut |b, r| { b.cast(r).map(|inter| (inter.toi, inter)) });

        match cast {
            None                 => self.background.clone(),
            Some((_, inter, sn)) => {
                let bsn       = sn.deref();
                let pt        = ray.ray.orig + ray.ray.dir * inter.toi;
                let obj       = bsn.material.compute(ray, &pt, &inter.normal, &uvs(&inter), self);
                let refl      = self.trace_reflection(bsn.refl_mix, bsn.refl_atenuation, ray, &pt, &inter.normal);

                let alpha     = obj.w * bsn.alpha;
                let obj_color = Vec3::new(obj.x, obj.y, obj.z) * (1.0 - bsn.refl_mix) + refl * bsn.refl_mix;
                let refr      = self.trace_refraction(alpha, bsn.refr_coeff, ray, &pt, &inter.normal);

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
    fn trace_reflection(&self, mix: f32, attenuation: f32, ray: &RayWithEnergy, pt: &Vect, normal: &Vect) -> Vec3<f32> {
        if !mix.is_zero() && ray.energy > 0.1 {
            let nproj      = normal * na::dot(&ray.ray.dir, normal);
            let rdir       = ray.ray.dir - nproj * na::cast::<f32, Scalar>(2.0);
            let new_energy = ray.energy - attenuation;

            self.trace(
                &RayWithEnergy::new_with_energy(
                    pt + rdir * na::cast::<f32, Scalar>(0.001),
                    rdir,
                    ray.refr.clone(),
                    new_energy))
        }
        else {
            na::zero()
        }
    }

    #[inline]
    fn trace_refraction(&self, alpha: f32, coeff: Scalar, ray: &RayWithEnergy, pt: &Vect, normal: &Vect) -> Vec3<f32>  {
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

            let dir_along_normal = normal * na::dot(&ray.ray.dir, normal);
            let tangent = ray.ray.dir - dir_along_normal;
            let new_dir = na::normalize(&(dir_along_normal + tangent * (n2 / n1)));
            let new_pt  = pt + new_dir * 0.001f64;

            self.trace(&RayWithEnergy::new_with_energy(new_pt, new_dir, n2, ray.energy))
        }
        else {
            na::zero()
        }
    }
}

#[dim3]
fn uvs(i: &RayIntersection) -> Option<Vec2<Scalar>> {
    i.uvs.clone()
}

#[not_dim3]
fn uvs(_: &RayIntersection) -> Option<Vec2<Scalar>> {
    None
}
