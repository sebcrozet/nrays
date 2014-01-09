use std::vec;
use std::io::stdio;
use nalgebra::na::{Vec3, Iterable};
use nalgebra::na;
use ncollide::bounding_volume::{AABB, HasAABB};
use ncollide::partitioning::bvt;
use ncollide::partitioning::bvt::BVT;
use ncollide::partitioning::bvt_visitor::BVTVisitor;
// use ncollide::partitioning::bvt_visitor::RayInterferencesCollector;
use ncollide::ray::{Ray, RayCast, RayCastWithTransform};
use ncollide::math::{N, V};
use material::Material;
use ray_with_energy::RayWithEnergy;
use scene_node::SceneNode;
use image::Image;
use light::Light;

#[cfg(dim4)]
use nalgebra::na::{Dim, Indexable};

#[cfg(dim3)]
use std::rand;
#[cfg(dim3)]
use nalgebra::na::Vec2;

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

        let bvt = BVT::new_with_partitioner(nodes_w_bvs, bvt::kdtree_partitioner);

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

        for j in range(0u, na::cast(resolution.y)) {
            for i in range(0u, na::cast(resolution.x)) {
                let mut tot_c: Vec3<f32> = na::zero();

                for _ in range(0u, ray_per_pixel) {
                    rays_done = rays_done + 1;

                    let perturbation  = (rand::random::<Vless>() - na::cast::<f32, N>(0.5)) * window_width;
                    let orig: Vec2<N> = Vec2::new(na::cast(i), na::cast(j)) + perturbation;

                    let ray = unproject(&orig);
                    let c   = self.trace(&RayWithEnergy::new(ray.orig.clone(), ray.dir));
                    print!("{} / {} − {}%\r", rays_done, nrays, ((rays_done as f64) / nrays * 100.0) as int);
                    stdio::flush();

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
        // FIXME: avoid allocations
        let mut interferences: ~[@SceneNode] = ~[];

        {
            let mut collector = RayInterferencesCollector::new(ray, &mut interferences);
            self.world.visit(&mut collector);
        }

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
}

/*
 * ----------------------------------------------------------------------------------------------
 *
 * XXX: This is an exact duplicate of lib/ncollide/src/partitioning/bvt_visitor.rs#RayInterferencesCollector
 * This does not compile cross-crate (ICE).
 *
 * ----------------------------------------------------------------------------------------------
 */
/// Bounding Volume Tree visitor collecting interferences with a given ray.
pub struct RayInterferencesCollector<'a, B> {
    priv ray:       &'a Ray,
    priv collector: &'a mut ~[B]
}

impl<'a, B> RayInterferencesCollector<'a, B> {
    /// Creates a new `RayInterferencesCollector`.
    #[inline]
    pub fn new(ray: &'a Ray, buffer: &'a mut ~[B])
               -> RayInterferencesCollector<'a, B> {
        RayInterferencesCollector {
            ray:       ray,
            collector: buffer
        }
    }
}

impl<'a, B: Clone, BV: RayCast> BVTVisitor<B, BV> for RayInterferencesCollector<'a, B> {
    #[inline]
    fn visit_internal(&mut self, bv: &BV) -> bool {
        bv.intersects_ray(self.ray)
    }

    #[inline]
    fn visit_leaf(&mut self, b: &B, bv: &BV) {
        if bv.intersects_ray(self.ray) {
            self.collector.push(b.clone())
        }
    }
}
