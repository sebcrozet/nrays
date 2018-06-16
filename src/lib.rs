#![warn(non_camel_case_types)]

extern crate nalgebra as na;
extern crate ncollide3d;
extern crate num_cpus;
extern crate num_traits as num;
extern crate png;
extern crate rand;
extern crate stb_image;

pub mod image;
pub mod light;
pub mod material;
pub mod ray_with_energy;
pub mod scene;
pub mod scene_node;

pub mod phong_material;
pub mod texture2d;

pub mod normal_material;
pub mod uv_material;

pub mod mesh;
pub mod mtl;
pub mod obj;

/// Type aliases for mathematical types.
pub mod math {
    use na::{Isometry3, Matrix3, Point3, Rotation3, Vector3};

    /// The scalar type.
    pub type Scalar = f64;

    /// The point type.
    pub type Point = Point3<Scalar>;

    /// The vector type.
    pub type Vect = Vector3<Scalar>;

    /// The orientation type.
    pub type Orientation = Vector3<Scalar>;

    /// The transformation matrix type.
    pub type Isometry = Isometry3<Scalar>;

    /// The rotation matrix type.
    pub type RotationMatrix = Rotation3<Scalar>;

    /// The inertia tensor type.
    pub type AngularInertia = Matrix3<Scalar>;
}
