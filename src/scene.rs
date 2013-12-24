use std::vec;
use nalgebra::na::{Cast, Vec, VecExt, AlgebraicVecExt, AbsoluteRotate, Dim, Transform, Rotate,
                   Translation, Vec4};
use nalgebra::na;
use ncollide::bounding_volume::{AABB, HasAABB};
use ncollide::partitioning::bvt;
use ncollide::partitioning::bvt::BVT;
use ncollide::partitioning::bvt_visitor::BVTVisitor;
// use ncollide::partitioning::bvt_visitor::RayInterferencesCollector;
use ncollide::ray::{Ray, RayCast, RayCastWithTransform};
use ray_with_energy::RayWithEnergy;
use scene_node::SceneNode;
use image::Image;
use light::Light;

pub struct Scene<N, V, Vless, M> {
    priv lights: ~[Light<V>],
    priv world:  BVT<@SceneNode<N, V, Vless, M>, AABB<N, V>>
}

impl<N:     'static + Cast<f32> + Send + Freeze + NumCast + Primitive + Algebraic + Signed + Float,
     V:     'static + AlgebraicVecExt<N> + Send + Freeze + Clone,
     Vless: VecExt<N> + Dim + Clone,
     M:     Translation<V> + Rotate<V> + Send + Freeze + Transform<V> + Mul<M, M> + AbsoluteRotate<V> + Dim>
Scene<N, V, Vless, M> {
    pub fn new(nodes:  ~[@SceneNode<N, V, Vless, M>],
               lights: ~[Light<V>]) -> Scene<N, V, Vless, M> {
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
    pub fn lights<'a>(&'a self) -> &'a [Light<V>] {
        let res: &'a [Light<V>] = self.lights;

        res
    }

    pub fn render(&self, resolution: &Vless, unproject: |&Vless| -> Ray<V>) -> Image<Vless> {
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

    pub fn intersects_ray(&self, ray: &Ray<V>) -> bool {
        // FIXME: avoid allocations
        let mut interferences: ~[@SceneNode<N, V, Vless, M>] = ~[];

        {
            let mut collector = RayInterferencesCollector::new(ray, &mut interferences);
            self.world.visit(&mut collector);
        }

        for i in interferences.iter() {
            if i.geometry.intersects_with_transform_and_ray(&i.transform, ray) {
                return true;
            }
        }

        false
    }

    pub fn trace(&self, ray: &RayWithEnergy<V>) -> Vec4<f32> {
        // FIXME: avoid allocations
        let mut interferences: ~[@SceneNode<N, V, Vless, M>] = ~[];

        {
            let mut collector = RayInterferencesCollector::new(&ray.ray, &mut interferences);
            self.world.visit(&mut collector);
        }

        let mut intersection = None;
        let mut mintoi:    N = Bounded::max_value();
        let mut minnormal: V = na::zero();
        for i in interferences.iter() {
            let toi = i.geometry.toi_and_normal_with_transform_and_ray(&i.transform, &ray.ray);

            match toi {
                None => { },
                Some((toi, normal)) => {
                    if toi < mintoi {
                        mintoi       = toi;
                        minnormal    = normal;
                        intersection = Some(i);
                    }
                }
            }
        }

        match intersection {
            None     => Vec4::new(0.0, 0.0, 0.0, 1.0),
            Some(sn) => {
                let inter = ray.ray.orig + ray.ray.dir * mintoi;

                let mut color: Vec4<f32> = na::zero();
                for m in sn.materials.iter() {
                    color = color + m.compute(ray, &inter, &minnormal, self);
                }

                color
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
pub struct RayInterferencesCollector<'a, V, B> {
    priv ray:       &'a Ray<V>,
    priv collector: &'a mut ~[B]
}

impl<'a, V, B> RayInterferencesCollector<'a, V, B> {
    /// Creates a new `RayInterferencesCollector`.
    #[inline]
    pub fn new(ray:    &'a Ray<V>,
               buffer: &'a mut ~[B])
               -> RayInterferencesCollector<'a, V, B> {
        RayInterferencesCollector {
            ray:       ray,
            collector: buffer
        }
    }
}

impl<'a,
     N,
     V:  Vec<N>,
     B:  Clone,
     BV: RayCast<N, V>>
BVTVisitor<B, BV> for RayInterferencesCollector<'a, V, B> {
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
