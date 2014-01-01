use std::vec;
use nalgebra::na::{Vec3, Dim, Iterable, Indexable};
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

        for i in interferences.iter() {
            let toi = i.geometry.toi_with_transform_and_ray(&i.transform, ray);

            match toi {
                None    => { },
                Some(t) => if t <= maxtoi { return true }
            }
        }

        false
    }

    pub fn trace(&self, ray: &RayWithEnergy) -> Vec3<f32> {
        // FIXME: avoid allocations
        let mut interferences: ~[@SceneNode] = ~[];

        {
            let mut collector = RayInterferencesCollector::new(&ray.ray, &mut interferences);
            self.world.visit(&mut collector);
        }

        let mut intersection = None;
        let mut mintoi:    N                 = Bounded::max_value();
        let mut minnormal: V                 = na::zero();
        let mut minuvs:    Option<(N, N, N)> = None;
        for i in interferences.iter() {
            let toi = i.cast(&ray.ray);

            match toi {
                None => { },
                Some((toi, normal, uvs)) => {
                    if toi < mintoi {
                        mintoi       = toi;
                        minnormal    = normal;
                        minuvs       = uvs;
                        intersection = Some(i);
                    }
                }
            }
        }

        match intersection {
            None     => Vec3::new(0.0, 0.0, 0.0),
            Some(sn) => {
                let inter = ray.ray.orig + ray.ray.dir * mintoi;

                let obj  = sn.material.compute(ray, &inter, &minnormal, &minuvs, self);
                let refl = sn.refl.compute(ray, &inter, &minnormal, &minuvs, self);

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
