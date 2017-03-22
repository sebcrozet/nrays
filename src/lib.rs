#![warn(non_camel_case_types)]

extern crate num;
extern crate num_cpus;
extern crate rand;
extern crate nalgebra as na;
extern crate ncollide;
extern crate png;
extern crate stb_image;

pub mod scene_node;
pub mod scene;
pub mod image;
pub mod light;
pub mod material;
pub mod ray_with_energy;

pub mod phong_material;
pub mod texture2d;

pub mod normal_material;
pub mod uv_material;

pub mod obj;
pub mod mtl;
pub mod mesh;

/// Type aliases for mathematical types.
pub mod math {
    use na::{Point3, Vector3, Matrix3, Rotation3, Isometry3};

    /// The scalar type.
    pub type Scalar = f64;

    /// The point type.
    pub type Point = Point3<Scalar>;

    /// The vector type.
    pub type Vect = Vector3<Scalar>;

    /// The orientation type.
    pub type Orientation = Vector3<Scalar>;

    /// The transformation matrix type.
    pub type Matrix = Isometry3<Scalar>;

    /// The rotation matrix type.
    pub type RotationMatrix = Rotation3<Scalar>;

    /// The inertia tensor type.
    pub type AngularInertia = Matrix3<Scalar>;
}
