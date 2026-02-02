# Weighted Blended Order-Independent Transparency

**Keywords:** Transparency, WebGL2, OIT, Blending

This demo demonstrates **order-independent transparency** using the **weighted blended OIT** technique in WebGL2.
Traditional transparency rendering requires sorting all transparent objects back-to-front, which becomes costly or incorrect when geometry is interpenetrating.
Weighted blended OIT provides an approximate but visually convincing solution without sorting.
It accumulates color and transparency in separate buffers using floating-point blending, then combines them in a final compositing pass.

This approach works efficiently on modern GPUs and is suitable for real-time scenes such as particle systems, glass objects, and volumetric effects.

![image](./minwebgl/area_light/showcase.png)

**[How to run](./how_to_run.md)**

**References:**

* [WebGL2 Fundamentals]
* [Khronos WebGL2 Specification]

[WebGL2 Fundamentals]: https://webgl2fundamentals.org
[Khronos WebGL2 Specification]: https://registry.khronos.org/webgl/specs/latest/2.0/
