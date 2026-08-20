# Object Picking

**Keywords:** Interaction, Mouse Picking, WebGL2, Selection

This demo demonstrates object picking techniques in WebGL2. It shows how to determine which 3D object the user clicked on, essential for interactive 3D applications and games.

Object picking typically uses color-coded rendering or ray casting. This example presents efficient methods for selection detection in complex 3D scenes.

Objects are draggable: click and hold the left mouse button on an object to pick it up (re-running the id-texture pick since dragging makes object positions non-static) and select it. While held, the cursor is unprojected each frame onto a fixed plane at the object's original depth, and the object is moved so the point originally grabbed stays under the cursor rather than snapping to its center. Releasing ends the drag; the mouseup listener is attached to the window rather than the canvas, so a release outside the canvas bounds still ends the drag cleanly instead of leaving the object stuck to the cursor.

![image](./showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [WebGL Object Picking] - Comprehensive picking tutorial
* [GPU Gems - Selection Techniques] - Advanced selection methods
* [Real-Time Rendering] - Interactive graphics theory
* [OpenGL Superbible] - Low-level graphics programming

[WebGL Object Picking]: https://webglfundamentals.org/webgl/lessons/webgl-picking.html
[GPU Gems - Selection Techniques]: https://developer.nvidia.com/gpugems/gpugems2/part-iii-high-quality-rendering/chapter-22-hardware-occlusion-queries-made-useful
[Real-Time Rendering]: http://www.realtimerendering.com/
[OpenGL Superbible]: https://www.openglsuperbible.com/
