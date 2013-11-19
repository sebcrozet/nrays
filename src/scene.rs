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
            let aabb = n.geometry.aabb(&n.transform);
            nodes_w_bvs.push((n, aabb));
        }

        let bvt = BVT::new_with_partitioner(nodes_w_bvs, bvt::dim_pow_2_aabb_partitioner);

        Scene {
            lights: lights,
            world:  bvt
        }
    }

    pub fn render(&self, resolution: &Vless, unproject: &fn(&Vless) -> Ray<V>) -> Image<Vless> {
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
            let c = self.trace(&unproject(&curr));
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

    pub fn trace(&self, ray: &Ray<V>) -> Vec4<f32> {
        let mut interferences: ~[@SceneNode<N, V, Vless, M>] = ~[];

        {
            let mut collector = RayInterferencesCollector::new(ray, &mut interferences);
            self.world.visit(&mut collector);
        }

        let mut intersection = None;
        let mut mintoi:    N = Bounded::max_value();
        let mut minnormal: V = na::zero();
        for i in interferences.iter() {
            let toi = i.geometry.toi_and_normal_with_transform_and_ray(&i.transform, ray);

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
                let inter = ray.orig + ray.dir * mintoi;
                sn.material.compute(ray, &inter, &minnormal, self)
                // // FIXME: create a shader system to handle lighting
                // // self.compute_lighting_from(*i, &(ray.orig + ray.dir * mintoi), &minnormal)
                // let cos_angle = na::dot(&ray.dir, &minnormal) / (na::norm(&ray.dir) * na::norm(&minnormal));

                // // FIXME: we use NumCast here since the structs::spec::f64Cast trait is private…
                // // Find a way to fix that on nalgebra.
                // let color: Vec3<f32> = Vec3::new(NumCast::from(-cos_angle).unwrap(), 0., 0.);
                // na::to_homogeneous(&color)
            }
        }
    }

    pub fn compute_lighting_from(&self, _: @SceneNode<N, V, Vless, M>, point: &V, _: &V) -> Vec4<f32> {
        let mut interferences: ~[@SceneNode<N, V, Vless, M>] = ~[];

        let mut color = Vec4::new(0.0f32, 0.0, 0.0, 1.0);

        'loop: for l in self.lights.iter() {
            interferences.clear();

            let ray = Ray::new(l.pos.clone(), *point - l.pos);

            {
                let mut collector = RayInterferencesCollector::new(&ray, &mut interferences);
                self.world.visit(&mut collector);
            }

            for i in interferences.iter() {
                if true { // !managed::ptr_eq(*i, node)
                    let toi = i.geometry.toi_with_transform_and_ray(&i.transform, &ray);
                    match toi {
                        None      => { },
                        Some(_/* toi */) => {
                           // if toi < na::cast(0.75 - 0.00001) {
                           //     continue 'loop;
                           // }
                        }
                    }
                }
            }

            // FIXME: we use NumCast here since the structs::spec::f64Cast trait is private…
            // Find a way to fix that on nalgebra.
            let distance_to_light: f32 = NumCast::from(na::norm(&(*point - l.pos))).expect("(0) Conversion failed.");
            color = color + na::to_homogeneous(&(l.color * (1.0 - distance_to_light / 5.0)));
        }

        color
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
