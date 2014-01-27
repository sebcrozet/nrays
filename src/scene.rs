use std::vec;
use nalgebra::na::Vec3;
use nalgebra::na;
use ncollide::bounding_volume::{AABB, HasAABB};
use ncollide::partitioning::BVT;
// use ncollide::partitioning::bvt_visitor::RayInterferencesCollector;
use ncollide::ray::{Ray, RayCastWithTransform};
use ncollide::math::N;
use material::Material;
use ray_with_energy::RayWithEnergy;
use light_path::LightPath;
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

pub struct Scene {
    priv lights: ~[Light],
    priv world:  BVT<@SceneNode, AABB>
}

#[cfg(dim3)]
type Vless = Vec2<N>;

#[cfg(dim4)]
type Vless = Vec3<N>;

impl Scene {
    pub fn new(nodes: ~[@SceneNode], lights: ~[Light]) -> Scene {
        let mut nodes_w_bvs = ~[];

        for n in nodes.move_iter() {
            nodes_w_bvs.push((n, n.aabb.clone()));
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
    pub fn render(&mut self,
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


        let nb_lights = self.lights().len();
        let mut light_progress = 0;

        let mut extra_lights: ~[Light] = ~[];
        // Light tracing
        for light in self.lights.iter() {
            print!("Processing light\n");
            light.sample(|pos| {
                let dir  = pos - light.pos();
                let mut path : &mut LightPath = &mut LightPath::new(pos, dir, light.color);
                for t in range(0, 10) {
                      self.trace_path(path);
                      let nproj = path.normal_contact * na::dot(&path.ray.dir,
                                                                &path.normal_contact);
                      let rdir  = path.ray.dir - nproj * na::cast::<f32, N>(2.0);
                      path.ray.orig = path.last_intersection + rdir * na::cast::<f32, N>(0.001);
                      path.ray.dir = rdir;
                     // path.ray.orig = path.last_intersection;

                      path.total_color = (path.total_color + path.color) * path.mix_coef;
                      extra_lights.push(Light::new(path.last_intersection, 0.0, 1, path.total_color));
                      path.color = Vec3::new(0.0f32, 0.0, 0.0);
                }
                light_progress = light_progress + 1;
                let percentage = light_progress;
                print!("lights done {} \n", percentage);
            });
        }

        for l in extra_lights.iter() {
            self.lights.push(*l);
        }

        for j in range(0u, na::cast(resolution.y)) {
            for i in range(0u, na::cast(resolution.x)) {
                let mut tot_c: Vec3<f32> = na::zero();

                for _ in range(0u, ray_per_pixel) {
                    rays_done = rays_done + 1;

                    let perturbation  = (rand::random::<Vless>() - na::cast::<f32, N>(0.5)) * window_width;
                    let orig: Vec2<N> = Vec2::new(na::cast(i), na::cast(j)) + perturbation;

                    let ray = unproject(&orig);
                    let mut path : &mut LightPath = &mut LightPath::new(ray.orig, ray.dir,
                                                                        Vec3::new(0.0f32, 0.0, 0.0));
                   let mut colors: ~[Vec3<f32>] = ~[];
                   let mut mixes: ~[f32] = ~[];

                   for t in range(0, 4) {
                        self.trace_path(path);
                        let nproj = path.normal_contact * na::dot(&path.ray.dir,
                                                                  &path.normal_contact);
                        let rdir  = path.ray.dir - nproj * na::cast::<f32, N>(2.0);
                        path.ray.orig = path.last_intersection + rdir * na::cast::<f32, N>(0.001);
                        path.ray.dir = rdir;
                       // path.ray.orig = path.last_intersection;

                        colors.push(path.color);
                        mixes.push(path.mix_coef);
                        path.color = Vec3::new(0.0f32, 0.0, 0.0);

                    }

                    let new_progress = ((rays_done as f64) / nrays * 100.0) as int;

                    if new_progress != progress {
                        progress = new_progress;
                        print!("{} / {} − {}%\r", rays_done, nrays, progress);
                        stdio::flush();
                    }

                    let mut total_color = Vec3::new(0.0f32, 0.0, 0.0);
                    for i in range(0, colors.len()) {
                        let index = colors.len() - 1 - i;
                        total_color = (total_color + colors[index]) * mixes[index];
                    }

                    tot_c = tot_c + total_color;
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
            match b.geometry.toi_with_transform_and_ray(&b.transform, r) {
                None    => None,
                Some(t) => if t <= maxtoi { Some((na::cast(-1.0), t)) } else { None }
            }
        }).is_some()
    }

    pub fn trace(&self, ray: &RayWithEnergy) -> Vec3<f32> {
        let cast = self.world.cast_ray(&ray.ray, &|b, r| { b.cast(r).map(|(t, n, u)| (t, (t, n, u))) });

        match cast {
            None     => Vec3::new(0.0, 0.0, 0.0),
            Some((_, toi, sn)) => {
                let inter = ray.ray.orig + ray.ray.dir * *toi.n0_ref();

                let obj  = sn.material.compute(ray, &inter, toi.n1_ref(), toi.n2_ref(), self);
                let refl = sn.refl.compute(ray, &inter, toi.n1_ref(), toi.n2_ref(), self);

                obj * (1.0 - sn.refl.mix)+ refl * sn.refl.mix
            }
        }
    }

    pub fn trace_path(&self, path: &mut LightPath) {
        let cast = self.world.cast_ray(&path.ray, &|b, r| { b.cast(r).map(|(t, n, u)| (t, (t, n, u))) });

        match cast {
            None     => {path.no_hit = true;},
            Some((_, toi, sn)) => {
                let inter = path.ray.orig + path.ray.dir * *toi.n0_ref();
                path.last_intersection = inter;
                path.no_hit = false;

                sn.material.compute_for_light_path(path, &inter, toi.n1_ref(), toi.n2_ref(), self);
                path.normal_contact = *toi.n1_ref();
                path.energy = path.energy - sn.refl.atenuation;
                path.mix_coef = sn.refl.mix;

            }
        }
    }

    pub fn trace_light(&self, path: &mut LightPath) {
        let cast = self.world.cast_ray(&path.ray, &|b, r| { b.cast(r).map(|(t, n, u)| (t, (t, n, u))) });
        match cast {
            None     => (),
            Some((_, toi, sn)) => {
                let inter = path.ray.orig + path.ray.dir * *toi.n0_ref();

                path.last_intersection = inter;
                sn.material.compute_for_light_path(path, &inter, toi.n1_ref(), toi.n2_ref(), self);
                path.mix_coef = sn.refl.mix;
                sn.refl.compute_for_light_path(path, &inter, toi.n1_ref(), toi.n2_ref(), self);
            }
        };
    }
}
