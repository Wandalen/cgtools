//! This module contains the implementation for offscreen
//! rendering to a texture using WebGL2. It includes a utility
//! function to create a framebuffer and the `CanvasRenderer`
//! struct for managing the rendering process.

mod private
{
  use minwebgl as gl;
  use gl::
  {
    F32x4,
    drawbuffers::drawbuffers,
    GL,
    web_sys::
    {
      WebGlFramebuffer,
      WebGlProgram,
      WebGlTexture
    }
  };
  use renderer::webgl::
  {
    Object3D,
    Node,
    Camera,
    Scene
  };
  use rustc_hash::FxHashMap;
  use std::cell::RefCell;
  use std::rc::Rc;

  /// Creates a WebGL2 framebuffer and a color attachment texture.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  /// * `width`, `height` - The size of the framebuffer and its attachment.
  ///
  /// # Returns
  ///
  /// An `Option< ( WebGlFramebuffer, WebGlTexture ) >` containing the created framebuffer and
  /// its color attachment texture, or `None` if creation fails.
  fn create_framebuffer
  (
    gl : &gl::GL,
    width : u32,
    height : u32
  )
  -> Option< ( WebGlFramebuffer, WebGlTexture ) >
  {
    let color = gl.create_texture()?;
    gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
    gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, width as i32, height as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

    let depthbuffer = gl.create_renderbuffer().unwrap();
    gl.bind_renderbuffer( GL::RENDERBUFFER, Some( &depthbuffer ) );
    gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width as i32, height as i32 );

    let framebuffer = gl.create_framebuffer()?;
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
    gl.viewport(0, 0, width as i32, height as i32 );
    gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &color ), 0 );
    gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, Some( &depthbuffer ) );

    drawbuffers( gl, &[ 0 ] );

    gl.bind_framebuffer( gl::FRAMEBUFFER, None );

    Some( ( framebuffer, color ) )
  }

  /// The fallback color applied to a mesh that has no corresponding entry in `colors`.
  fn default_color() -> F32x4
  {
    F32x4::from_array( [ 1.0, 0.0, 1.0, 1.0 ] )
  }

  /// Resolves, in mesh-encounter order, the color that `CanvasRenderer::render` will apply to
  /// each mesh node while traversing `scene`.
  ///
  /// `colors` holds one entry per *mesh*, in the order meshes are encountered during
  /// traversal -- not one entry per traversed node. Non-mesh nodes (`Object3D::Light`,
  /// `Object3D::Other`, and any future variant) are visited but must never consume an entry
  /// from `colors`.
  ///
  /// Kept separate from `render`'s own GL-drawing traversal so the mesh-to-color
  /// correspondence can be verified independent of a live WebGL context.
  ///
  /// # Arguments
  ///
  /// * `scene` - The scene to traverse (read-only; traversal order matches `render`'s).
  /// * `colors` - Per-mesh colors, in mesh-encounter order.
  ///
  /// # Returns
  ///
  /// One resolved color per mesh encountered, in traversal order. A mesh beyond the end of
  /// `colors` resolves to `default_color()`.
  // Fix(TASK-016): index `colors` by `resolved.len()` -- the count of meshes already resolved
  // -- instead of a counter shared with every traversed node.
  // Root cause: the lookup index previously advanced once per traversed node (mesh or not),
  // but was only ever read while visiting a mesh, so any non-mesh node visited before or
  // between mesh nodes shifted every later mesh onto the wrong `colors` entry.
  // Pitfall: when a lookup index is shared between a filtered consumer (only meshes read it)
  // and an unfiltered traversal (every node advances it), the two silently drift apart the
  // moment a "skipped" item actually occurs -- count only what is actually consumed.
  pub fn resolve_mesh_colors( scene : &Scene, colors : &[ F32x4 ] ) -> Vec< F32x4 >
  {
    let mut resolved = Vec::new();

    let mut visit =
    |
      node : Rc< RefCell< Node > >
    | -> Result< (), gl::WebglError >
    {
      if let Object3D::Mesh( _ ) = node.borrow().object
      {
        resolved.push( *colors.get( resolved.len() ).unwrap_or( &default_color() ) );
      }

      Ok( () )
    };

    // `visit` never returns `Err`, so `traverse` cannot fail here.
    scene.traverse( &mut visit ).unwrap();

    resolved
  }

  /// A 2D canvas renderer that renders 3D scenes to a texture using WebGL.
  ///
  /// This renderer creates a framebuffer with a color attachment texture and provides
  /// methods to render scenes with custom colors and camera configurations.
  pub struct CanvasRenderer
  {
    /// The WebGL program used for rendering.
    program : WebGlProgram,
    /// A map storing the locations of uniform variables in the program.
    uniforms : FxHashMap< String, Option< gl::WebGlUniformLocation > >,
    /// The WebGL framebuffer used for offscreen rendering.
    framebuffer : WebGlFramebuffer,
    /// The texture attached to the framebuffer, where the rendering results are stored.
    output_texture : WebGlTexture,
    /// The width of the framebuffer and its output texture.
    width : u32,
    /// The height of the framebuffer and its output texture.
    height : u32
  }

  impl CanvasRenderer
  {
    /// Creates a new canvas renderer with the specified dimensions.
    ///
    /// This function compiles and links the canvas shaders, initializes uniform locations,
    /// and creates a framebuffer with color and depth attachments.
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL2 rendering context
    /// * `width` - Width of the render target in pixels
    /// * `height` - Height of the render target in pixels
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if shader compilation or program linking fails.
    pub fn new( gl : &GL, width : u32, height : u32 ) -> Result< Self, gl::WebglError >
    {
      let vertex_shader_src = include_str!( "../shaders/canvas.vert" );
      let fragment_shader_src = include_str!( "../shaders/canvas.frag" );
      let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src )
      .compile_and_link( gl )?;

      let mut uniforms = FxHashMap::default();
      let mut add_location =
      | name : &str |
      {
        uniforms.insert
        (
          name.to_string(),
          gl.get_uniform_location( &program, name )
        )
      };

      add_location( "color" );
      add_location( "worldMatrix" );
      add_location( "viewMatrix" );
      add_location( "projectionMatrix" );

      let Some( ( framebuffer, output_texture ) ) = create_framebuffer( gl, width, height )
      else
      {
        return Err( gl::WebglError::FailedToAllocateResource( "Framebuffer" ) );
      };

      Ok(
        Self
        {
          program,
          uniforms,
          framebuffer,
          output_texture,
          width,
          height
        }
      )
    }

    /// Uploads the camera's view and projection matrices to the shader uniforms.
    fn upload_camera( &self, gl : &GL, camera : &Camera )
    {
      gl::uniform::matrix_upload
      (
        gl,
        self.uniforms.get( "viewMatrix" ).unwrap().clone(),
        &camera.get_view_matrix().to_array(),
        true
      ).unwrap();

      gl::uniform::matrix_upload
      (
        gl,
        self.uniforms.get( "projectionMatrix" ).unwrap().clone(),
        &camera.get_projection_matrix().to_array(),
        true
      ).unwrap();
    }

    /// Uploads the world transformation matrix of a node to the GPU.
    ///
    /// This method updates the "worldMatrix" uniform with the node's world transformation matrix.
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL2 rendering context
    /// * `node` - The scene node whose world matrix will be uploaded
    ///
    /// # Panics
    ///
    /// Panics if the `worldMatrix` uniform location is missing or the matrix upload fails.
    pub fn upload_node
    (
      &self,
      gl : &GL,
      node : &Rc< RefCell< Node > >
    )
    {
      gl::uniform::matrix_upload
      (
        gl,
        self.uniforms.get( "worldMatrix" ).unwrap().clone(),
        node.borrow().get_world_matrix().to_array().as_slice(),
        true
      ).unwrap();
    }

    /// Renders a 3D scene to the internal framebuffer using specified colors.
    ///
    /// This method configures WebGL state, binds the framebuffer, and traverses the scene
    /// to render all mesh nodes with their corresponding colors from the colors array.
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL2 rendering context
    /// * `scene` - The 3D scene to render (will update world matrices)
    /// * `camera` - The camera defining view and projection matrices
    /// * `colors` - Array of colors to apply to scene nodes in order
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if a mesh upload or draw step fails.
    ///
    /// # Panics
    ///
    /// Panics if a required uniform location was not registered at construction, or if scene
    /// traversal or a uniform upload fails.
    pub fn render
    (
      &self,
      gl : &GL,
      scene : &mut Scene,
      camera : &Camera,
      colors : &[ F32x4 ]
    ) -> Result< (), gl::WebglError >
    {
      scene.update_world_matrix();

      gl.enable( gl::DEPTH_TEST );
      gl.disable( gl::BLEND );
      gl.depth_mask( true );
      gl.clear_depth( 1.0 );
      gl.front_face( gl::CCW );

      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
      gl.viewport(0, 0, self.width as i32, self.height as i32 );

      gl::drawbuffers::drawbuffers( gl, &[ 0 ] );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, &[ 0.0, 0.0, 0.0, 0.0 ] );
      gl.clear( gl::DEPTH_BUFFER_BIT );

      gl.use_program( Some( &self.program ) );

      // Resolved once, up front, in mesh-encounter order -- see `resolve_mesh_colors` for why
      // this can't be a counter shared with the node-traversal below.
      let mesh_colors = resolve_mesh_colors( scene, colors );
      let mut mesh_i = 0;

      // Define a closure to handle the drawing of each node in the scene.
      let mut draw_node =
      |
        node : Rc< RefCell< Node > >
      | -> Result< (), gl::WebglError >
      {
        // If the node contains a mesh...
        if let Object3D::Mesh( ref mesh ) = node.borrow().object
        {
          gl::uniform::upload
          (
            gl,
            self.uniforms.get( "color" ).unwrap().clone(),
            mesh_colors.get( mesh_i ).unwrap_or( &default_color() ).as_slice()
          ).unwrap();

          mesh_i += 1;

          // Iterate over each primitive in the mesh.
          for primitive_rc in &mesh.borrow().primitives
          {
            let primitive = primitive_rc.borrow();

            self.upload_camera( gl, camera );
            self.upload_node( gl, &node );

            primitive.geometry.borrow().bind( gl );
            primitive.draw( gl );
          }
        }

        Ok( () )
      };

      // Traverse the scene and draw all opaque objects.
      scene.traverse( &mut draw_node )?;

      Ok( () )
    }

    /// Sets a new output texture as the color attachment for the framebuffer.
    ///
    /// This method replaces the current output texture with the provided one,
    /// effectively changing where the renderer will draw its output.
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL2 rendering context
    /// * `output_texture` - The new texture to use as the color attachment
    pub fn set_texture
    (
      &mut self,
      gl : &GL,
      output_texture : WebGlTexture
    )
    {
      gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
      gl.viewport(0, 0, self.width as i32, self.height as i32 );
      gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &output_texture ), 0 );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      self.output_texture = output_texture;
    }

    /// Returns a clone of the current output texture.
    ///
    /// This method provides access to the texture that contains the rendered output,
    /// which can be used for further processing or display.
    ///
    /// # Returns
    ///
    /// A clone of the WebGlTexture that serves as the color attachment.
    #[must_use]
    pub fn get_texture( &self ) -> WebGlTexture
    {
      self.output_texture.clone()
    }
  }
}

// Documented exception (task 068) to the all-tests-in-tests/ convention: this test stays
// inline because it needs `super::private::*` by design. `resolve_mesh_colors` is
// deliberately internal -- it exists precisely so the mesh-to-color correspondence can be
// verified WITHOUT a live WebGL context, and publishing it solely for test placement would
// widen the API for no caller. Testing through the public surface instead is not an option
// either: every `CanvasRenderer` method takes `&GL`, so a native `tests/` suite would have
// nothing it could exercise -- browser-side testing waits on the workspace's wasm
// test-runner infrastructure (see tilemap_renderer's roadmap for that gap).
#[ cfg( test ) ]
mod tests
{
  use super::private::*;
  use renderer::webgl::{ Mesh, Node, Object3D, Scene };
  use minwebgl::F32x4;
  use std::cell::RefCell;
  use std::rc::Rc;

  /// Builds a non-mesh node -- a transform-only group, matching how
  /// `primitive_generation::primitives_data_to_gltf` creates "parent" nodes for
  /// `PrimitiveData` entries that carry no attributes.
  fn group_node() -> Rc< RefCell< Node > >
  {
    Rc::new( RefCell::new( Node::new() ) )
  }

  /// Builds a mesh node with no primitives -- `resolve_mesh_colors` only inspects whether the
  /// node is `Object3D::Mesh`, never `Mesh::primitives`.
  fn mesh_node() -> Rc< RefCell< Node > >
  {
    let node = Rc::new( RefCell::new( Node::new() ) );
    node.borrow_mut().object = Object3D::Mesh( Rc::new( RefCell::new( Mesh::new() ) ) );
    node
  }

  /// ## Root Cause
  /// `CanvasRenderer::render` looked up each mesh's color using a counter that advanced once
  /// per *traversed scene node* (mesh or not), while `colors` holds one entry per *mesh*, in
  /// mesh-encounter order (per `render`'s own doc comment: "renders all mesh nodes with their
  /// corresponding colors from the colors array"). Any non-mesh node visited before or between
  /// mesh nodes -- a transform-only group being the common case in a real scene graph -- shifted
  /// the counter, so every mesh after it silently read the wrong `colors` entry, or, once the
  /// counter ran past the end of `colors`, fell back to the magenta default. No panic, no
  /// error: just a wrong-colored mesh.
  ///
  /// ## Why Not Caught
  /// Every existing caller (the `animation_surface_rendering`, `lottie_surface_rendering`, and
  /// `curve_surface_rendering` examples) happens to build scenes where every node is a mesh
  /// node, so the traversal-position counter and the mesh-encounter counter were always
  /// numerically identical and the desync never manifested. Nothing exercised a scene mixing
  /// mesh and non-mesh nodes.
  ///
  /// ## Fix Applied
  /// Extracted the mesh-to-color resolution into `resolve_mesh_colors`, which indexes `colors`
  /// by `resolved.len()` -- a count that only grows when a mesh is actually pushed -- instead
  /// of a counter shared with every traversed node. `render` now calls this function once up
  /// front and walks its result in lockstep with a mesh-only counter during the real
  /// GL-drawing traversal.
  ///
  /// ## Prevention
  /// This test builds a scene with two top-level groups, each owning one mesh child, so a
  /// non-mesh node sits between the first and second mesh in traversal order -- exactly the
  /// shape that desyncs a traversal-position counter from a mesh-position counter. It fails
  /// immediately if the counter regresses to counting every node again.
  ///
  /// ## Pitfall
  /// When a lookup index is shared between a filtered consumer (only meshes read it) and an
  /// unfiltered traversal (every node advances it), the two stay accidentally in sync only
  /// while the "skipped" case never actually occurs in test data. Count only what is actually
  /// consumed, never everything visited.
  #[ test ]
  fn resolve_mesh_colors_stays_in_sync_across_non_mesh_siblings()
  {
    // scene
    // |- group_1 (non-mesh)
    // |   `- mesh_1
    // `- group_2 (non-mesh)
    //     `- mesh_2
    let mut scene = Scene::new();

    let group_1 = group_node();
    group_1.borrow_mut().add_child( mesh_node() );

    let group_2 = group_node();
    group_2.borrow_mut().add_child( mesh_node() );

    scene.add( group_1 );
    scene.add( group_2 );

    let color_for_mesh_1 = F32x4::from_array( [ 1.0, 0.0, 0.0, 1.0 ] );
    let color_for_mesh_2 = F32x4::from_array( [ 0.0, 1.0, 0.0, 1.0 ] );
    let colors = [ color_for_mesh_1, color_for_mesh_2 ];

    let resolved = resolve_mesh_colors( &scene, &colors );

    assert_eq!( resolved.len(), 2, "expected exactly one resolved color per mesh" );
    assert_eq!
    (
      resolved[ 0 ], color_for_mesh_1,
      "first mesh encountered must get colors[0], not a color shifted by the preceding non-mesh group"
    );
    assert_eq!
    (
      resolved[ 1 ], color_for_mesh_2,
      "second mesh encountered must get colors[1], not fall back to the default color"
    );
  }
}

crate::mod_interface!
{
  orphan use
  {
    CanvasRenderer
  };
}
