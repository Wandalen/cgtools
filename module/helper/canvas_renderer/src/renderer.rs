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
  fn framebuffer_create
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

    // Fix(BUG-227): propagate `None` via `?` instead of `.unwrap()`.
    // Root cause: `create_texture`/`create_framebuffer` in this same function already honor
    // the doc comment's "or `None` if creation fails" contract via `?`, but this call used
    // `.unwrap()` -- the identical WebGL failure class (context loss) that its siblings
    // handle gracefully instead panics here.
    // Pitfall: when several calls of the same resource-creation shape sit in one function,
    // fixing the contract on some of them doesn't guarantee the rest were caught -- audit
    // every call of that shape in the function, not just the ones that already look handled.
    let depthbuffer = gl.create_renderbuffer()?;
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
  ///
  /// # Panics
  ///
  /// Panics only if [`Scene::traverse`] reports an error, which cannot happen here : the
  /// visitor passed to it is infallible.
  // Fix(TASK-016): index `colors` by `resolved.len()` -- the count of meshes already resolved
  // -- instead of a counter shared with every traversed node.
  // Root cause: the lookup index previously advanced once per traversed node (mesh or not),
  // but was only ever read while visiting a mesh, so any non-mesh node visited before or
  // between mesh nodes shifted every later mesh onto the wrong `colors` entry.
  // Pitfall: when a lookup index is shared between a filtered consumer (only meshes read it)
  // and an unfiltered traversal (every node advances it), the two silently drift apart the
  // moment a "skipped" item actually occurs -- count only what is actually consumed.
  #[ must_use ]
  pub fn mesh_colors_resolve( scene : &Scene, colors : &[ F32x4 ] ) -> Vec< F32x4 >
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

      let Some( ( framebuffer, output_texture ) ) = framebuffer_create( gl, width, height )
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
    fn camera_upload( &self, gl : &GL, camera : &Camera )
    {
      gl::uniform::matrix_upload
      (
        gl,
        self.uniforms.get( "viewMatrix" ).unwrap().clone(),
        &camera.view_matrix_get().to_array(),
        true
      ).unwrap();

      gl::uniform::matrix_upload
      (
        gl,
        self.uniforms.get( "projectionMatrix" ).unwrap().clone(),
        &camera.projection_matrix_get().to_array(),
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
    pub fn node_upload
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
        node.borrow().world_matrix_get().to_array().as_slice(),
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
      scene.world_matrix_update();

      // Fix(BUG-493)
      // Root cause: this function unconditionally overwrites 4 pieces of global GL state
      // (`DEPTH_TEST`/`BLEND` enable flags, `depth_mask`, `front_face`) below, but -- unlike the
      // framebuffer binding restored near the end of this function (Fix(BUG-342)) -- never
      // restored any of them before returning. Any caller that had its own GL state in place
      // before calling `render()` (e.g. `BLEND` enabled for its own transparent pass, or `CW`
      // front-face winding) silently had that state overwritten and left overwritten after
      // `render()` returned, with no error or indication anywhere.
      // Pitfall: `render()` already restores one piece of global state it mutates (the
      // framebuffer binding, per BUG-342) -- fixing that one restore doesn't guarantee the rest
      // of the state this function mutates is also restored; each piece of global GL state a
      // function changes has to be individually audited for its own snapshot/restore, never
      // assumed covered just because a sibling piece of state already looks handled.
      let depth_test_was_enabled = gl.is_enabled( gl::DEPTH_TEST );
      let blend_was_enabled = gl.is_enabled( gl::BLEND );
      let depth_mask_was_enabled = gl.get_parameter( gl::DEPTH_WRITEMASK )
      .ok()
      .and_then( | v | v.as_bool() )
      .unwrap_or( true );
      let front_face_was = gl.get_parameter( gl::FRONT_FACE )
      .ok()
      .and_then( | v | v.as_f64() )
      .map_or( gl::CCW, | v | v as u32 );

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

      // Resolved once, up front, in mesh-encounter order -- see `mesh_colors_resolve` for why
      // this can't be a counter shared with the node-traversal below.
      let mesh_colors = mesh_colors_resolve( scene, colors );
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

            self.camera_upload( gl, camera );
            self.node_upload( gl, &node );

            primitive.geometry.borrow().bind( gl );
            primitive.draw( gl );
          }
        }

        Ok( () )
      };

      // Traverse the scene and draw all opaque objects.
      scene.traverse( &mut draw_node )?;

      // Fix(BUG-342)
      // Root cause: this function bound `self.framebuffer` above but never rebound the default
      // (`None`) framebuffer before returning, unlike its siblings `framebuffer_create` and
      // `texture_set`, which both explicitly restore `None` as their last GL state change.
      // WebGL's `bindFramebuffer` state persists on the context until explicitly changed, so any
      // GL call issued after `render()` returned, by code that didn't itself rebind first, would
      // silently target this internal offscreen texture instead of the intended target.
      // Pitfall: when several functions in the same file share a "bind non-default, do work,
      // restore default" shape, fixing the restore on some of them doesn't guarantee the rest
      // were caught -- each has to be individually audited against the shape it shares with its
      // siblings, not assumed consistent once most of them look handled.
      gl.bind_framebuffer( GL::FRAMEBUFFER, None );

      // Fix(BUG-493): restore the 4 global GL state bits snapshotted above.
      if depth_test_was_enabled { gl.enable( gl::DEPTH_TEST ); } else { gl.disable( gl::DEPTH_TEST ); }
      if blend_was_enabled { gl.enable( gl::BLEND ); } else { gl.disable( gl::BLEND ); }
      gl.depth_mask( depth_mask_was_enabled );
      gl.front_face( front_face_was );

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
    pub fn texture_set
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
    pub fn texture_get( &self ) -> WebGlTexture
    {
      self.output_texture.clone()
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    CanvasRenderer
  };

  own use
  {
    mesh_colors_resolve
  };
}
