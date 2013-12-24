//! Data structure of a scene node geometry.

use extra::arc::Arc;
use std::vec;
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
    // FIXME: add a GPU-only storage location
    // FIXME: add SharedMutable
}

impl<T: Clone + Send + Freeze> StorageLocation<T> {
    pub fn unwrap(&self) -> T {
        match *self {
            SharedImmutable(ref a) => a.get().clone(),
            NotShared(ref t)       => t.clone()
        }
    }
}

impl<T: Send + Freeze> StorageLocation<T> {
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
            SharedImmutable(ref s) => s.get(),
            NotShared(ref s)     => s
        }
    }

    pub fn is_shared(&self) -> bool {
        match *self {
            SharedImmutable(_) => true,
            NotShared(_)       => false
        }
    }
}

impl<T: Send + Freeze + Clone> StorageLocation<T> {
    pub fn write_cow<'r>(&'r mut self, f: |&mut T| -> ()) {
        match *self {
            SharedImmutable(ref mut s) => {
                let mut cpy = s.get().clone();
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
    priv coords:  StorageLocation<~[Coord]>,
    priv faces:   StorageLocation<~[Face]>,
    priv normals: StorageLocation<~[Normal]>,
    priv uvs:     StorageLocation<~[UV]>,
}

impl Mesh {
    /// Creates a new mesh. Arguments set to `None` are automatically computed.
    pub fn new(coords:          StorageLocation<~[Coord]>,
               faces:           StorageLocation<~[Face]>,
               normals:         Option<StorageLocation<~[Normal]>>,
               uvs:             Option<StorageLocation<~[UV]>>)
               -> Mesh {
        let normals = match normals {
            Some(ns) => ns,
            None     => {
                let normals = compute_normals_array(*coords.get(), *faces.get());
                StorageLocation::new(normals, coords.is_shared())
            }
        };

        let uvs = match uvs {
            Some(us) => us,
            None     => {
                let uvs = vec::from_elem(coords.get().len(), na::zero());
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
        self.normals.write_cow(
            |normals| compute_normals(*self.coords.get(), *self.faces.get(), normals)
        )
    }

    /// This mesh faces.
    pub fn faces<'r>(&'r self) -> &'r [Face] {
        let res: &'r [Face] = *self.faces.get();

        res
    }

    /// This mesh faces.
    pub fn mut_faces<'r>(&'r mut self) -> &'r mut StorageLocation<~[Face]> {
        &'r mut self.faces
    }

    /// This mesh normals.
    pub fn normals<'r>(&'r self) -> &'r [Normal] {
        let res: &'r [Normal] = *self.normals.get();

        res
    }

    /// This mesh normals.
    pub fn mut_normals<'r>(&'r mut self) -> &'r mut StorageLocation<~[Normal]> {
        &'r mut self.normals
    }

    /// This mesh vertices coordinates.
    pub fn coords<'r>(&'r self) -> &'r [Coord] {
        let res: &'r [Coord] = *self.coords.get();

        res
    }

    /// This mesh vertices coordinates.
    pub fn mut_coords<'r>(&'r mut self) -> &'r mut StorageLocation<~[Coord]> {
        &'r mut self.coords
    }

    /// This mesh texture coordinates.
    pub fn uvs<'r>(&'r self) -> &'r [UV] {
        let res: &'r [UV] = *self.uvs.get();

        res
    }

    /// This mesh texture coordinates.
    pub fn mut_uvs<'r>(&'r mut self) -> &'r mut StorageLocation<~[UV]> {
        &'r mut self.uvs
    }
}

/// Comutes normals from a set of faces.
pub fn compute_normals_array(coordinates: &[Coord],
                             faces:       &[Face])
                             -> ~[Normal] {
    let mut res = ~[];

    compute_normals(coordinates, faces, &mut res);

    res
}

/// Comutes normals from a set of faces.
pub fn compute_normals(coordinates: &[Coord],
                       faces:       &[Face],
                       normals:     &mut ~[Normal]) {
    let mut divisor = vec::from_elem(coordinates.len(), 0f32);

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
        let edge1  = coordinates[f.y] - coordinates[f.x];
        let edge2  = coordinates[f.z] - coordinates[f.x];
        let normal = na::normalize(&na::cross(&edge1, &edge2));

        normals[f.x] = normals[f.x] + normal;
        normals[f.y] = normals[f.y] + normal;
        normals[f.z] = normals[f.z] + normal;

        divisor[f.x] = divisor[f.x] + 1.0;
        divisor[f.y] = divisor[f.y] + 1.0;
        divisor[f.z] = divisor[f.z] + 1.0;
    }

    // ... and compute the mean
    for (n, divisor) in normals.mut_iter().zip(divisor.iter()) {
        *n = *n / *divisor
    }
}
