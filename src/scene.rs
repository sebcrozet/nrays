use std::vec;
use std::rt;
use extra::arc::{Arc, RWArc};
use nalgebra::na::{Vec3, Vec4, Mat4};
use nalgebra::na;
use ncollide::bounding_volume::{AABB, HasAABB};
use ncollide::partitioning::BVT;
// use ncollide::partitioning::bvt_visitor::RayInterferencesCollector;
use ncollide::ray::{Ray, RayCast};
use ncollide::math::{N, V};
use material::Material;
use ray_with_energy::RayWithEnergy;
use scene_node::SceneNode;
use image::Image;
use light::Light;

#[cfg(dim4)]
use nalgebra::na::{Dim, Indexable};
#[cfg(dim4)]
use nalgebra::na::Iterable;

#[cfg(dim3)]
use std::rand;
#[cfg(dim3)]
use nalgebra::na::Vec2;
#[cfg(dim3)]
use std::io::stdio;
#[cfg(dim3)]
use native;

pub struct Scene {
    priv lights: ~[Light],
    priv world:  BVT<Arc<SceneNode>, AABB>
}

#[cfg(dim3)]
pub type Vless = Vec2<N>;

#[cfg(dim4)]
pub type Vless = Vec3<N>;

#[cfg(dim3)]
pub fn render(scene:         &Arc<Scene>,
              resolution:    &Vless,
              ray_per_pixel: uint,
              window_width:  N,
              camera_eye:    V,
              projection:    Mat4<N>)
              -> Image {
    assert!(ray_per_pixel > 0);

    let npixels: uint = NumCast::from(resolution.x * resolution.y).unwrap();
    let pixels        = vec::from_elem(npixels, na::zero());
    let pixels        = RWArc::new(pixels);

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

    do native::run {
        for i in range(0u, num_thread) {
            let pixels    = mpixels.clone();
            let scene     = mscene.clone();
            let parts     = npixels / num_thread + 1;
            let low_limit = parts * i;
            let up_limit  = (parts * (i + 1)).min(&npixels);

            do spawn() {
                for ipt in range(low_limit, up_limit) {
                    let j = ipt / resx;
                    let i = ipt - j * resx;

                    let mut tot_c: Vec3<f32> = na::zero();

                    for _ in range(0u, ray_per_pixel) {
                        let perturbation  = (rand::random::<Vless>() - na::cast::<f32, N>(0.5)) * window_width;
                        let orig: Vec2<N> = Vec2::new(na::cast(i), na::cast(j)) + perturbation;

                        /*
                         * unproject
                         */
                        let device_x = (orig.x / (resx as f64) - 0.5) * 2.0;
                        let device_y = -(orig.y / (resy as f64) - 0.5) * 2.0;
                        let start = Vec4::new(device_x, device_y, -1.0, 1.0);
                        let h_eye = projection * start;
                        let eye: Vec3<f64> = na::from_homogeneous(&h_eye);
                        let ray = Ray::new(camera_eye, na::normalize(&(eye - camera_eye)));


                        let c: Vec3<f32> = scene.get().trace(&RayWithEnergy::new(ray.orig.clone(), ray.dir));

                        tot_c = tot_c + c;
                    }

                    pixels.write(|p| p[i + j * resx as uint] = tot_c / na::cast::<uint, f32>(ray_per_pixel));
                }
            }
        }
    };

    pixels.read(|p| Image::new(resolution.clone(), p.clone()))
}

impl Scene {
    pub fn new(nodes: ~[Arc<SceneNode>], lights: ~[Light]) -> Scene {
        let mut nodes_w_bvs = ~[];

        for n in nodes.move_iter() {
            nodes_w_bvs.push((n.clone(), n.get().aabb.clone()));
        }

        let bvt = BVT::new_kdtree(nodes_w_bvs);

        Scene {
            lights: lights,
            world:  bvt
        }
    }

    #[inline]
    pub fn lights<'a>(&'a self) -> &'a [Light] {
        let res: &'a [Light] = self.lights;

        res
    }

    #[cfg(dim3)]
    pub fn render(&self,
    resolution:    &Vless,
    ray_per_pixel: uint,
    window_width:  N,
    unproject:     |&Vless| -> Ray)
        -> Image {
            assert!(ray_per_pixel > 0);

            let npixels: uint = NumCast::from(resolution.x * resolution.y).unwrap();
            let mut pixels    = vec::with_capacity(npixels);

            let nrays = resolution.y * resolution.x * (ray_per_pixel as f64);
            println!("Tracing {} rays.", nrays as int);
            let mut rays_done = 0;
            let mut progress  = 0;

            print!("{} / {} − {}%\r", rays_done, nrays, progress);
            stdio::flush();

            for j in range(0u, na::cast(resolution.y)) {
                for i in range(0u, na::cast(resolution.x)) {
                    let mut tot_c: Vec3<f32> = na::zero();

                    for _ in range(0u, ray_per_pixel) {
                        rays_done = rays_done + 1;

                        let perturbation  = (rand::random::<Vless>() - na::cast::<f32, N>(0.5)) * window_width;
                        let orig: Vec2<N> = Vec2::new(na::cast(i), na::cast(j)) + perturbation;

                        let ray = unproject(&orig);
                        let c   = self.trace(&RayWithEnergy::new(ray.orig.clone(), ray.dir));
                        let new_progress = ((rays_done as f64) / nrays * 100.0) as int;

                        if new_progress != progress {
                            progress = new_progress;
                            print!("{} / {} − {}%\r", rays_done, nrays, progress);
                            stdio::flush();
                        }

                        tot_c = tot_c + c;
                    }

                    pixels.push(tot_c / na::cast::<uint, f32>(ray_per_pixel));
                }
            }

            Image::new(resolution.clone(), pixels)
        }

    #[cfg(dim4)]
    pub fn render(&self, resolution: &Vless, unproject: |&Vless| -> Ray) -> Image {
        let mut npixels: N = na::one();

        for i in resolution.iter() {
            npixels = npixels * *i;
        }

        let mut curr: Vless = na::zero();

        // Sample a rectangular n-1 surface (with n the rendering dimension):
        //   * a rectangle for 3d rendering.
        //   * a cube for 4d rendering.
        //   * an hypercube for 5d rendering.
        //   * etc
        let mut pixels    = vec::with_capacity(NumCast::from(npixels.clone()).unwrap());

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

    pub fn intersects_ray(&self, ray: &Ray, maxtoi: N) -> bool {
        self.world.cast_ray(ray, &|b, r| {
            match b.get().geometry.toi_with_transform_and_ray(&b.get().transform, r) {
                None    => None,
                Some(t) => if t <= maxtoi { Some((na::cast(-1.0), t)) } else { None }
            }
        }).is_some()
    }

    pub fn trace(&self, ray: &RayWithEnergy) -> Vec3<f32> {
        let cast = self.world.cast_ray(&ray.ray, &|b, r| { b.get().cast(r).map(|(t, n, u)| (t, (t, n, u))) });

        match cast {
            None     => Vec3::new(0.0, 0.0, 0.0),
            Some((_, toi, sn)) => {
                let inter = ray.ray.orig + ray.ray.dir * *toi.n0_ref();

                let obj  = sn.get().material.get().compute(ray, &inter, toi.n1_ref(), toi.n2_ref(), self);
                let refl = sn.get().refl.compute(ray, &inter, toi.n1_ref(), toi.n2_ref(), self);

                obj * (1.0 - sn.get().refl.mix)+ refl * sn.get().refl.mix
            }
        }
    }
}
