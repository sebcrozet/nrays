use ncollide::geom::Geom;
use nalgebra::vec::Vec4;

pub struct Material {
    diffuse_color: Vec4<f64>
}

impl Material {
    pub fn new(diffuse_color: Vec4<f64>) -> Material {
        Material {
            diffuse_color: diffuse_color
        }
    }
}

pub struct SceneNode<N, V, M> {
    material:  Material,
    transform: M,
    geometry:  Geom<N, V, M>
}

impl<N, V, M> SceneNode<N, V, M> {
    pub fn new(material: Material, transform: M, geometry: Geom<N, V, M>) -> SceneNode<N, V, M> {
        SceneNode {
            material:  material,
            transform: transform,
            geometry:  geometry
        }
    }
}
