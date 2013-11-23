use ncollide::ray::RayCastWithTransform;
use ncollide::bounding_volume::{HasAABB, AABB};
use material::Material;

pub struct SceneNode<N, V, Vlessi, M> {
    materials: ~[@Material<N, V, Vlessi, M>],
    transform: M,
    geometry:  @RayCastWithTransform<N, V, M>,
    aabb:      AABB<N, V>
}

impl<N, V, Vlessi, M> SceneNode<N, V, Vlessi, M> {
    pub fn new<G: 'static + RayCastWithTransform<N, V, M> + HasAABB<N, V, M>>(
               materials: ~[@Material<N, V, Vlessi, M>],
               transform: M,
               geometry:  @G)
               -> SceneNode<N, V, Vlessi, M> {
        SceneNode {
            materials: materials,
            geometry:  geometry as @RayCastWithTransform<N, V, M>,
            aabb:      geometry.aabb(&transform),
            transform: transform
        }
    }
}
