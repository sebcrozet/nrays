use std::num::Zero;
use std::vec;
use nalgebra::vec::{AlgebraicVecExt, VecExt, Dim, Vec4, Vec, VecCast};
use nalgebra::mat::{Translation, Rotate, Transform, AbsoluteRotate};
use ncollide::bounding_volume::{AABB, HasAABB};
use ncollide::partitioning::bvt;
use ncollide::partitioning::bvt::BVT;
use ncollide::partitioning::bvt_visitor::BVTVisitor;
// use ncollide::partitioning::bvt_visitor::RayInterferencesCollector;
use ncollide::ray::{Ray, RayCast, RayCastWithTransform};
use scene_node::SceneNode;
use image::Image;

pub struct Scene<N, V, Vi, M> {
    priv world: BVT<@SceneNode<N, V, M>, AABB<N, V>>
}

impl<N:  'static + NumCast + Primitive + Algebraic + Signed + Float + ToStr,
     V:  'static + AlgebraicVecExt<N> + Clone + ToStr,
     Vi: VecExt<uint> + VecCast<V> + Dim + Clone + ToStr,
     M:  Translation<V> + Rotate<V> + Transform<V> + Mul<M, M> + AbsoluteRotate<V> + Dim>
Scene<N, V, Vi, M> {
    pub fn new(nodes: ~[@SceneNode<N, V, M>]) -> Scene<N, V, Vi, M> {
        let mut nodes_w_bvs = ~[];

        for n in nodes.move_iter() {
            let aabb = n.geometry.aabb(&n.transform);
            nodes_w_bvs.push((n, aabb));
        }

        let bvt = BVT::new_with_partitioner(nodes_w_bvs, bvt::dim_pow_2_aabb_partitioner);

        Scene {
            world: bvt
        }
    }

    pub fn render(&self, eye: &V, at: &V, extents: &V, resolution: &Vi, projection: &M) -> Image<Vi> {
        let mut npixels = 1;

        for i in resolution.iter() {
            npixels = npixels * *i;
        }

        let mut curr: Vi = Zero::zero();

        // Sample a rectangular n-1 surface (with n the rendering dimension):
        //   * a rectangle for 3d rendering.
        //   * a cube for 4d rendering.
        //   * an hypercube for 5d rendering.
        //   * etc
        let principal_dir = (at - *eye).normalized();
        let mut pixels    = vec::with_capacity(npixels);

        for _ in range(0u, npixels) {
            // curr contains the index of the current sample point.
            let ray_pos = VecCast::from(curr.clone()); // FIXME: this is obviously wrong!
            let ray_dir = principal_dir.clone();

            pixels.push(self.trace(&Ray::new(ray_pos, ray_dir)));

            for j in range(0u, Dim::dim(None::<V>)) {
                let inc = curr.at(j) + 1;

                if inc == resolution.at(j) {
                    curr.set(j, 0);
                }
                else {
                    curr.set(j, inc);
                    break;
                }
            }
        }

        Image::new(resolution.clone(), pixels)
    }

    pub fn trace(&self, ray: &Ray<V>) -> Vec4<f64> {
        let mut interferences: ~[@SceneNode<N, V, M>] = ~[];

        {
            let mut collector = RayInterferencesCollector::new(ray, &mut interferences);
            self.world.visit(&mut collector);
        }

        /*
         * Nothing fancy at the moment: simply return the color of the first object hit by the ray.
         */
        let mut intersection = None;
        let mut mintoi: N    = Bounded::max_value();
        for i in interferences.iter() {
            let toi = i.geometry.toi_with_transform_and_ray(&i.transform, ray);

            match toi {
                None => { },
                Some(toi) => {
                    if toi < mintoi {
                        mintoi = toi;
                        intersection = Some(i);
                    }
                }
            }
        }

        match intersection {
            None    => Vec4::new(0.0, 0.0, 0.0, 0.0),
            Some(i) => i.material.diffuse_color.clone()
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
pub struct RayInterferencesCollector<'self, V, B> {
    priv ray:       &'self Ray<V>,
    priv collector: &'self mut ~[B]
}

impl<'self, V, B> RayInterferencesCollector<'self, V, B> {
    /// Creates a new `RayInterferencesCollector`.
    #[inline]
    pub fn new(ray:    &'self Ray<V>,
               buffer: &'self mut ~[B])
               -> RayInterferencesCollector<'self, V, B> {
        RayInterferencesCollector {
            ray:       ray,
            collector: buffer
        }
    }
}

impl<'self,
     N,
     V:  Vec<N>,
     B:  Clone,
     BV: RayCast<N, V>>
BVTVisitor<B, BV> for RayInterferencesCollector<'self, V, B> {
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
