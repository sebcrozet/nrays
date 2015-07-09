//! Data structure of a scene node geometry.

use std::sync::Arc;
use std::iter;
use num::Zero;
use na::{Vec3, Pnt2, Pnt3};
use na;

pub type Coord  = Pnt3<f32>;
pub type Normal = Vec3<f32>;
pub type UV     = Pnt2<f32>;
pub type Vertex = usize;
pub type Face   = Pnt3<Vertex>;

pub enum StorageLocation<T> {
    SharedImmutable(Arc<T>),
    NotShared(T)
}

impl<T: Clone + Sync + Send> StorageLocation<T> {
    pub fn unwrap(&self) -> T {
        match *self {
            StorageLocation::SharedImmutable(ref a) => (**a).clone(),
            StorageLocation::NotShared(ref t)       => t.clone()
        }
    }
}

impl<T: Sync + Send + Clone> Clone for StorageLocation<T> {
    fn clone(&self) -> StorageLocation<T> {
        match *self {
            StorageLocation::SharedImmutable(ref t) => StorageLocation::SharedImmutable(t.clone()),
            StorageLocation::NotShared(ref t)       => StorageLocation::NotShared(t.clone())
        }
    }
}


impl<T: Sync + Send> StorageLocation<T> {
    pub fn new(t: T, shared: bool) -> StorageLocation<T> {
        if shared {
            StorageLocation::SharedImmutable(Arc::new(t))
        }
        else {
            StorageLocation::NotShared(t)
        }
    }

    pub fn get<'r>(&'r self) -> &'r T {
        match *self {
            StorageLocation::SharedImmutable(ref s) => &**s,
            StorageLocation::NotShared(ref s)       => s
        }
    }

    pub fn is_shared(&self) -> bool {
        match *self {
            StorageLocation::SharedImmutable(_) => true,
            StorageLocation::NotShared(_)       => false
        }
    }
}

impl<T: Sync + Send + Clone> StorageLocation<T> {
    pub fn write_cow<'r, F: Fn(&mut T) -> ()>(&'r mut self, f: F) {
        match *self {
            StorageLocation::SharedImmutable(ref mut s) => {
                let mut cpy = (**s).clone();
                f(&mut cpy);

                *s = Arc::new(cpy);
            },
            StorageLocation::NotShared(ref mut s) => f(s)
        }
    }
}

/// A Mesh contains all geometric data of a mesh: vertex buffer, index buffer, normals and uvs.
/// It also contains the GPU location of those buffers.
pub struct Mesh {
    coords:  StorageLocation<Vec<Coord>>,
    faces:   StorageLocation<Vec<Face>>,
    normals: StorageLocation<Vec<Normal>>,
    uvs:     StorageLocation<Vec<UV>>,
}

impl Mesh {
    /// Creates a new mesh. Arguments set to `None` are automatically computed.
    pub fn new(coords:          StorageLocation<Vec<Coord>>,
               faces:           StorageLocation<Vec<Face>>,
               normals:         Option<StorageLocation<Vec<Normal>>>,
               uvs:             Option<StorageLocation<Vec<UV>>>)
               -> Mesh {
        let normals = match normals {
            Some(ns) => ns,
            None     => {
                let normals = compute_normals_array(&coords.get()[..], &faces.get()[..]);
                StorageLocation::new(normals, coords.is_shared())
            }
        };

        let uvs = match uvs {
            Some(us) => us,
            None     => {
                let uvs = iter::repeat(na::orig()).take(coords.get().len()).collect();
                StorageLocation::new(uvs, coords.is_shared())
            }
        };

        Mesh {
            coords:  coords,
            faces:   faces,
            normals: normals,
            uvs:     uvs
        }
    }

    /// Number of points needed to draw this mesh.
    pub fn num_pts(&self) -> usize {
        self.faces.get().len() * 3
    }

    /// Recompute this mesh normals.
    pub fn recompute_normals(&mut self) {
        panic!("Review that.")
        /*
        self.normals.write_cow(
            |normals| compute_normals(self.coords.get().as_slice(), self.faces.get().as_slice(), normals)
        )
        */
    }

    /// This mesh faces.
    pub fn faces(&self) -> &[Face] {
        &self.faces.get()[..]
    }

    /// This mesh faces.
    pub fn mut_faces(&mut self) -> &mut StorageLocation<Vec<Face>> {
        &mut self.faces
    }

    /// This mesh normals.
    pub fn normals(&self) -> &[Normal] {
        &self.normals.get()[..]
    }

    /// This mesh normals.
    pub fn mut_normals(&mut self) -> &mut StorageLocation<Vec<Normal>> {
        &mut self.normals
    }

    /// This mesh vertices coordinates.
    pub fn coords(&self) -> &[Coord] {
        &self.coords.get()[..]
    }

    /// This mesh vertices coordinates.
    pub fn mut_coords(&mut self) -> &mut StorageLocation<Vec<Coord>> {
        &mut self.coords
    }

    /// This mesh texture coordinates.
    pub fn uvs(&self) -> &[UV] {
        &self.uvs.get()[..]
    }

    /// This mesh texture coordinates.
    pub fn mut_uvs<'r>(&'r mut self) -> &'r mut StorageLocation<Vec<UV>> {
        &mut self.uvs
    }
}

/// Comutes normals from a set of faces.
pub fn compute_normals_array(coordinates: &[Coord],
                             faces:       &[Face])
                             -> Vec<Normal> {
    let mut res = Vec::new();

    compute_normals(coordinates, faces, &mut res);

    res
}

/// Comutes normals from a set of faces.
pub fn compute_normals(coordinates: &[Coord],
                       faces:       &[Face],
                       normals:     &mut Vec<Normal>) {
    let mut divisor: Vec<f32> = iter::repeat(0.0f32).take(coordinates.len()).collect();

    // Grow the output buffer if it is too small.
    normals.clear();
    normals.extend(iter::repeat(na::zero::<Vec3<f32>>()).take(coordinates.len()));

    // Accumulate normals ...
    for f in faces.iter() {
        let edge1  = coordinates[f.y as usize] - coordinates[f.x as usize];
        let edge2  = coordinates[f.z as usize] - coordinates[f.x as usize];
        let cross  = na::cross(&edge1, &edge2);
        let normal;

        if !cross.is_zero() {
            normal = na::normalize(&cross)
        }
        else {
            normal = cross
        }

        let normals = &mut normals[..];
        let divisor = &mut divisor[..];
        normals[f.x as usize] = normals[f.x as usize] + normal;
        normals[f.y as usize] = normals[f.y as usize] + normal;
        normals[f.z as usize] = normals[f.z as usize] + normal;

        divisor[f.x as usize] = divisor[f.x as usize] + 1.0;
        divisor[f.y as usize] = divisor[f.y as usize] + 1.0;
        divisor[f.z as usize] = divisor[f.z as usize] + 1.0;
    }

    // ... and compute the mean
    for (n, divisor) in normals.iter_mut().zip(divisor.iter()) {
        *n = *n / *divisor
    }
}
