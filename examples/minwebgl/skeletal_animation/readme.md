# Skeletal Animation

**Keywords:** Animation, Skeletal Animation, glTF, WebGL2, Skinning

This demo demonstrates skeletal animation implementation in WebGL2, featuring animated skinned 3D models loaded from glTF files. It showcases smooth skeletal animation playback with interpolation between keyframes, supporting multiple animation sequences that can be selected through an interactive UI. The viewer handles joint transformations, inverse bind matrices, and vertex skinning in real-time.

Skeletal animation is fundamental for character animation in games and interactive 3D applications. This example provides a complete implementation of the glTF animation pipeline, including channel interpolation (linear, step, cubic spline), joint hierarchy management, and GPU-accelerated vertex skinning, making it suitable for character-driven applications and animation previews.

![image](./showcase.gif)

**[How to run](../how_to_run.md)**

**References:**

* [WebGL Skinning] - Comprehensive guide to skeletal animation in WebGL
* [glTF 2.0 Reference Guide] - Official glTF specification and format details
* [Animation Sampler Interpolation Modes] - glTF animation interpolation methods
* [WebGL2 3D - Data Textures] - Using textures for storing joint matrices
* [How to use textures as data] - Efficient uniform data storage techniques
