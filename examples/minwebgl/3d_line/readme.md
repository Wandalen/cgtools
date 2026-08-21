# 3D Line Rendering

**Keywords:** 3D Graphics, Lines, WebGL2, Perspective

This demo renders an N-body gravitational simulation as animated 3D line trails: bodies attract
each other according to Newton's law of gravitation (with a central restoring force keeping them
from drifting off-screen, and a repulsion kick to avoid a singularity when two bodies nearly
overlap), and each body's recent trajectory is drawn as a dashed 3D line using instanced line
rendering with perspective projection.

A mouse-orbit camera (left-click drag to rotate, right-click drag to pan, scroll to zoom) frames
the simulation, and a 13-control lil-gui panel exposes live tuning for world/screen line width,
alpha-to-coverage, world-space vs. screen-space units, dash visibility, trail length, simulation
speed, and the dash pattern itself (two independently sized dash/gap segments plus a dash offset
and a dash-version toggle).

This example builds on basic 3D line rendering to demonstrate model-view-projection matrices,
depth testing, and 3D coordinate systems, serving as a building block for understanding more
complex 3D rendering pipelines.

![image](./showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [Instanced Line Rendering Part I]
* [Instanced Line Rendering Part II: Alpha blending]
* [Three.js line example]

[Instanced Line Rendering Part I]: https://wwwtyro.net/2019/11/18/instanced-lines.html
[Instanced Line Rendering Part II: Alpha blending]: https://wwwtyro.net/2021/10/01/instanced-lines-part-2.html
[Three.js line example]: https://threejs.org/examples/?q=Line#webgl_lines_fat_raycasting
