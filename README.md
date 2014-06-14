nrays
=====

**nrays** is an attempt to make a 3 and 4 dimensional ray tracer in Rust.
It uses the [ncollide](ncollide.org) library to cast rays.

## 3d ray tracing
The current implementation handles phong lighting and reflexions. Nothing fancy here. It supports
the same geometries as **ncollide**, that is, plane, box, sphere, cylinder, cone, and Minkowski sum
of any supported convex objects. Triangle meshes are not yet supported. Here is an example of 3d ray
casting on the Minkowski sum of a cylinder and a box:

![3d ray tracing](http://crozet.re/nrays/render3d.png)

Several examples are given on the `bin` folder. Those are `.scene` files that can be read by the
`loader3d` executable produced by the command `make test`. Those scenes require some assets
originally available [here](http://graphics.cs.williams.edu/data/meshes.xml). The whole set of
asset is packaged [here](https://www.dropbox.com/s/hts81ejea7quxes/media.tar.bz2) and has to be
extracted on the `bin` folder. Here is an example of commands you might type the first time:

```
git clone --recursive git://github.com/sebcrozet/nrays.git
cd nrays
make deps
make
make test
cd bin
wget https://www.dropbox.com/s/hts81ejea7quxes/media.tar.bz2
tar xf media.tar.bz2
./loader3d crytek_sponza.scene
```

## 4d ray tracing
4d ray tracing works quite the same as 3d ray tracing except that the result is no longer a 2d
image. Instead, the output of 4d ray tracing is a voxel grid. Here is an example of 4d ray casting on
a hypercube, hypersphere, hypercone and hypercylinder:

![4d ray tracing](http://crozet.re/nrays/render4d.png)

The voxel grid is visualized with [ParaView](www.paraview.org) and uses false colors and
transparency to render the volumes. To visualize this scene yourself, the data file is available
[here](www.crozet.re/nrays/render.4d) and ParaView should import the file as a _Raw (binary) file_
and use the following settings:
  * Data Extents: 0, 99 (for the three extents)
  * Data Scalar Type: float
  * Data Byte Order: LittleEndian

Then, visualize the scene using the "Volume" rendering mode.

The more reddish a voxel is, the more bright it is (due to a 4-dimensional light). Reflexions are
activated but not visible on this screenshot. If you rotate the grid in ParaView, you will see
a small cylinder inside of the ball.

Here is a screenshot of the properties dialog box:

![dialog box](http://crozet.re/nrays/properties_paraview.png)

Note that this scene is obtained using an orthographic projection, with the eye pointing toward the
fourth axis. Thus, since the fourth dimension collapses into the 3d hyperplane, the (not rotated)
objects appear in a natural form.

## Bibliography
Some interesting informations about 4-dimensional rendering:
* http://steve.hollasch.net/thesis/chapter4.html
* http://eusebeia.dyndns.org/4d/vis/01-intro
* http://www.urticator.net/maze/
* http://spacesymmetrystructure.wordpress.com/2008/12/11/4-dimensional-rotations/
