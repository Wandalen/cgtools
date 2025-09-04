# canvas_renderer

A Rust crate designed for offscreen WebGL2 rendering of 2D graphics. The `CanvasRenderer` provides a dedicated system for drawing objects such as text, curves, and 2D shapes, encapsulating them within a `Scene` object. Instead of rendering directly to the screen, it outputs the result to a texture, which can then be used as a dynamic texture on a 3D object within a larger WebGL scene. This is ideal for creating interactive interfaces, information displays, or other dynamic 2D elements within a 3D environment.

[![Crates.io](https://img.shields.io/crates/v/canvas_renderer.svg)](https://crates.io/crates/canvas_renderer)
[![Documentation](https://docs.rs/canvas_renderer/badge.svg)](https://docs.rs/canvas_renderer)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

  * **Specialized 2D Rendering:** A dedicated renderer for drawing a variety of 2D objects, including text and vector shapes.

  * **Offscreen Rendering to Texture:** Renders the 2D scene directly to a WebGL texture, enabling dynamic content to be seamlessly integrated into a 3D world.

  * **Scene Integration:** Designed to work with a scene graph based on `Node` and `Object3D` types, allowing for structured scene management and the integration of 2D and 3D elements.

  * **Shader Uniforms:** Automatically handles uploading `worldMatrix`, `viewMatrix`, and `projectionMatrix` to the shader program.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
canvas_renderer = "0.1.0"
```

For full functionality, enable the `full` feature:

```toml
[dependencies]
canvas_renderer = { version = "0.1.0", features = ["full"] }
```

## Features

- `enabled` (default): Core canvas rendering functionality
- `full`: All features enabled

## Usage

Here is a conceptual example demonstrating how to initialize and use the `CanvasRenderer`. This example shows how to render a 2D scene to a texture, and then use that texture on a 3D object in your main scene, which is rendered by a `Renderer`.

```rust
use canvas_renderer::renderer::CanvasRenderer;
use minwebgl as gl;
use renderer::webgl::{ Camera, Node, Scene, Renderer, Texture, TextureInfo, Material };
use std::rc::Rc;
use std::cell::RefCell;

fn set_texture
(
  node : &Rc< RefCell< Node > >,
  mut material_callback : impl FnMut( &mut Material )
)
{
  if let renderer::webgl::Object3D::Mesh( ref mesh ) = &node.borrow().object
  {
    for p in &mesh.borrow().primitives
    {
      material_callback( &mut p.borrow().material.borrow_mut() );
    }
  }
}

fn setup_scene() -> Scene
{
  let mut scene = Scene::new();
  let node = Node::default();
  scene.children.push( Rc::new( RefCell::new( node ) ) );

  scene
}

fn setup_and_render( gl : &gl::GL ) -> Result< (), gl::WebglError >
{
  let width = 800;
  let height = 600;

  // --- 1. Set up and render the content for the offscreen canvas ---

  // Create the CanvasRenderer and get its initial output texture
  let canvas_renderer = CanvasRenderer::new( &gl, width, height )?;
  let canvas_texture_handle = canvas_renderer.get_texture();

  // In a real application, you would populate `canvas_scene` with 2D elements.
  let mut canvas_scene = Scene::new();

  let mut eye = gl::math::F32x3::from( [ 0.0, 0.0, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] );

  let aspect_ratio = width as f32 / height as f32;
  let fov = 70.0f32.to_radians();
  let near = 0.1;
  let far = 10000000.0;

  // Set up the main camera and render the final scene.
  let canvas_camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );

  let colors = &[]; // Colors for the 2D elements.

  // Render the 2D scene to the CanvasRenderer's texture.
  canvas_renderer.render( &gl, &mut canvas_scene, &canvas_camera, colors )?;

  // --- 2. Set up the main 3D scene and use the offscreen texture ---

  // Create the main Renderer for the final output
  let mut main_renderer = Renderer::new( &gl, width, height, 4 )?;

  // For this example, we assume that setup_scene returns
  // complete `Scene` struct with 3D objects.
  let mut main_scene = setup_scene();

  // Object for that you want change texture
  let object = main_scene.children.get( 0 ).unwrap().clone();

  // Create a new `Texture` and set its source to the texture from the CanvasRenderer.
  let canvas_texture = Texture::former()
  .source( canvas_texture_handle )
  .end();

  set_texture
  (
    &object,
    | m |
    {
      m.base_color_texture.as_mut()
      .map
      (
        | t |
        {
          let texture = t.texture.borrow().clone();
          t.texture = Rc::new( RefCell::new( texture ) );
          t.texture.borrow_mut().source = canvas_texture.clone().source;
        }
      );
      m.alpha_mode = renderer::webgl::AlphaMode::Blend;
    }
  );

  // Set up the main camera and render the final scene.
  let main_camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );

  main_renderer.render( &gl, &mut main_scene, &main_camera )?;

  Ok( () )
}
```

## Platform Support

This crate is designed for WebAssembly environments and supports:

- `wasm32-unknown-unknown` (primary target)
- Native builds for development and testing

## Dependencies

- `minwebgl`: WebGL context management and utilities
- `mingl`: Mathematics and 3D graphics utilities
- `renderer`: Core rendering functionality
- `web-sys`: Browser API bindings

## License

Licensed under the MIT License. See [LICENSE](license) file for details.

## Contributing

Contributions are welcome! Please see the [repository](https://github.com/Wandalen/cgtools) for contribution guidelines.
