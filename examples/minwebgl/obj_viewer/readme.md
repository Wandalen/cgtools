# OBJ Model Viewer

**Keywords:** OBJ Format, 3D Viewer, WebGL2, Interactive

This demo is a complete OBJ model viewer with interactive camera controls in WebGL2. Building on OBJ loading, it provides a full viewing experience with rotation, zoom, and lighting. It starts with a bundled "lost empire" scene — upload your own `.obj` file via the picker in the top-left corner to swap it in, along with a full diagnostic report (vertex/normal/texcoord/face counts, bounding box and sphere, material properties) logged to the browser console.

A standalone uploaded `.obj` has no server-side path to fetch a companion `.mtl`/textures from, so an uploaded model renders with the shader's default (untextured) look rather than failing — geometry and diagnostics work all the same.

This example demonstrates practical 3D viewer implementation, suitable for asset inspection, education, or portfolio presentation of 3D models.

![image](showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [Wavefront OBJ]

[Wavefront OBJ]: https://uk.wikipedia.org/wiki/Wavefront_OBJ
