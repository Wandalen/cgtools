# Curve surface rendering

This example demonstrates how to render a 2d curve on surface like sphere utilizing the `web_sys`, `minwebgl`, `renderer`, `canvas_renderer`, `primitive_generation` crates.

![Showcase]( ./showcase.png )

## How it is useful

The example showcases several useful techniques and concepts:

  * How create main scene for rendering by `renderer::Renderer`.
  * How use `renderer::Renderer` and `renderer::SwapFramebuffer`.
  * How create canvas scene with text for rendering by `canvas_renderer::CanvasRenderer`.
  * How use `canvas_renderer::CanvasRenderer`.
  * How connect main scene object materials with `canvas_renderer::CanvasRenderer` rendered result texture.
  * How set start state of `Camera` and make `Camera` static.

## How it works

1. **Setup main scene and camera**.

Load or create GLTF with scenes and choose scene to render. Set camera state and bind input to camera's controls.

2. **Setup canvas scene and camera**.

Load or create GLTF with scenes and choose scene to render. There is can be added text or curve geometry.

3. **Connect canvas renderer output texture with base texture of any 3d object from main scene**.

Rendering text on surface requires 3D object base texture setup. Base texture of target surface must be output of canvas renderer. That can be applied with this call: 

```no_run, rust
  set_texture
  ( 
    &canvas_sphere, 
    | m | 
    { 
      m.base_color_texture.as_mut()
      .map
      ( 
        | t | 
        {
          let texture = t.texture.borrow().clone();
          t.texture = Rc::new( RefCell::new( texture ) );
          t.texture.borrow_mut().source = Some( canvas_texture.clone() );
        } 
      ); 
      m.alpha_mode = renderer::webgl::AlphaMode::Blend;
    } 
  );
```

If many different surfaces is used with different content, then need collect all textures from base textures of this surfaces. And then for every texture:

  * setup canvas scene layout
  * setup canvas output texture
  * render canvas scene
  * repeat for another texture

4. **Call CanvasRenderer::render**. 

Render canvas scene to canvas output texture. It can be repeated for every unique surface. You only need change `CanvasRenderer` output texture every time you need render texture for next surface.

5. **Call Renderer::render**.

Render main scene for making final frame.

## Running

Make sure you have installed all the necessary dependencies. This example requires `trunk` for building and serving the WebAssembly application.

In order to run the example:

1. Navigate to the example's directory in your terminal.

2. Run the command:

``` bash
  trunk serve
```

3. Open your web browser to the address provided by trunk (usually http://127.0.0.1:8080).

The application will load the GLTF model, compile and link shaders, set up WebGL resources, and start the rendering loop, displaying the 3D object with rendered text on its surface.

If you want you can:
  1. Use a different 3D model for rendering to its surface, replace the `sphere.glb` with any gltf file from `assets/gltf` folder. 
  2. Experiment with canvas scene layout by adding/removing text and changing its transform.
  3. Change canvas renderer camera state.