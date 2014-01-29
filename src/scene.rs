use std::num::Zero;
use std::vec;
use std::rt;
use extra::arc::{Arc, RWArc};
use nalgebra::na::{Vec3, Vec4, Mat4, Norm};
use nalgebra::na;
use nalgebra::na::Indexable;
use ncollide::bounding_volume::{AABB, HasAABB};
use ncollide::partitioning::BVT;
// use ncollide::partitioning::bvt_visitor::RayInterferencesCollector;
use ncollide::ray::{Ray, RayIntersection};
use ncollide::math::{N, V};
use material::Material;
use ray_with_energy::RayWithEnergy;
use intersection::Intersection;
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


    // Light rays


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
            let nb_bounces = 4;

            do spawn() {
                let mut pxs = vec::with_capacity(up_limit - low_limit);
                let mut paths : ~[~[Intersection]] = ~[];

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

                    pxs.push(tot_c / na::cast::<uint, f32>(ray_per_pixel));
                }

                pixels.write(|p| {
                    for ipt in range(low_limit, up_limit) {
                        let j = ipt / resx;
                        let i = ipt - j * resx;

                        p[i + j * resx as uint] = pxs[ipt - low_limit];
                    }
                });
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

    pub fn add_biderectional_lights(&mut self) {

        let mut extra_lights: ~[Light] = ~[];
        for light in self.lights.iter() {

            light.sample(|pos| {
                         let mut ldir = pos - light.pos;
                         ldir.normalize();
                         let mut ray = Ray::new(light.pos, ldir);
                         self.trace_lights(&RayWithEnergy::new(light.pos, ldir), &mut extra_lights);

            });
        }

        for light in extra_lights.iter() {
            self.lights.push(*light);
        }
        println!("Number of additional lights : {}.", self.lights.len());
    }

    #[inline]
    pub fn lights<'a>(&'a self) -> &'a [Light] {
        let res: &'a [Light] = self.lights;

        res
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

    pub fn intersects_ray(&self, ray: &Ray, maxtoi: N) -> Option<Vec3<f32>> {
        let mut filter = Vec3::new(1.0, 1.0, 1.0);

        let inter = self.world.cast_ray(ray, &|b, r| {
            // this will make a very coarce approximation of the transparent shadow.
            // (occluded objects might show up)
            match b.get().cast(r) {
                Some(t) => {
                    if t.toi <= maxtoi {
                        let color = b.get().material.get().ambiant(&(ray.orig + ray.dir * t.toi), &t.normal, &uvs(&t));

                        let alpha = color.w * b.get().alpha;

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

    pub fn integrate(&self, path: &[Intersection]) -> Vec3<f32> {
        let mut color = Vec3::new(0.0f32, 0.0, 0.0);
        for i in path.rev_iter() {
            let col = Vec3::new(i.color.at(0), i.color.at(1), i.color.at(2));
            color = color + col * (i.intensity as f32);
        }
        color
    }

    pub fn trace(&self, ray: &RayWithEnergy) -> Vec3<f32> {
        let cast = self.world.cast_ray(&ray.ray, &|b, r| { b.get().cast(r).map(|inter| (inter.toi, inter)) });

        match cast {
            None                 => na::zero(),
            Some((_, inter, sn)) => {
                let bsn       = sn.get();
                let pt        = ray.ray.orig + ray.ray.dir * inter.toi;
                let obj       = bsn.material.get().compute(ray, &pt, &inter.normal, &uvs(&inter), self);
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
    fn trace_reflection(&self, mix: f32, attenuation: f32, ray: &RayWithEnergy, pt: &V, normal: &V) -> Vec3<f32> {
        if !mix.is_zero() && ray.energy > 0.1 {
            let nproj      = normal * na::dot(&ray.ray.dir, normal);
            let rdir       = ray.ray.dir - nproj * na::cast::<f32, N>(2.0);
            let new_energy = ray.energy - attenuation;

            self.trace(
                &RayWithEnergy::new_with_energy(
                    pt + rdir * na::cast::<f32, N>(0.001),
                    rdir,
                    ray.refr.clone(),
                    new_energy))
        }
        else {
            na::zero()
        }
    }

    #[inline]
    fn trace_refraction(&self, alpha: f32, coeff: N, ray: &RayWithEnergy, pt: &V, normal: &V) -> Vec3<f32>  {
        if alpha != 1.0 {
            let n1;
            let n2;

            if ray.refr == na::cast(1.0) {
                n1 = na::cast(1.0);
                n2 = coeff;
            }
            else {
                n1 = coeff;
                n2 = na::cast(1.0);
            }

            let dir_along_normal = normal * na::dot(&ray.ray.dir, normal);
            let tangent = ray.ray.dir - dir_along_normal;
            let new_dir = na::normalize(&(dir_along_normal + tangent * (n2 / n1)));
            let new_pt  = pt + new_dir * 0.001;

            self.trace(&RayWithEnergy::new_with_energy(new_pt, new_dir, n2, ray.energy))
        }
        else {
            na::zero()
        }
    }

    pub fn trace_lights(&self, ray: &RayWithEnergy, lights: &mut ~[Light]) -> Vec3<f32> {
        let cast = self.world.cast_ray(&ray.ray, &|b, r| { b.get().cast(r).map(|inter| (inter.toi, inter)) });

        match cast {
            None                 => na::zero(),
            Some((_, inter, sn)) => {
                let bsn       = sn.get();
                let pt        = ray.ray.orig + ray.ray.dir * inter.toi;
                let obj       = bsn.material.get().compute(ray, &pt, &inter.normal, &uvs(&inter), self);
                let refl      = self.trace_reflection_lights(bsn.refl_mix, bsn.refl_atenuation, ray,
                                                             &pt, &inter.normal, lights);

                let alpha     = obj.w * bsn.alpha;
                let obj_color = Vec3::new(obj.x, obj.y, obj.z) * (1.0 - bsn.refl_mix) + refl * bsn.refl_mix;
                let refr      = self.trace_refraction_lights(alpha, bsn.refr_coeff, ray, &pt,
                                                      &inter.normal, lights);

                if alpha == 1.0 {
                    let color = Vec3::new(obj_color.x, obj_color.y, obj_color.z);
                    lights.push(Light::new(pt, 0.0, 1, color));
                    color
                }
                else {
                    let obj_color = Vec3::new(obj_color.x, obj_color.y, obj_color.z);
                    let refr      = Vec3::new(refr.x, refr.y, refr.z);

                    let color = obj_color * alpha + refr * (1.0 - alpha);
                    lights.push(Light::new(pt, 0.0, 1, color));
                    color
                }
            }
        }
    }


    #[inline]
    fn trace_reflection_lights(&self, mix: f32, attenuation: f32, ray: &RayWithEnergy, pt: &V, normal: &V,
                        lights: &mut ~[Light]) -> Vec3<f32> {
        if !mix.is_zero() && ray.energy > 0.1 {
            let nproj      = normal * na::dot(&ray.ray.dir, normal);
            let rdir       = ray.ray.dir - nproj * na::cast::<f32, N>(2.0);
            let new_energy = ray.energy - attenuation;

            self.trace_lights(
                &RayWithEnergy::new_with_energy(
                    pt + rdir * na::cast::<f32, N>(0.001),
                    rdir,
                    ray.refr.clone(),
                    new_energy), lights)
        }
        else {
            na::zero()
        }
    }

    #[inline]
    fn trace_refraction_lights(&self, alpha: f32, coeff: N, ray: &RayWithEnergy, pt: &V, normal: &V,
                               lights: &mut ~[Light])  -> Vec3<f32> {
        if alpha != 1.0 {
            let n1;
            let n2;

            if ray.refr == na::cast(1.0) {
                n1 = na::cast(1.0);
                n2 = coeff;
            }
            else {
                n1 = coeff;
                n2 = na::cast(1.0);
            }

            let dir_along_normal = normal * na::dot(&ray.ray.dir, normal);
            let tangent = ray.ray.dir - dir_along_normal;
            let new_dir = na::normalize(&(dir_along_normal + tangent * (n2 / n1)));
            let new_pt  = pt + new_dir * 0.001;

            self.trace_lights(&RayWithEnergy::new_with_energy(new_pt, new_dir, n2, ray.energy), lights)
        }
        else {
            na::zero()
        }
    }
}

#[cfg(dim3)]
fn uvs(i: &RayIntersection) -> Option<Vec3<N>> {
    i.uvs.clone()
}

#[cfg(not(dim3))]
fn uvs(i: &RayIntersection) -> Option<Vec3<N>> {
    None
}
