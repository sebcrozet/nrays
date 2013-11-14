use ncollide::geom::Geom;
use material::Material;

pub struct SceneNode<N, V, Vlessi, M> {
    material:  @Material<N, V, Vlessi, M>,
    transform: M,
    geometry:  Geom<N, V, M>
}

impl<N, V, Vlessi, M> SceneNode<N, V, Vlessi, M> {
    pub fn new(material:  @Material<N, V, Vlessi, M>,
               transform: M,
               geometry:  Geom<N, V, M>)
               -> SceneNode<N, V, Vlessi, M> {
        SceneNode {
            material:  material,
            transform: transform,
            geometry:  geometry
        }
    }
}
