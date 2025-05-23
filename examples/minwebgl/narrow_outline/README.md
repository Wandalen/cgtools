# Real-time narrow outline using WebGL2

This example demonstrates how to render a 3D object with a real-time narrow outline effect using **WebGL2** and the `web_sys` and `minwebgl` crates.

The outline is achieved using a **post-processing technique** that analyzes the rendered object's silhouette.

-----

## How it is useful

This example showcases several useful techniques and concepts for WebGL2 development:

  * **Multi-pass rendering pipeline**: Learn how to chain multiple rendering operations to achieve a complex visual effect.
  * **Offscreen framebuffers**: Understand how to render to textures (`WebGlFramebuffer`) instead of directly to the screen, and then use these textures as inputs in subsequent passes.
  * **Post-processing effects**: Implement a visual effect applied to an entire rendered scene.
  * **3D model loading**: See how to load and process 3D models in **GLTF format** using the `gltf` crate.
  * **3D transformations**: Work with **Model, View, and Projection matrices** to position and orient objects in 3D space.
  * **Basic animation**: Observe how to animate properties like camera rotation and outline thickness over time.
  * **`minwebgl` utilities**: Utilize helper functions for common WebGL tasks like file loading, buffer uploads, and shader program compilation.
  * **`web_sys` interaction**: See how to interact with the underlying WebGL2 API directly through the `web_sys` crate.
  * **CSG (Constructive Solid Geometry)**: Generate complex 3D shapes programmatically.

-----

## How it works

The outline effect is created through a two-pass rendering process:

1.  **Object Pass**:

      * The 3D object(s) are rendered to an **offscreen framebuffer** named `object_fb`.
      * The `object.vert` and `object.frag` shaders are used for this pass.
      * The `object.frag` shader simply outputs **white for every fragment** of the object, while the background remains transparent (or clear color).
      * The result of this pass is a **silhouette of the object** stored in the `object_fb_color` texture, along with its depth information in `object_fb_depth`.

2.  **Outline Pass**:

      * A fullscreen quad is rendered directly to the screen (the default framebuffer).
      * The `fullscreen.vert` and `outline.frag` shaders are used.
      * The `outline.frag` shader samples both the `object_fb_color` (silhouette) and `object_fb_depth` textures from the previous pass.
      * It uses a **Sobel operator** (a common edge detection filter) on the `object_fb_color` texture to find the edges of the silhouette.
      * It then combines this edge information with the **linear depth** of the pixels from `object_fb_depth` to create a more robust outline.
      * Based on whether a pixel is part of the original object, part of the calculated outline, or the background, it applies `u_object_color`, `u_outline_color`, or `u_background_color` respectively.

-----

## Running

Ensure you have all the necessary dependencies installed. This example uses `trunk` for building and serving the WebAssembly application.

To run the example:

1.  Navigate to the example's directory in your terminal.

2.  Run the command:

    ```bash
    trunk serve
    ```

3.  Open your web browser to the address provided by trunk (usually `http://127.0.0.1:8080`).

The application will load the GLTF model, compile shaders, set up WebGL resources, and start the rendering loop, displaying the 3D object with an animated outline.

Feel free to replace `resources/model.glb` with your own 3D model, or experiment with the outline parameters (thickness, colors) by modifying the values in the `outline_pass` function in `main.rs`.

---

## Used resources and links

1. [model.glb](https://sketchfab.com/3d-models/kawasaki-ninja-h2-free-0ab38d6e39664b25bfaf9f5c3e0c767c).

2. outline.frag created using this [shader](https://godotshaders.com/shader/post-effect-outline-shader-for-gles2/).