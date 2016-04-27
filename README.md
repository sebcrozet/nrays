nrays
=====

**nrays** is an attempt to make a 3 and 4 dimensional ray tracer in Rust.
It uses the [ncollide](http://ncollide.org) library to cast rays.

## 3d ray tracing
The current implementation handles phong lighting and reflexions. Nothing fancy here. It supports
the same geometries as **ncollide**, that is, plane, box, sphere, cylinder, cone, and Minkowski sum
of any supported convex objects. Triangle meshes are not yet supported. Here is an example of 3d ray
casting on the Minkowski sum of a cylinder and a box:

![3d ray tracing](http://crozet.re/nrays/render3d.png)

Several examples are given on the `bin` folder. Those are `.scene` files that can be read by the
`loader3d` executable produced by the command `make`. Those scenes require some assets
originally available [here](http://graphics.cs.williams.edu/data/meshes.xml). The whole set of
asset is packaged [here](https://www.dropbox.com/s/hts81ejea7quxes/media.tar.bz2) and has to be
extracted on the `bin` folder. Here is an example of commands you might type the first time:

```
git clone git://github.com/sebcrozet/nrays.git
cd nrays
make
cd bin
wget https://www.dropbox.com/s/hts81ejea7quxes/media.tar.bz2
tar xf media.tar.bz2
../target/release/loader3d crytek_sponza.scene
```
