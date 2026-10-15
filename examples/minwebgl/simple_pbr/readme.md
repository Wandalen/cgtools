# Simple PBR

**Keywords:** PBR, Materials, WebGL2, Lighting

This demo demonstrates a simplified physically-based rendering implementation in WebGL2. It introduces PBR concepts with a minimal, educational approach focusing on core metallic-roughness workflow.

The whole scene is a single fullscreen-quad fragment shader -- no mesh, just a per-pixel analytic
sphere SDF -- rendering a grid of spheres that sweep roughness (smooth to rough, left to right)
across two rows (dielectric on the bottom, metal on top), all sharing one live-adjustable base
color. Lighting is a three-point key/fill/rim setup plus a hemisphere sky/ground ambient term, so
no side of a sphere is ever pure black, tonemapped with an ACES filmic curve. Base color, light
intensity, ambient intensity and exposure are controllable live via the lil-gui panel.

Simple PBR provides realistic materials without full complexity. This example is ideal for learning PBR fundamentals before tackling complete implementations.

![](./showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [Physically based rendering]
* [PBR Theory]

[Physically based rendering]: https://en.wikipedia.org/wiki/Physically_based_rendering
[PBR Theory]: https://learnopengl.com/PBR/Theory
