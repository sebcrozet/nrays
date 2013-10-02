use ncollide::geom::Geom;
use nalgebra::vec::Vec4;

pub struct Material {
    diffuse_color: Vec4<f64>
}

pub struct SceneNode<N, V, M> {
    material:  Material,
    transform: M,
    geometry:  Geom<N, V, M>
}
