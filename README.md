nrays
=====

**nrays** is an attempt to make a 3 and 4 dimensional ray tracer in Rust.
It uses the [ncollide](https://github.com/sebcrozet/ncollide) library to cast rays.

# 3d ray tracing
The current implementation handles phong lighting and reflexions. Nothing fancy here. It supports
the same geometries as **ncollide**, that is, plane, box, sphere, cylinder, cone, and minkowski sum
of any supported convex objects. Triangle meshes are not yet supported.

# 4d ray tracing
4d ray tracing works quite the same as 3d ray tracing except that the result is no longer a 2d
image. Instead, the output of 4d ray tracing is a voxel grid. Here is an example of 4d rendering of
an hypercube, hypersphere, hypercone and hypercylinder:
[4d ray tracing]: http://crozet.re/nrays/render4d.png "4d ray tracing"

The voxel grid is visualized with [ParaView](www.paraview.org) and uses false colors and
transparency to render the volumes. To visualize this scene yourself, the data file is available
[here](www.crozet.re/nrays/render.4d) and ParaView should import the file as a _Raw (binary) file_
and use the following settings:
    − Data Extents: 0, 99 (for the three extents)
    − Data Scalar Type: float
    − Data Byte Order: LittleEndian
Then, visualize the scene with the "Volume" rendering mode.
Here is a screenshot of the properties dialog box:
[bug]: http://crozet.re/properties.png "Logo Title Text 2"

Note that this scene is obtained using an orthographic projection, with the eye pointing toward the
fourth axis. Thus, since the fourth dimension collapses into the 3d hyperplane, the (not rotated)
objects appear in a natural form.

## Bibliography
Some interesting informations about 4-dimensional rendering:
* http://steve.hollasch.net/thesis/chapter4.html
* http://eusebeia.dyndns.org/4d/vis/01-intro
* http://www.urticator.net/maze/
* http://spacesymmetrystructure.wordpress.com/2008/12/11/4-dimensional-rotations/
