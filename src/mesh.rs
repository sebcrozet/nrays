//! Data structure of a scene node geometry.

use std::sync::Arc;
use std::num::Zero;
use nalgebra::na::{Vec2, Vec3};
use nalgebra::na;

pub type Coord  = Vec3<f32>;
pub type Normal = Vec3<f32>;
pub type UV     = Vec2<f32>;
pub type Vertex = u32;
pub type Face   = Vec3<Vertex>;

pub enum StorageLocation<T> {
    SharedImmutable(Arc<T>),
    NotShared(T)
}

impl<T: Clone + Sync + Send> StorageLocation<T> {
    pub fn unwrap(&self) -> T {
        match *self {
            SharedImmutable(ref a) => a.deref().clone(),
            NotShared(ref t)       => t.clone()
        }
    }
}

impl<T: Sync + Send + Clone> Clone for StorageLocation<T> {
    fn clone(&self) -> StorageLocation<T> {
        match *self {
            SharedImmutable(ref t) => SharedImmutable(t.clone()),
            NotShared(ref t)       => NotShared(t.clone())
        }
    }
}


impl<T: Sync + Send> StorageLocation<T> {
    pub fn new(t: T, shared: bool) -> StorageLocation<T> {
        if shared {
            SharedImmutable(Arc::new(t))
        }
        else {
            NotShared(t)
        }
    }

    pub fn get<'r>(&'r self) -> &'r T {
        match *self {
            SharedImmutable(ref s) => s.deref(),
            NotShared(ref s)       => s
        }
    }

    pub fn is_shared(&self) -> bool {
        match *self {
            SharedImmutable(_) => true,
            NotShared(_)       => false
        }
    }
}

impl<T: Sync + Send + Clone> StorageLocation<T> {
    pub fn write_cow<'r>(&'r mut self, f: |&mut T| -> ()) {
        match *self {
            SharedImmutable(ref mut s) => {
                let mut cpy = s.deref().clone();
                f(&mut cpy);

                *s = Arc::new(cpy);
            },
            NotShared(ref mut s) => f(s)
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
                let normals = compute_normals_array(coords.get().as_slice(), faces.get().as_slice());
                StorageLocation::new(normals, coords.is_shared())
            }
        };

        let uvs = match uvs {
            Some(us) => us,
            None     => {
                let uvs = Vec::from_elem(coords.get().len(), na::zero());
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
    pub fn num_pts(&self) -> uint {
        self.faces.get().len() * 3
    }

    /// Recompute this mesh normals.
    pub fn recompute_normals(&mut self) {
        fail!("Review that.")
        /*
        self.normals.write_cow(
            |normals| compute_normals(self.coords.get().as_slice(), self.faces.get().as_slice(), normals)
        )
        */
    }

    /// This mesh faces.
    pub fn faces<'r>(&'r self) -> &'r [Face] {
        self.faces.get().as_slice()
    }

    /// This mesh faces.
    pub fn mut_faces<'r>(&'r mut self) -> &'r mut StorageLocation<Vec<Face>> {
        &mut self.faces
    }

    /// This mesh normals.
    pub fn normals<'r>(&'r self) -> &'r [Normal] {
        self.normals.get().as_slice()
    }

    /// This mesh normals.
    pub fn mut_normals<'r>(&'r mut self) -> &'r mut StorageLocation<Vec<Normal>> {
        &mut self.normals
    }

    /// This mesh vertices coordinates.
    pub fn coords<'r>(&'r self) -> &'r [Coord] {
        self.coords.get().as_slice()
    }

    /// This mesh vertices coordinates.
    pub fn mut_coords<'r>(&'r mut self) -> &'r mut StorageLocation<Vec<Coord>> {
        &mut self.coords
    }

    /// This mesh texture coordinates.
    pub fn uvs<'r>(&'r self) -> &'r [UV] {
        self.uvs.get().as_slice()
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
    let mut divisor = Vec::from_elem(coordinates.len(), 0f32);

    // Shrink the output buffer if it is too big.
    if normals.len() > coordinates.len() {
        normals.truncate(coordinates.len())
    }

    // Reinit all normals to zero.
    for n in normals.mut_iter() {
        *n = na::zero()
    }

    // Grow the output buffer if it is too small.
    normals.grow_set(coordinates.len() - 1, &na::zero(), na::zero());

    // Accumulate normals ...
    for f in faces.iter() {
        let edge1  = coordinates[f.y as uint] - coordinates[f.x as uint];
        let edge2  = coordinates[f.z as uint] - coordinates[f.x as uint];
        let cross  = na::cross(&edge1, &edge2);
        let normal;

        if !cross.is_zero() {
            normal = na::normalize(&cross)
        }
        else {
            normal = cross
        }

        let normals = normals.as_mut_slice();
        let divisor = divisor.as_mut_slice();
        normals[f.x as uint] = normals[f.x as uint] + normal;
        normals[f.y as uint] = normals[f.y as uint] + normal;
        normals[f.z as uint] = normals[f.z as uint] + normal;

        divisor[f.x as uint] = divisor[f.x as uint] + 1.0;
        divisor[f.y as uint] = divisor[f.y as uint] + 1.0;
        divisor[f.z as uint] = divisor[f.z as uint] + 1.0;
    }

    // ... and compute the mean
    for (n, divisor) in normals.mut_iter().zip(divisor.iter()) {
        *n = *n / *divisor
    }
}
