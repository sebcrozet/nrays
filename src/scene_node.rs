use ncollide::geom::Geom;
use material::Material;

pub struct SceneNode<N, V, Vlessi, M> {
    materials: ~[@Material<N, V, Vlessi, M>],
    transform: M,
    geometry:  Geom<N, V, M>
}

impl<N, V, Vlessi, M> SceneNode<N, V, Vlessi, M> {
    pub fn new(materials: ~[@Material<N, V, Vlessi, M>],
               transform: M,
               geometry:  Geom<N, V, M>)
               -> SceneNode<N, V, Vlessi, M> {
        SceneNode {
            materials: materials,
            transform: transform,
            geometry:  geometry
        }
    }
}
