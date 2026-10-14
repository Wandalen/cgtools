//! GPU id-buffer object picking over a renderer [`Scene`].
//!
//! [`ScenePicker::render`] draws every visible mesh node's pick id into an off-screen
//! `R16UI` texture owned by an [`IdBuffer`]; [`IdBuffer::read`] then reads a single
//! pixel back to find out what is under a given screen position. The GPU has already
//! resolved occlusion, so no CPU-side ray/triangle math or retained vertex data is
//! needed.
//!
//! Unlike `gpu_picking::Pickable`, which assumes one VAO per part and `UNSIGNED_INT`
//! indices, this draws the scene's own [`Primitive`]s through `Geometry::draw`, so it
//! handles every index type and non-indexed geometry the glTF loader can produce.
//!
//! The pass is meant to be rendered on demand — e.g. once when the displayed image
//! stops changing — and then read from many times: reading is a one-pixel
//! `readPixels`, rendering is a full scene draw.
//!
//! Limitations: skinning and morph targets are ignored (meshes are drawn in bind
//! pose), and every drawn fragment is opaque (no alpha-cutout `discard`).

mod private
{
  use minwebgl::{ self as gl, WebglError };
  use gl::
  {
    GL,
    F32x4x4,
    JsCast,
    web_sys::
    {
      js_sys::{ Int32Array, Uint32Array },
      WebGlFramebuffer,
      WebGlProgram,
      WebGlRenderbuffer,
      WebGlTexture,
      WebGlUniformLocation,
      WebGlVertexArrayObject,
    },
  };
  use crate::webgl::{ Node, Object3D, Scene };

  /// A pick id as stored in the id texture. `0` is reserved — see [`PICK_ID_NONE`].
  pub type PickId = u16;

  /// The id written for background pixels and for meshes that should occlude but never
  /// be reported as a hit. [`IdBuffer::read`] maps it to `None`.
  pub const PICK_ID_NONE : PickId = 0;

  /// Whether `( x, y )` names an in-bounds pixel of a `width`×`height` buffer.
  #[ must_use ]
  pub fn pick_in_bounds( x : i32, y : i32, width : u32, height : u32 ) -> bool
  {
    x >= 0 && y >= 0 && i64::from( x ) < i64::from( width ) && i64::from( y ) < i64::from( height )
  }

  /// Maps a raw readback value to a picked id: [`PICK_ID_NONE`] means "nothing here".
  /// Values above `u16::MAX` cannot come from an `R16UI` texture and are rejected too.
  #[ must_use ]
  pub fn readback_to_pick_id( raw : u32 ) -> Option< PickId >
  {
    PickId::try_from( raw ).ok().filter( | id | *id != PICK_ID_NONE )
  }

  /// Off-screen `R16UI` id texture plus a depth renderbuffer.
  ///
  /// Created without storage; [`IdBuffer::size_set`] allocates it, so a picker that is
  /// constructed but never used costs one framebuffer object and nothing else.
  #[ derive( Debug ) ]
  pub struct IdBuffer
  {
    gl : GL,
    framebuffer : Option< WebGlFramebuffer >,
    id_texture : Option< WebGlTexture >,
    depth_renderbuffer : Option< WebGlRenderbuffer >,
    width : u32,
    height : u32,
    // Reused for every read. `RGBA_INTEGER` + `UNSIGNED_INT` is the readback pair every
    // WebGL2 implementation must support for unsigned-integer colour buffers, so four
    // components are read even though only red is meaningful.
    readback : Uint32Array,
  }

  impl IdBuffer
  {
    /// Creates the framebuffer object. No texture storage is allocated yet.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if the framebuffer cannot be created (e.g. a lost context).
    pub fn new( gl : &GL ) -> Result< Self, WebglError >
    {
      let framebuffer = gl.create_framebuffer()
      .ok_or( WebglError::FailedToAllocateResource( "IdBuffer framebuffer" ) )?;

      Ok
      (
        Self
        {
          gl : gl.clone(),
          framebuffer : Some( framebuffer ),
          id_texture : None,
          depth_renderbuffer : None,
          width : 0,
          height : 0,
          readback : Uint32Array::new_with_length( 4 ),
        }
      )
    }

    /// Current storage size in pixels; `( 0, 0 )` before the first [`IdBuffer::size_set`].
    #[ must_use ]
    pub fn size_get( &self ) -> ( u32, u32 )
    {
      ( self.width, self.height )
    }

    /// (Re)allocates storage at `width`×`height`. A no-op when the size is unchanged;
    /// a zero dimension releases the storage.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if a texture or renderbuffer cannot be created, or if the
    /// resulting framebuffer is incomplete.
    pub fn size_set( &mut self, width : u32, height : u32 ) -> Result< (), WebglError >
    {
      if width == self.width && height == self.height { return Ok( () ); }

      self.storage_free();
      if width == 0 || height == 0 { return Ok( () ); }

      let gl = &self.gl;
      let w = i32::try_from( width ).map_err( | _ | WebglError::IdOutOfRange( format!( "IdBuffer width {width}" ) ) )?;
      let h = i32::try_from( height ).map_err( | _ | WebglError::IdOutOfRange( format!( "IdBuffer height {height}" ) ) )?;

      let id_texture = gl.create_texture()
      .ok_or( WebglError::FailedToAllocateResource( "IdBuffer id texture" ) )?;
      gl.bind_texture( GL::TEXTURE_2D, Some( &id_texture ) );
      gl.tex_storage_2d( GL::TEXTURE_2D, 1, GL::R16UI, w, h );
      gl::texture::d2::filter_nearest( gl );
      gl::texture::d2::wrap_clamp( gl );
      gl.bind_texture( GL::TEXTURE_2D, None );

      let Some( depth_renderbuffer ) = gl.create_renderbuffer() else
      {
        gl.delete_texture( Some( &id_texture ) );
        return Err( WebglError::FailedToAllocateResource( "IdBuffer depth renderbuffer" ) );
      };
      gl.bind_renderbuffer( GL::RENDERBUFFER, Some( &depth_renderbuffer ) );
      gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT16, w, h );
      gl.bind_renderbuffer( GL::RENDERBUFFER, None );

      let previous_framebuffer = framebuffer_binding_get( gl );
      gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
      gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &id_texture ), 0 );
      gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, Some( &depth_renderbuffer ) );
      let status = gl.check_framebuffer_status( GL::FRAMEBUFFER );
      gl.bind_framebuffer( GL::FRAMEBUFFER, previous_framebuffer.as_ref() );

      self.id_texture = Some( id_texture );
      self.depth_renderbuffer = Some( depth_renderbuffer );
      self.width = width;
      self.height = height;

      if status != GL::FRAMEBUFFER_COMPLETE
      {
        self.storage_free();
        return Err( WebglError::Other( "IdBuffer framebuffer incomplete" ) );
      }

      Ok( () )
    }

    /// Reads the id at `( x, y )` — bottom-up pixel coordinates, matching `readPixels`.
    ///
    /// Returns `None` for background / occluder-only pixels, for out-of-bounds
    /// coordinates, before storage exists, and if the readback fails (e.g. a lost
    /// context). Reading never renders; it returns whatever the last
    /// [`ScenePicker::render`] into this buffer produced.
    #[ must_use ]
    pub fn read( &self, x : i32, y : i32 ) -> Option< PickId >
    {
      if self.id_texture.is_none() || !pick_in_bounds( x, y, self.width, self.height ) { return None; }

      let gl = &self.gl;
      let previous_framebuffer = framebuffer_binding_get( gl );
      gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
      gl.read_buffer( GL::COLOR_ATTACHMENT0 );
      let result = gl.read_pixels_with_opt_array_buffer_view
      (
        x, y, 1, 1, GL::RGBA_INTEGER, GL::UNSIGNED_INT, Some( &self.readback )
      );
      gl.bind_framebuffer( GL::FRAMEBUFFER, previous_framebuffer.as_ref() );

      result.ok()?;
      readback_to_pick_id( self.readback.get_index( 0 ) )
    }

    fn storage_free( &mut self )
    {
      if let Some( texture ) = self.id_texture.take() { self.gl.delete_texture( Some( &texture ) ); }
      if let Some( rb ) = self.depth_renderbuffer.take() { self.gl.delete_renderbuffer( Some( &rb ) ); }
      self.width = 0;
      self.height = 0;
    }
  }

  impl Drop for IdBuffer
  {
    fn drop( &mut self )
    {
      self.storage_free();
      if let Some( framebuffer ) = self.framebuffer.take() { self.gl.delete_framebuffer( Some( &framebuffer ) ); }
    }
  }

  /// Draws a [`Scene`]'s pick ids into an [`IdBuffer`].
  #[ derive( Debug ) ]
  pub struct ScenePicker
  {
    gl : GL,
    program : WebGlProgram,
    view_projection_location : Option< WebGlUniformLocation >,
    world_matrix_location : Option< WebGlUniformLocation >,
    pick_id_location : Option< WebGlUniformLocation >,
  }

  impl ScenePicker
  {
    /// Compiles the id program.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if the id shaders fail to compile or link.
    pub fn new( gl : &GL ) -> Result< Self, WebglError >
    {
      let program = gl::Program::compile_and_link
      (
        gl,
        include_str!( "shaders/picking/id.vert" ),
        include_str!( "shaders/picking/id.frag" ),
      )?;

      Ok
      (
        Self
        {
          gl : gl.clone(),
          view_projection_location : gl.get_uniform_location( &program, "viewProjectionMatrix" ),
          world_matrix_location : gl.get_uniform_location( &program, "worldMatrix" ),
          pick_id_location : gl.get_uniform_location( &program, "pickId" ),
          program,
        }
      )
    }

    /// Renders every visible mesh node of `scene` into `buffer`, which must already be
    /// sized (see [`IdBuffer::size_set`]).
    ///
    /// `id_of` is asked once per visible mesh node. Return [`PICK_ID_NONE`] for nodes
    /// that must still occlude — a ground plane, say — but never be reported as a hit.
    /// Invisible nodes are skipped, but their children are still visited, matching
    /// `Renderer::render`.
    ///
    /// World matrices are taken as they are; call `Scene::world_matrix_update` first
    /// if transforms changed.
    ///
    /// GL state this pass touches — framebuffer binding, viewport, program, vertex
    /// array, depth test/func/mask, colour mask, cull face, blend and scissor — is
    /// restored before returning, including on error.
    ///
    /// # Errors
    ///
    /// Returns `WebglError` if `buffer` has no storage, or a uniform upload fails.
    pub fn render< F >
    (
      &self,
      buffer : &IdBuffer,
      scene : &Scene,
      view_projection : F32x4x4,
      mut id_of : F,
    ) -> Result< (), WebglError >
    where
      F : FnMut( &Node ) -> PickId,
    {
      if buffer.id_texture.is_none() { return Err( WebglError::Other( "IdBuffer has no storage; call size_set first" ) ); }

      let gl = &self.gl;
      let saved = GlState::capture( gl );

      let result = ( || -> Result< (), WebglError >
      {
        gl.bind_framebuffer( GL::FRAMEBUFFER, buffer.framebuffer.as_ref() );
        #[ allow( clippy::cast_possible_wrap, reason = "size_set already rejected dimensions above i32::MAX" ) ]
        gl.viewport( 0, 0, buffer.width as i32, buffer.height as i32 );

        gl.enable( GL::DEPTH_TEST );
        gl.depth_func( GL::LESS );
        gl.depth_mask( true );
        gl.color_mask( true, true, true, true );
        // Picking must not depend on winding or material cull settings: a double-sided
        // or inward-facing surface that is visible on screen has to be pickable too.
        gl.disable( GL::CULL_FACE );
        gl.disable( GL::BLEND );
        gl.disable( GL::SCISSOR_TEST );

        gl.clear_bufferuiv_with_u32_array( GL::COLOR, 0, &[ u32::from( PICK_ID_NONE ); 4 ] );
        gl.clear_bufferfv_with_f32_array( GL::DEPTH, 0, &[ 1.0 ] );

        gl.use_program( Some( &self.program ) );
        gl::uniform::matrix_upload( gl, self.view_projection_location.clone(), view_projection.to_array().as_slice(), true )?;

        scene.traverse
        (
          &mut | node |
          {
            let node = node.borrow();
            if !node.is_visible() { return Ok( () ); }
            let Object3D::Mesh( mesh ) = &node.object else { return Ok( () ); };

            let id = id_of( &node );
            gl::uniform::matrix_upload( gl, self.world_matrix_location.clone(), node.world_matrix_get().to_array().as_slice(), true )?;
            gl.uniform1ui( self.pick_id_location.as_ref(), u32::from( id ) );

            for primitive in &mesh.borrow().primitives
            {
              let primitive = primitive.borrow();
              let geometry = primitive.geometry.borrow();
              geometry.bind( gl );
              geometry.draw( gl );
            }

            Ok( () )
          }
        )
      } )();

      saved.restore( gl );
      result
    }
  }

  impl Drop for ScenePicker
  {
    fn drop( &mut self )
    {
      self.gl.delete_program( Some( &self.program ) );
    }
  }

  fn framebuffer_binding_get( gl : &GL ) -> Option< WebGlFramebuffer >
  {
    gl.get_parameter( GL::FRAMEBUFFER_BINDING ).ok()?.dyn_into::< WebGlFramebuffer >().ok()
  }

  /// The GL state [`ScenePicker::render`] changes, captured so it can be put back.
  /// Queries are CPU-side state lookups; none of them forces a GPU sync.
  struct GlState
  {
    framebuffer : Option< WebGlFramebuffer >,
    viewport : [ i32; 4 ],
    program : Option< WebGlProgram >,
    vertex_array : Option< WebGlVertexArrayObject >,
    depth_test : bool,
    depth_func : u32,
    depth_mask : bool,
    color_mask : [ bool; 4 ],
    cull_face : bool,
    blend : bool,
    scissor_test : bool,
  }

  impl GlState
  {
    fn capture( gl : &GL ) -> Self
    {
      let viewport = gl.get_parameter( GL::VIEWPORT ).ok()
      .and_then( | v | v.dyn_into::< Int32Array >().ok() )
      .map_or( [ 0, 0, gl.drawing_buffer_width(), gl.drawing_buffer_height() ], | v |
      {
        [ v.get_index( 0 ), v.get_index( 1 ), v.get_index( 2 ), v.get_index( 3 ) ]
      } );

      let color_mask = gl.get_parameter( GL::COLOR_WRITEMASK ).ok()
      .and_then( | v | v.dyn_into::< gl::js_sys::Array >().ok() )
      .map_or( [ true; 4 ], | v | core::array::from_fn( | i | v.get( i as u32 ).as_bool().unwrap_or( true ) ) );

      Self
      {
        framebuffer : framebuffer_binding_get( gl ),
        viewport,
        program : gl.get_parameter( GL::CURRENT_PROGRAM ).ok().and_then( | v | v.dyn_into().ok() ),
        vertex_array : gl.get_parameter( GL::VERTEX_ARRAY_BINDING ).ok().and_then( | v | v.dyn_into().ok() ),
        depth_test : gl.is_enabled( GL::DEPTH_TEST ),
        #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "DEPTH_FUNC is a GLenum reported as a JS number" ) ]
        depth_func : gl.get_parameter( GL::DEPTH_FUNC ).ok().and_then( | v | v.as_f64() ).map_or( GL::LESS, | v | v as u32 ),
        depth_mask : gl.get_parameter( GL::DEPTH_WRITEMASK ).ok().and_then( | v | v.as_bool() ).unwrap_or( true ),
        color_mask,
        cull_face : gl.is_enabled( GL::CULL_FACE ),
        blend : gl.is_enabled( GL::BLEND ),
        scissor_test : gl.is_enabled( GL::SCISSOR_TEST ),
      }
    }

    fn restore( self, gl : &GL )
    {
      let toggle = | cap : u32, on : bool | if on { gl.enable( cap ) } else { gl.disable( cap ) };

      gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
      let [ x, y, w, h ] = self.viewport;
      gl.viewport( x, y, w, h );
      gl.use_program( self.program.as_ref() );
      gl.bind_vertex_array( self.vertex_array.as_ref() );
      toggle( GL::DEPTH_TEST, self.depth_test );
      gl.depth_func( self.depth_func );
      gl.depth_mask( self.depth_mask );
      let [ r, g, b, a ] = self.color_mask;
      gl.color_mask( r, g, b, a );
      toggle( GL::CULL_FACE, self.cull_face );
      toggle( GL::BLEND, self.blend );
      toggle( GL::SCISSOR_TEST, self.scissor_test );
    }
  }
}

crate::mod_interface!
{
  own use
  {
    PickId,
    PICK_ID_NONE,
    pick_in_bounds,
    readback_to_pick_id,
    IdBuffer,
    ScenePicker,
  };
}
