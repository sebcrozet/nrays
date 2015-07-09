#![warn(non_camel_case_types)]

extern crate num;
extern crate num_cpus;
extern crate rand;
extern crate nalgebra as na;
extern crate ncollide;
extern crate png;
extern crate stb_image;
extern crate byteorder;

pub mod scene_node;
pub mod scene;
pub mod image;
pub mod light;
pub mod material;
pub mod ray_with_energy;

pub mod phong_material;
pub mod texture2d;

// NOTE: Those cannot be used on 4d ray cast.

// pub mod normal_material;
// pub mod uv_material;

pub mod obj;
pub mod mtl;
pub mod mesh;

/// Compilation flags dependent aliases for mathematical types.
pub mod math {
    use na::{Pnt4, Vec4, Mat4, Rot4, Iso4};

    /// The scalar type.
    #[cfg(feature = "f32")]
    pub type Scalar = f32;

    /// The scalar type.
    #[cfg(feature = "f64")]
    pub type Scalar = f64;

    /// The point type.
    pub type Point = Pnt4<Scalar>;

    /// The vector type.
    pub type Vect = Vec4<Scalar>;

    /// The orientation type.
    pub type Orientation = Vec4<Scalar>;

    /// The transformation matrix type.
    pub type Matrix = Iso4<Scalar>;

    /// The rotation matrix type.
    pub type RotationMatrix = Rot4<Scalar>;

    /// The inertia tensor type.
    pub type AngularInertia = Mat4<Scalar>;
}
