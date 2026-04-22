//! WebGL backend adapter.
//!
//! Hardware-accelerated 2D rendering via WebGL2 (wasm32 target).
//! Uses `minwebgl` for GL calls. Quad vertices are generated in
//! the vertex shader via `gl_VertexID` — no quad VAO needed.

mod private
{
  use std::rc::Rc;
  use core::cell::{ Cell, RefCell };
  use web_sys::HtmlImageElement;
  use wasm_bindgen::prelude::*;
  use minwebgl as gl;
  use super::webgl_helpers::
  {
    ArrayBuffer,
    SpriteInstanceData,
    MeshInstanceData,
    GpuResources,
    GpuTexture,
    GpuSprite,
    GpuGeometry,
    GpuBatch,
    setup_sprite_batch_vao,
    setup_mesh_batch_vao,
    apply_blend,
    source_to_loadable,
    resolve_loadable,
    index_format,
    apply_texture_filter,
    topology_to_gl,
  };
  use crate::assets::Assets;
  use crate::backend::*;
  use crate::commands::*;
  use crate::types::*;

  // ============================================================================
  // Sprite renderer
  // ============================================================================

  /// Handles single sprite draws and sprite batch instancing.
  /// Quad is generated in vertex shader from `gl_VertexID` (triangle strip, 4 vertices).
  struct SpriteRenderer
  {
    program : gl::Program,
    batch_program : gl::Program,
  }

  impl SpriteRenderer
  {
    fn new( gl : &gl::GL ) -> Result< Self, gl::WebglError >
    {
      let program = gl::Program::new
      (
        gl.clone(),
        include_str!( "shaders/sprite.vert" ),
        include_str!( "shaders/sprite.frag" ),
      )?;
      let batch_program = gl::Program::new
      (
        gl.clone(),
        include_str!( "shaders/sprite_batch.vert" ),
        include_str!( "shaders/sprite_batch.frag" ),
      )?;
      Ok( Self { program, batch_program } )
    }

    /// Draw a single sprite as a textured quad (triangle strip, 4 vertices from `gl_VertexID`).
    fn draw( &self, gl : &gl::GL, transform : &[ f32; 9 ], uv_rect : &[ f32; 4 ], sprite_size : &[ f32; 2 ], tint : &[ f32; 4 ], viewport : &[ f32; 2 ], depth : f32 )
    {
      // Unbind any VAO to prevent stale attribute state from interfering
      gl.bind_vertex_array( None );
      self.program.activate();
      self.program.uniform_matrix_upload( "u_transform", transform.as_slice(), true );
      self.program.uniform_upload( "u_uv_rect", uv_rect );
      self.program.uniform_upload( "u_sprite_size", sprite_size );
      self.program.uniform_upload( "u_tint", tint );
      self.program.uniform_upload( "u_viewport", viewport );
      self.program.uniform_upload( "u_depth", &depth );
      gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );
    }

    /// Draw an instanced sprite batch.
    fn draw_batch( &self, gl : &gl::GL, batch : &GpuBatch, resources : &GpuResources, viewport : &[ f32; 2 ] )
    {
      let GpuBatch::Sprite { instances, vao, params, .. } = batch else { return; };
      if instances.is_empty() { return; }

      let Some( gpu_tex ) = resources.texture( params.sheet ) else { return; };
      let tw = gpu_tex.width.get();
      let th = gpu_tex.height.get();
      if tw == 0 || th == 0 { return; }

      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );

      self.batch_program.activate();
      self.batch_program.uniform_upload( "u_viewport", viewport );
      self.batch_program.uniform_upload( "u_tex_size", &[ tw as f32, th as f32 ] );
      let parent_mat = params.transform.to_mat3();
      self.batch_program.uniform_matrix_upload( "u_parent", &parent_mat, true );
      self.batch_program.uniform_upload( "u_parent_depth", &params.transform.depth );

      gl.bind_vertex_array( Some( vao ) );
      gl.draw_arrays_instanced( gl::TRIANGLE_STRIP, 0, 4, instances.len() as i32 );
      // Unbind the batch VAO so subsequent GL state setup (e.g. a later
      // vertex_attrib_pointer call during batch construction) cannot
      // accidentally mutate this batch's attribute layout. Matches the
      // single-draw path which also leaves VAO 0 bound.
      gl.bind_vertex_array( None );
    }
  }

  // ============================================================================
  // Mesh renderer
  // ============================================================================

  /// Handles single mesh draws and mesh batch instancing.
  struct MeshRenderer
  {
    program : gl::Program,
    batch_program : gl::Program,
  }

  impl MeshRenderer
  {
    fn new( gl : &gl::GL ) -> Result< Self, gl::WebglError >
    {
      let program = gl::Program::new
      (
        gl.clone(),
        include_str!( "shaders/mesh.vert" ),
        include_str!( "shaders/mesh.frag" ),
      )?;
      let batch_program = gl::Program::new
      (
        gl.clone(),
        include_str!( "shaders/mesh_batch.vert" ),
        include_str!( "shaders/mesh.frag" ),
      )?;
      Ok( Self { program, batch_program } )
    }

    /// Draw a single mesh.
    fn draw
    (
      &self,
      gl : &gl::GL,
      geom : &GpuGeometry,
      transform : &[ f32; 9 ],
      color : &[ f32; 4 ],
      topology : u32,
      viewport : &[ f32; 2 ],
      use_texture : bool,
      depth : f32,
    )
    {
      self.program.activate();
      self.program.uniform_matrix_upload( "u_transform", transform.as_slice(), true );
      self.program.uniform_upload( "u_color", color );
      self.program.uniform_upload( "u_viewport", viewport );
      self.program.uniform_upload( "u_use_texture", &i32::from( use_texture ) );
      self.program.uniform_upload( "u_depth", &depth );

      gl.bind_vertex_array( Some( &geom.vao ) );

      if let Some( ( count, gl_type ) ) = geom.index_count
      {
        gl.draw_elements_with_i32( topology, count as i32, gl_type, 0 );
      }
      else
      {
        gl.draw_arrays( topology, 0, geom.vertex_count as i32 );
      }
      // Unbind the geometry VAO so a subsequent `vertex_attrib_pointer` call
      // (e.g. during `setup_mesh_batch_vao` for another batch) cannot silently
      // mutate this geometry's attribute layout.
      gl.bind_vertex_array( None );
    }

    /// Draw an instanced mesh batch. VAO is already configured via `setup_mesh_batch_vao`.
    fn draw_batch( &self, gl : &gl::GL, batch : &GpuBatch, resources : &GpuResources, viewport : &[ f32; 2 ] )
    {
      let GpuBatch::Mesh { instances, vao, params, .. } = batch else { return };
      if instances.is_empty() { return; }

      let Some( geom ) = resources.geometry( params.geometry ) else { return };
      let color = match params.fill { FillRef::Solid( c ) => c, _ => [ 1.0, 1.0, 1.0, 1.0 ] };
      let topology = topology_to_gl( &params.topology );

      let mut use_texture = false;
      if let Some( tex_id ) = params.texture
        && let Some( gpu_tex ) = resources.texture( tex_id )
      {
        gl.active_texture( gl::TEXTURE0 );
        gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );
        use_texture = true;
      }

      self.batch_program.activate();
      self.batch_program.uniform_upload( "u_viewport", viewport );
      self.batch_program.uniform_upload( "u_color", &color );
      self.batch_program.uniform_upload( "u_use_texture", &i32::from( use_texture ) );
      let parent_mat = params.transform.to_mat3();
      self.batch_program.uniform_matrix_upload( "u_parent", &parent_mat, true );
      self.batch_program.uniform_upload( "u_parent_depth", &params.transform.depth );

      gl.bind_vertex_array( Some( vao ) );

      if let Some( ( count, gl_type ) ) = geom.index_count
      {
        gl.draw_elements_instanced_with_i32( topology, count as i32, gl_type, 0, instances.len() as i32 );
      }
      else
      {
        gl.draw_arrays_instanced( topology, 0, geom.vertex_count as i32, instances.len() as i32 );
      }
      // Unbind the batch VAO so subsequent GL state setup (e.g. a later
      // vertex_attrib_pointer call during batch construction) cannot
      // accidentally mutate this batch's attribute layout. The single-draw
      // path (`MeshRenderer::draw`) likewise unbinds on exit, so both mesh
      // draw paths leave VAO 0 bound.
      gl.bind_vertex_array( None );
    }
  }

  // ============================================================================
  // Backend struct
  // ============================================================================

  /// WebGL renderer backend.
  ///
  /// ```ignore
  /// let config = RenderConfig { width: 800, height: 600, ..Default::default() };
  /// let gl_ctx = minwebgl::context::from_canvas( &canvas )?;
  /// let mut backend = WebGlBackend::new( config, gl_ctx )?;
  /// backend.load_assets( &assets )?;
  /// backend.submit( &commands )?;
  /// ```
  pub struct WebGlBackend
  {
    config : RenderConfig,
    gl : gl::GL,
    resources : Rc< RefCell< GpuResources > >,
    sprite : SpriteRenderer,
    mesh : MeshRenderer,
    max_texture_size : u32,

    // -- batch editing state --
    recording_batch : Option< ResourceId< Batch > >,
  }

  impl WebGlBackend
  {
    /// Creates a new WebGL backend.
    ///
    /// **Antialiasing note:** MSAA in WebGL2 is controlled by the `antialias` attribute
    /// passed to `getContext("webgl2", { antialias: true })` at context creation time,
    /// not by `RenderConfig::antialias`. That field is only meaningful for the SVG adapter.
    /// Pass the desired AA setting when creating the WebGL2 context before calling this.
    ///
    /// **Depth buffer note:** `DEPTH_TEST` is enabled here so `Transform::depth` takes
    /// effect (higher → drawn on top). A depth attachment is required; `getContext`
    /// provides one by default (`depth: true` is the WebGL default). If the context
    /// was created with `depth: false`, depth testing is silently a no-op. Depth is
    /// only reliable for fully opaque draws — see `Transform::depth` for the
    /// transparency caveat.
    ///
    /// # Errors
    /// Returns error if shader compilation fails.
    pub fn new( config : RenderConfig, gl : gl::GL ) -> Result< Self, RenderError >
    {
      let map_err = | e : gl::WebglError | RenderError::BackendError( format!( "{e:?}" ) );

      let sprite = SpriteRenderer::new( &gl ).map_err( map_err )?;
      let mesh = MeshRenderer::new( &gl ).map_err( map_err )?;

      // Initial GL state
      gl.viewport( 0, 0, config.width as i32, config.height as i32 );
      gl.enable( gl::BLEND );
      // Use separate factors for the alpha channel so the framebuffer alpha follows
      // the Porter-Duff "over" rule: a = src_a + dst_a * (1 - src_a). Using the same
      // SRC_ALPHA factor on alpha would yield src_a^2 + dst_a*(1-src_a), corrupting
      // alpha when the canvas is composited against a transparent page or read via
      // readPixels.
      gl.blend_func_separate( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ONE_MINUS_SRC_ALPHA );

      // LEQUAL (not LESS) so equal-depth draws fall back to submission order rather
      // than rejecting the second one — keeps the default (all depth = 0) case
      // rendering identically to the pre-depth implementation.
      gl.enable( gl::DEPTH_TEST );
      gl.depth_func( gl::LEQUAL );

      // Query the actual hardware limit; fall back to the WebGL2 guaranteed minimum.
      // get_parameter returns a JsValue; as_f64() is the idiomatic way to extract it.
      // u32::try_from(i64) rejects negatives; the i64 intermediate avoids cast_sign_loss.
      let max_texture_size : u32 = gl
        .get_parameter( gl::MAX_TEXTURE_SIZE )
        .ok()
        .and_then( | v | v.as_f64() )
        .and_then( | v | u32::try_from( v as i64 ).ok() )
        .unwrap_or( 2048 );

      Ok( Self
      {
        config,
        gl,
        resources : Rc::new( RefCell::new( GpuResources::new() ) ),
        sprite,
        mesh,
        max_texture_size,
        recording_batch : None,
      })
    }

    fn viewport_size( &self ) -> [ f32; 2 ]
    {
      [ self.config.width as f32, self.config.height as f32 ]
    }

    // ---- Command handlers ----
    //
    // Contract for batch-targeting commands (cmd_add_*_instance, cmd_set_*_instance,
    // cmd_set_*_batch_params, cmd_remove_instance, cmd_draw_batch):
    //
    // - `recording_batch == None` (no active BindBatch, or called after UnbindBatch):
    //   silently return Ok(()). Mid-frame state transitions can legitimately leave
    //   this slot empty; making every add/set/remove a hard error would require the
    //   caller to mirror the bind-state machine.
    //
    // - Referenced id not found in the resource map (batch / sprite / geometry):
    //   emit a console.warn and return Ok(()). Async asset loading can legitimately
    //   leave an id unresolved for a short window, and we prefer a visible
    //   diagnostic over a hard error that would tear down the whole submit().
    //
    // - Referenced batch exists but has the WRONG variant (Sprite-targeting command
    //   hits a Mesh batch or vice versa): return Err. Batch variant is assigned at
    //   CreateSpriteBatch / CreateMeshBatch time — synchronous and never racy — so a
    //   mismatch is a genuine caller bug that should surface immediately.

    fn cmd_clear( &self, c : &Clear )
    {
      let [ r, g, b, a ] = c.color;
      self.gl.clear_color( r, g, b, a );
      self.gl.clear_depth( 1.0 );
      self.gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );
    }

    fn cmd_mesh( &self, m : &Mesh, viewport : &[ f32; 2 ] )
    {
      let res = self.resources.borrow();
      if let Some( geom ) = res.geometry( m.geometry )
      {
        let mat = m.transform.to_mat3();
        let color = match m.fill { FillRef::Solid( c ) => c, _ => [ 1.0, 1.0, 1.0, 1.0 ] };
        apply_blend( &self.gl, &m.blend );

        let mut use_texture = false;
        if let Some( tex_id ) = m.texture && let Some( gpu_tex ) = res.texture( tex_id )
        {
          self.gl.active_texture( gl::TEXTURE0 );
          self.gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );
          use_texture = true;
        }

        self.mesh.draw( &self.gl, geom, &mat, &color, topology_to_gl( &m.topology ), viewport, use_texture, m.transform.depth );
      }
    }

    fn cmd_sprite( &self, s : &Sprite, viewport : &[ f32; 2 ] )
    {
      let res = self.resources.borrow();
      let Some( gpu_sprite ) = res.sprite( s.sprite ) else { return };
      let Some( gpu_tex ) = res.texture( gpu_sprite.sheet ) else { return };

      let tw = gpu_tex.width.get();
      let th = gpu_tex.height.get();
      if tw == 0 || th == 0 { return; } // image not loaded yet

      self.gl.active_texture( gl::TEXTURE0 );
      self.gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );

      let [ rx, ry, rw, rh ] = gpu_sprite.region;
      let tw = tw as f32;
      let th = th as f32;
      let uv_rect = [ rx / tw, ry / th, rw / tw, rh / th ];
      let sprite_size = [ rw, rh ];

      let mat = s.transform.to_mat3();
      apply_blend( &self.gl, &s.blend );
      self.sprite.draw( &self.gl, &mat, &uv_rect, &sprite_size, &s.tint, viewport, s.transform.depth );
    }

    fn cmd_create_sprite_batch( &mut self, cmd : &CreateSpriteBatch ) -> Result< (), RenderError >
    {
      let map_err = | e : gl::WebglError | RenderError::BackendError( format!( "{e:?}" ) );
      let gl = &self.gl;
      let instances = ArrayBuffer::< SpriteInstanceData >::new( gl, 16 ).map_err( map_err )?;
      let vao = gl::vao::create( gl ).map_err( map_err )?;
      setup_sprite_batch_vao( gl, &vao, instances.buffer() );
      self.resources.borrow_mut().store_batch( cmd.batch, GpuBatch::Sprite
      {
        gl : self.gl.clone(),
        instances,
        vao,
        params : cmd.params,
      });
      Ok( () )
    }

    fn cmd_create_mesh_batch( &mut self, cmd : &CreateMeshBatch ) -> Result< (), RenderError >
    {
      let map_err = | e : gl::WebglError | RenderError::BackendError( format!( "{e:?}" ) );
      let gl = &self.gl;
      let instances = ArrayBuffer::< MeshInstanceData >::new( gl, 16 ).map_err( map_err )?;
      let vao = gl::vao::create( gl ).map_err( map_err )?;
      let res = self.resources.borrow();
      if let Some( geom ) = res.geometry( cmd.params.geometry )
      {
        setup_mesh_batch_vao
        (
          gl,
          &vao,
          geom.position_buffer.as_ref(),
          geom.uv_buffer.as_ref(),
          geom.index_buffer.as_ref(),
          instances.buffer(),
        );
      }
      drop( res );
      self.resources.borrow_mut().store_batch( cmd.batch, GpuBatch::Mesh
      {
        gl : self.gl.clone(),
        instances,
        vao,
        params : cmd.params,
      });
      Ok( () )
    }

    fn cmd_bind_batch( &mut self, cmd : &BindBatch ) -> Result< (), RenderError >
    {
      if let Some( current ) = self.recording_batch
      {
        return Err( RenderError::BackendError
        (
          format!( "BindBatch({:?}): batch {:?} is already bound; call UnbindBatch first", cmd.batch, current )
        ));
      }
      self.recording_batch = Some( cmd.batch );
      Ok( () )
    }

    fn cmd_add_sprite_instance( &mut self, si : &AddSpriteInstance ) -> Result< (), RenderError >
    {
      let Some( batch_id ) = self.recording_batch else { return Ok( () ) };
      let mut res = self.resources.borrow_mut();
      let Some( region ) = res.sprite( si.sprite ).map( | s | s.region )
      else
      {
        web_sys::console::warn_1
        (
          &format!( "AddSpriteInstance: sprite {:?} not found (dropped)", si.sprite ).into()
        );
        return Ok( () );
      };
      let data = SpriteInstanceData
      {
        transform : si.transform.to_mat3(),
        region,
        tint : si.tint,
        depth : si.transform.depth,
      };
      match res.batch_mut( batch_id )
      {
        Some( GpuBatch::Sprite { instances, .. } ) =>
          instances.push( &data ).map_err( | e | RenderError::BackendError( e.to_string() ) )?,
        Some( GpuBatch::Mesh { .. } ) => return Err( RenderError::BackendError
        (
          format!( "AddSpriteInstance: batch {batch_id:?} is a Mesh batch; sprite instances require a Sprite batch" )
        )),
        None =>
        {
          web_sys::console::warn_1
          (
            &format!( "AddSpriteInstance: batch {batch_id:?} not found (dropped)" ).into()
          );
        }
      }
      Ok( () )
    }

    fn cmd_add_mesh_instance( &mut self, mi : &AddMeshInstance ) -> Result< (), RenderError >
    {
      let Some( batch_id ) = self.recording_batch else { return Ok( () ) };
      let data = MeshInstanceData { transform : mi.transform.to_mat3(), depth : mi.transform.depth };
      let mut res = self.resources.borrow_mut();
      match res.batch_mut( batch_id )
      {
        Some( GpuBatch::Mesh { instances, .. } ) =>
          instances.push( &data ).map_err( | e | RenderError::BackendError( e.to_string() ) )?,
        Some( GpuBatch::Sprite { .. } ) => return Err( RenderError::BackendError
        (
          format!( "AddMeshInstance: batch {batch_id:?} is a Sprite batch; mesh instances require a Mesh batch" )
        )),
        None =>
        {
          web_sys::console::warn_1
          (
            &format!( "AddMeshInstance: batch {batch_id:?} not found (dropped)" ).into()
          );
        }
      }
      Ok( () )
    }

    fn cmd_set_sprite_instance( &mut self, si : &SetSpriteInstance ) -> Result< (), RenderError >
    {
      let Some( batch_id ) = self.recording_batch else { return Ok( () ) };
      let mut res = self.resources.borrow_mut();
      let Some( region ) = res.sprite( si.sprite ).map( | s | s.region )
      else
      {
        web_sys::console::warn_1
        (
          &format!( "SetSpriteInstance: sprite {:?} not found (dropped)", si.sprite ).into()
        );
        return Ok( () );
      };
      let data = SpriteInstanceData
      {
        transform : si.transform.to_mat3(),
        region,
        tint : si.tint,
        depth : si.transform.depth,
      };
      match res.batch_mut( batch_id )
      {
        Some( GpuBatch::Sprite { instances, .. } ) =>
        {
          if si.index >= instances.len()
          {
            return Err( RenderError::BackendError
            (
              format!( "SetSpriteInstance: index {} out of bounds (len {})", si.index, instances.len() )
            ));
          }
          instances.set( si.index, &data );
        }
        Some( GpuBatch::Mesh { .. } ) => return Err( RenderError::BackendError
        (
          format!( "SetSpriteInstance: batch {batch_id:?} is a Mesh batch; sprite instances require a Sprite batch" )
        )),
        None =>
        {
          web_sys::console::warn_1
          (
            &format!( "SetSpriteInstance: batch {batch_id:?} not found (dropped)" ).into()
          );
        }
      }
      Ok( () )
    }

    fn cmd_set_mesh_instance( &mut self, mi : &SetMeshInstance ) -> Result< (), RenderError >
    {
      let Some( batch_id ) = self.recording_batch else { return Ok( () ) };
      let data = MeshInstanceData { transform : mi.transform.to_mat3(), depth : mi.transform.depth };
      let mut res = self.resources.borrow_mut();
      match res.batch_mut( batch_id )
      {
        Some( GpuBatch::Mesh { instances, .. } ) =>
        {
          if mi.index >= instances.len()
          {
            return Err( RenderError::BackendError
            (
              format!( "SetMeshInstance: index {} out of bounds (len {})", mi.index, instances.len() )
            ));
          }
          instances.set( mi.index, &data );
        }
        Some( GpuBatch::Sprite { .. } ) => return Err( RenderError::BackendError
        (
          format!( "SetMeshInstance: batch {batch_id:?} is a Sprite batch; mesh instances require a Mesh batch" )
        )),
        None =>
        {
          web_sys::console::warn_1
          (
            &format!( "SetMeshInstance: batch {batch_id:?} not found (dropped)" ).into()
          );
        }
      }
      Ok( () )
    }

    fn cmd_remove_instance( &mut self, ri : &RemoveInstance ) -> Result< (), RenderError >
    {
      let Some( batch_id ) = self.recording_batch else { return Ok( () ) };
      let mut res = self.resources.borrow_mut();
      let Some( batch ) = res.batch_mut( batch_id )
      else
      {
        web_sys::console::warn_1
        (
          &format!( "RemoveInstance: batch {batch_id:?} not found (dropped)" ).into()
        );
        return Ok( () );
      };
      // RemoveInstance is polymorphic — it doesn't care whether the batch is
      // Sprite or Mesh, only that the index is in-bounds — so no type-mismatch
      // branch here.
      let len = match batch
      {
        GpuBatch::Sprite { instances, .. } => instances.len(),
        GpuBatch::Mesh { instances, .. } => instances.len(),
      };
      if ri.index >= len
      {
        return Err( RenderError::BackendError
        (
          format!( "RemoveInstance: index {} out of bounds (len {})", ri.index, len )
        ));
      }
      match batch
      {
        GpuBatch::Sprite { instances, .. } => { instances.swap_remove( ri.index ); },
        GpuBatch::Mesh { instances, .. } => { instances.swap_remove( ri.index ); },
      }
      Ok( () )
    }

    fn cmd_set_sprite_batch_params( &mut self, cmd : &SetSpriteBatchParams ) -> Result< (), RenderError >
    {
      let Some( batch_id ) = self.recording_batch else { return Ok( () ) };
      let mut res = self.resources.borrow_mut();
      match res.batch_mut( batch_id )
      {
        Some( GpuBatch::Sprite { params, .. } ) => { *params = cmd.params; }
        Some( GpuBatch::Mesh { .. } ) => return Err( RenderError::BackendError
        (
          format!( "SetSpriteBatchParams: batch {batch_id:?} is a Mesh batch" )
        )),
        None =>
        {
          web_sys::console::warn_1
          (
            &format!( "SetSpriteBatchParams: batch {batch_id:?} not found (dropped)" ).into()
          );
        }
      }
      Ok( () )
    }

    fn cmd_set_mesh_batch_params( &mut self, cmd : &SetMeshBatchParams ) -> Result< (), RenderError >
    {
      let Some( batch_id ) = self.recording_batch else { return Ok( () ) };
      let mut res = self.resources.borrow_mut();
      match res.batch_mut( batch_id )
      {
        Some( GpuBatch::Mesh { params, .. } ) => { *params = cmd.params; }
        Some( GpuBatch::Sprite { .. } ) => return Err( RenderError::BackendError
        (
          format!( "SetMeshBatchParams: batch {batch_id:?} is a Sprite batch" )
        )),
        None =>
        {
          web_sys::console::warn_1
          (
            &format!( "SetMeshBatchParams: batch {batch_id:?} not found (dropped)" ).into()
          );
        }
      }
      Ok( () )
    }

    fn cmd_unbind_batch( &mut self )
    {
      if let Some( batch_id ) = self.recording_batch.take()
      {
        let res = self.resources.borrow();
        if let Some( batch ) = res.batch( batch_id )
        {
          match batch
          {
            GpuBatch::Sprite { instances, vao, .. } =>
            {
              setup_sprite_batch_vao( &self.gl, vao, instances.buffer() );
            }
            GpuBatch::Mesh { instances, vao, params, .. } =>
            {
              if let Some( geom ) = res.geometry( params.geometry )
              {
                setup_mesh_batch_vao
                (
                  &self.gl,
                  vao,
                  geom.position_buffer.as_ref(),
                  geom.uv_buffer.as_ref(),
                  geom.index_buffer.as_ref(),
                  instances.buffer(),
                );
              }
            }
          }
        }
      }
    }

    fn cmd_draw_batch( &self, db : &DrawBatch, viewport : &[ f32; 2 ] ) -> Result< (), RenderError >
    {
      if self.recording_batch == Some( db.batch )
      {
        return Err( RenderError::BackendError
        (
          format!( "DrawBatch({:?}): batch is still bound; call UnbindBatch before drawing", db.batch )
        ));
      }
      let res = self.resources.borrow();
      let Some( gpu_batch ) = res.batch( db.batch )
      else
      {
        web_sys::console::warn_1
        (
          &format!( "DrawBatch: batch {:?} not found (dropped)", db.batch ).into()
        );
        return Ok( () );
      };
      apply_blend( &self.gl, match gpu_batch
      {
        GpuBatch::Sprite { params, .. } => &params.blend,
        GpuBatch::Mesh { params, .. } => &params.blend,
      });
      match gpu_batch
      {
        GpuBatch::Sprite { .. } => self.sprite.draw_batch( &self.gl, gpu_batch, &res, viewport ),
        GpuBatch::Mesh { .. } => self.mesh.draw_batch( &self.gl, gpu_batch, &res, viewport ),
      }
      Ok( () )
    }

    fn cmd_delete_batch( &mut self, db : &DeleteBatch )
    {
      // If the batch being deleted is currently bound, clear the recording slot so
      // subsequent instance commands do not silently target a dangling id.
      if self.recording_batch == Some( db.batch )
      {
        self.recording_batch = None;
      }
      // ArrayBuffer::drop handles GPU buffer cleanup.
      self.resources.borrow_mut().batches.remove( &db.batch );
    }

    // ---- Asset loading ----

    fn load_images( &mut self, images : &[ crate::assets::ImageAsset ] ) -> Result< (), RenderError >
    {
      let gl = &self.gl;
      self.resources.borrow_mut().textures.clear();

      for img in images
      {
        let ( texture, w, h ) = match &img.source
        {
          crate::assets::ImageSource::Bitmap { bytes, width, height, format } =>
          {
            let tex = gl.create_texture()
            .ok_or_else( || RenderError::BackendError( "failed to create texture".into() ) )?;

            gl.bind_texture( gl::TEXTURE_2D, Some( &tex ) );

            // Pick a WebGL2 format + upload bytes. Gray8 and GrayAlpha8 are
            // expanded to RGBA8 on the CPU before upload because:
            //
            //   1. WebGL1's LUMINANCE / LUMINANCE_ALPHA replicated the stored
            //      channels across RGB on sample. On WebGL2 they are legacy
            //      unsized formats backed by R8 / RG8 and sample as
            //      (L, 0, 0, 1) / (L, 0, 0, A) — grayscale images render red.
            //
            //   2. The obvious native GL ES 3.0 fix — R8 / RG8 + TEXTURE_SWIZZLE_*
            //      — is explicitly *removed* from WebGL2 (spec §6.19):
            //      TEXTURE_SWIZZLE_R/G/B/A are not valid `texParameteri` names
            //      and produce INVALID_ENUM.
            //
            // CPU expansion costs 4× memory for Gray8 / 2× for GrayAlpha8 at
            // upload time, which is acceptable for the grayscale images typical
            // in tilemap content (masks, icons, height fields) and is portable
            // across WebGL2 implementations without special GL state.
            let ( gl_fmt, unpack_alignment, bytes_owned ) : ( u32, i32, Option< Vec< u8 > > ) = match format
            {
              crate::assets::PixelFormat::Rgba8 => ( gl::RGBA, 4, None ),
              // RGB rows are 3*width bytes — may not be 4-aligned, so relax the
              // UNPACK stride to match. Restored below.
              crate::assets::PixelFormat::Rgb8  => ( gl::RGB, 1, None ),
              crate::assets::PixelFormat::Gray8 =>
              {
                let mut rgba = Vec::with_capacity( bytes.len() * 4 );
                for &l in bytes
                {
                  rgba.extend_from_slice( &[ l, l, l, 0xFF ] );
                }
                ( gl::RGBA, 4, Some( rgba ) )
              }
              crate::assets::PixelFormat::GrayAlpha8 =>
              {
                let mut rgba = Vec::with_capacity( bytes.len() * 2 );
                for pair in bytes.chunks_exact( 2 )
                {
                  let ( l, a ) = ( pair[ 0 ], pair[ 1 ] );
                  rgba.extend_from_slice( &[ l, l, l, a ] );
                }
                ( gl::RGBA, 4, Some( rgba ) )
              }
            };

            // Relax UNPACK_ALIGNMENT only when the per-row byte count may not be
            // a multiple of 4 (RGB8 at odd widths). Default 4 is correct for
            // RGBA8 and for the CPU-expanded grayscale paths above.
            if unpack_alignment != 4 { gl.pixel_storei( gl::UNPACK_ALIGNMENT, unpack_alignment ); }

            let upload_bytes : &[ u8 ] = bytes_owned.as_deref().unwrap_or( bytes );

            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
            (
              gl::TEXTURE_2D, 0, gl_fmt as i32,
              *width as i32, *height as i32, 0,
              gl_fmt, gl::UNSIGNED_BYTE, Some( upload_bytes ),
            )
            .map_err( | e | RenderError::BackendError
            (
              format!
              (
                "tex_image_2d failed for image {:?}: {:?}",
                img.id, e
              )
            ))?;

            // Restore default so later uploads aren't surprised by residual state.
            if unpack_alignment != 4 { gl.pixel_storei( gl::UNPACK_ALIGNMENT, 4 ); }

            ( tex, *width, *height )
          }
          crate::assets::ImageSource::Encoded( _ ) => { continue; } // qqq: decode
          crate::assets::ImageSource::Path( path ) =>
          {
            let path = path.as_path().to_str()
              .ok_or_else( || RenderError::BackendError( "non-UTF-8 image path".into() ) )?;
            // Async path: sampler state (filter, wrap, mipmap chain) is applied inside
            // the on_load callback after the image bytes are actually uploaded, so the
            // texture is guaranteed to be complete (esp. for mipmap modes, which leave
            // the texture incomplete until generate_mipmap runs).
            let generation = self.resources.borrow().generation;
            let tex = upload_image_from_path( gl, path, img.id, &self.resources, img.filter, img.mipmap, generation );
            gl.bind_texture( gl::TEXTURE_2D, Some( &tex ) );
            ( tex, 0, 0 )
          }
        };

        // Sync bitmap path: level 0 is already uploaded; apply sampler state and
        // generate the mip chain right away so the texture is immediately usable.
        // (The async Path branch does all of this inside on_load.)
        if matches!( img.source, crate::assets::ImageSource::Bitmap { .. } )
        {
          apply_texture_filter( gl, &img.filter, &img.mipmap );
          gl::texture::d2::wrap_clamp( gl );
          if !matches!( img.mipmap, MipmapMode::Off )
          {
            gl.generate_mipmap( gl::TEXTURE_2D );
          }
        }

        self.resources.borrow_mut().store_texture( img.id, GpuTexture
        {
          gl : gl.clone(),
          texture,
          width : Cell::new( w ),
          height : Cell::new( h ),
          filter : img.filter,
          mipmap : img.mipmap,
        });
      }

      Ok( () )
    }

    // Returns () unlike load_images/load_geometries because sprite loading is
    // infallible — it only stores sub-regions of already-loaded textures (no GPU
    // upload, no allocation that can fail).
    fn load_sprites( &mut self, sprites : &[ crate::assets::SpriteAsset ] )
    {
      self.resources.borrow_mut().sprites.clear();

      for spr in sprites
      {
        self.resources.borrow_mut().store_sprite( spr.id, GpuSprite
        {
          sheet : spr.sheet,
          region : spr.region,
        });
      }
    }

    fn load_geometries( &mut self, geometries : &[ crate::assets::GeometryAsset ] ) -> Result< (), RenderError >
    {
      let gl = &self.gl;
      let map_err = | e : gl::WebglError | RenderError::BackendError( format!( "{e:?}" ) );
      self.resources.borrow_mut().geometries.clear();

      for geom in geometries
      {
        // Validate index format early so both sync and async paths can use it.
        // geom.indices == None is fine (non-indexed draw); geom.data_type only matters when indices are present.
        let ( idx_stride, idx_gl_type ) = if geom.indices.is_some()
        {
          index_format( &geom.data_type )?
        }
        else
        {
          ( 0, 0 ) // unused when there are no indices
        };

        let has_path =
          matches!( geom.positions, crate::assets::Source::Path( _ ) )
          || matches!( geom.uvs, Some( crate::assets::Source::Path( _ ) ) )
          || matches!( geom.indices, Some( crate::assets::Source::Path( _ ) ) );

        if has_path
        {
          // Register a placeholder geometry immediately so the id is available.
          // The placeholder owns its own VAO (never shared): when `store_geometry`
          // later replaces it, its `Drop` deletes *this* VAO, not the populated one.
          // The spawn_local future creates a separate VAO for the populated entry.
          let placeholder_vao = gl::vao::create( gl ).map_err( map_err )?;
          self.resources.borrow_mut().store_geometry( geom.id, GpuGeometry
          {
            gl : gl.clone(), vao : placeholder_vao, position_buffer : None, uv_buffer : None, index_buffer : None,
            vertex_count : 0, index_count : None,
          });

          let gl_clone = gl.clone();
          let resources = Rc::clone( &self.resources );
          let id = geom.id;
          let generation = self.resources.borrow().generation;

          let positions_source = source_to_loadable( &geom.positions );
          let uvs_source = geom.uvs.as_ref().map( source_to_loadable );
          let indices_source = geom.indices.as_ref().map( source_to_loadable );

          gl::spawn_local( async move
          {
            let gl = &gl_clone;

            let positions = resolve_loadable( positions_source ).await;
            let uvs = match uvs_source { Some( s ) => Some( resolve_loadable( s ).await ), None => None };
            let indices = match indices_source { Some( s ) => Some( resolve_loadable( s ).await ), None => None };

            // Bail out if `load_assets` ran again while we were fetching — this future
            // belongs to a previous cycle and must not overwrite fresh entries.
            if resources.borrow().generation != generation { return; }

            // Create a fresh VAO for the populated entry — distinct from the placeholder's,
            // so placeholder drop can't delete the GPU object this entry depends on.
            let Ok( vao ) = gl::vao::create( gl ) else { return };

            gl.bind_vertex_array( Some( &vao ) );

            // Positions (attrib 0)
            let mut position_buffer = None;
            if let Some( ref bytes ) = positions
              && let Ok( buffer ) = gl::buffer::create( gl )
            {
              gl::buffer::upload( gl, &buffer, bytes, gl::STATIC_DRAW );
              gl.enable_vertex_attrib_array( 0 );
              gl.vertex_attrib_pointer_with_i32( 0, 2, gl::FLOAT, false, 0, 0 );
              position_buffer = Some( buffer );
            }

            // UVs (attrib 1)
            let mut uv_buffer = None;
            if let Some( Some( ref bytes ) ) = uvs
              && let Ok( buffer ) = gl::buffer::create( gl )
            {
              gl::buffer::upload( gl, &buffer, bytes, gl::STATIC_DRAW );
              gl.enable_vertex_attrib_array( 1 );
              gl.vertex_attrib_pointer_with_i32( 1, 2, gl::FLOAT, false, 0, 0 );
              uv_buffer = Some( buffer );
            }

            // Indices
            let mut index_buffer = None;
            let mut index_count = None;
            if let Some( Some( ref bytes ) ) = indices
              && let Ok( buffer ) = gl::buffer::create( gl )
            {
              gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, Some( &buffer ) );
              let u8_array = gl::js_sys::Uint8Array::from( bytes.as_slice() );
              gl.buffer_data_with_array_buffer_view( gl::ELEMENT_ARRAY_BUFFER, &u8_array, gl::STATIC_DRAW );
              index_count = Some( ( ( bytes.len() as u32 ) / idx_stride, idx_gl_type ) );
              index_buffer = Some( buffer );
            }

            gl.bind_vertex_array( None );

            let vertex_count = positions.as_ref().map_or( 0, | b | ( b.len() / 8 ) as u32 );

            resources.borrow_mut().store_geometry( id, GpuGeometry
            {
              gl : gl.clone(), vao, position_buffer, uv_buffer, index_buffer, vertex_count, index_count,
            });

            // Re-setup any mesh batch VAOs that reference this geometry.
            // Batches created before async load completed only have instance attribs;
            // now that geometry buffers are available, add geometry attribs too.
            {
              let res = resources.borrow();
              if let Some( geom ) = res.geometry( id )
              {
                for batch in res.batches.values()
                {
                  if let GpuBatch::Mesh { vao, params, instances, .. } = batch
                    && params.geometry == id
                  {
                    setup_mesh_batch_vao
                    (
                      gl,
                      vao,
                      geom.position_buffer.as_ref(),
                      geom.uv_buffer.as_ref(),
                      geom.index_buffer.as_ref(),
                      instances.buffer(),
                    );
                  }
                }
              }
            }
          });
        }
        else
        {
          // Synchronous path — all data is already in memory.
          let vao = gl::vao::create( gl ).map_err( map_err )?;
          gl.bind_vertex_array( Some( &vao ) );

          let mut position_buffer = None;
          if let crate::assets::Source::Bytes( ref bytes ) = geom.positions
          {
            let buffer = gl::buffer::create( gl ).map_err( map_err )?;
            gl::buffer::upload( gl, &buffer, bytes, gl::STATIC_DRAW );
            gl.enable_vertex_attrib_array( 0 );
            gl.vertex_attrib_pointer_with_i32( 0, 2, gl::FLOAT, false, 0, 0 );
            position_buffer = Some( buffer );
          }

          let mut uv_buffer = None;
          if let Some( crate::assets::Source::Bytes( ref bytes ) ) = geom.uvs
          {
            let buffer = gl::buffer::create( gl ).map_err( map_err )?;
            gl::buffer::upload( gl, &buffer, bytes, gl::STATIC_DRAW );
            gl.enable_vertex_attrib_array( 1 );
            gl.vertex_attrib_pointer_with_i32( 1, 2, gl::FLOAT, false, 0, 0 );
            uv_buffer = Some( buffer );
          }

          let mut index_buffer = None;
          let mut index_count = None;
          if let Some( crate::assets::Source::Bytes( ref bytes ) ) = geom.indices
          {
            let buffer = gl::buffer::create( gl ).map_err( map_err )?;
            gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, Some( &buffer ) );
            let u8_array = js_sys::Uint8Array::from( bytes.as_slice() );
            gl.buffer_data_with_array_buffer_view( gl::ELEMENT_ARRAY_BUFFER, &u8_array, gl::STATIC_DRAW );
            index_count = Some( ( ( bytes.len() as u32 ) / idx_stride, idx_gl_type ) );
            index_buffer = Some( buffer );
          }

          gl.bind_vertex_array( None );

          let vertex_count = if let crate::assets::Source::Bytes( ref bytes ) = geom.positions
          { ( bytes.len() / 8 ) as u32 } else { 0 };

          self.resources.borrow_mut().store_geometry( geom.id, GpuGeometry
          {
            gl : gl.clone(), vao, position_buffer, uv_buffer, index_buffer, vertex_count, index_count,
          });
        }
      }

      Ok( () )
    }
  }

  // ============================================================================
  // Backend trait impl
  // ============================================================================

  impl Backend for WebGlBackend
  {
    fn load_assets( &mut self, assets : &Assets ) -> Result< (), RenderError >
    {
      // Reset all GPU state: textures, sprites, geometries, and batches.
      // GpuBatch::drop calls delete_vertex_array; ArrayBuffer::drop calls delete_buffer.
      // Safe to call multiple times (e.g. level transitions).
      //
      // Bump the generation counter so any in-flight `spawn_local` futures from
      // a previous cycle notice they are stale and bail out before overwriting
      // entries belonging to this new cycle.
      //
      // ORDER MATTERS: batches must be cleared BEFORE geometries / textures (which
      // are cleared inside `load_images` / `load_geometries` below). A mesh batch's
      // VAO holds attrib pointers into the geometry's position / uv / index buffers;
      // if the geometry was dropped first, those buffers would be deleted while
      // still referenced by live batch VAOs. Dropping batches first ensures each
      // batch VAO is gone before any buffer it referenced is freed.
      {
        let mut res = self.resources.borrow_mut();
        res.generation = res.generation.wrapping_add( 1 );
        res.batches.clear();
      }
      // Clear the stale recording batch ID: the referenced batch no longer exists,
      // so leaving it set would make cmd_bind_batch reject any new bind on the next frame.
      self.recording_batch = None;
      self.load_images( &assets.images )?;
      self.load_sprites( &assets.sprites );
      self.load_geometries( &assets.geometries )?;
      // qqq: gradients, patterns, clip masks, fonts
      Ok( () )
    }

    fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
    {
      let viewport = self.viewport_size();

      for cmd in commands
      {
        // Unimplemented placeholder arms (Path/Text/Group) all map to {} and are
        // intentionally kept separate for readability and future expansion.
        #[ allow( clippy::match_same_arms ) ]
        match cmd
        {
          RenderCommand::Clear( c ) => self.cmd_clear( c ),

          // Mesh & sprite
          RenderCommand::Mesh( m ) => self.cmd_mesh( m, &viewport ),
          RenderCommand::Sprite( s ) => self.cmd_sprite( s, &viewport ),

          // Batch lifecycle
          RenderCommand::CreateSpriteBatch( c ) => self.cmd_create_sprite_batch( c )?,
          RenderCommand::CreateMeshBatch( c ) => self.cmd_create_mesh_batch( c )?,
          RenderCommand::BindBatch( b ) => self.cmd_bind_batch( b )?,
          RenderCommand::AddSpriteInstance( si ) => self.cmd_add_sprite_instance( si )?,
          RenderCommand::AddMeshInstance( mi ) => self.cmd_add_mesh_instance( mi )?,
          RenderCommand::SetSpriteInstance( si ) => self.cmd_set_sprite_instance( si )?,
          RenderCommand::SetMeshInstance( mi ) => self.cmd_set_mesh_instance( mi )?,
          RenderCommand::RemoveInstance( ri ) => self.cmd_remove_instance( ri )?,
          RenderCommand::SetSpriteBatchParams( sp ) => self.cmd_set_sprite_batch_params( sp )?,
          RenderCommand::SetMeshBatchParams( mp ) => self.cmd_set_mesh_batch_params( mp )?,
          RenderCommand::UnbindBatch( _ ) => self.cmd_unbind_batch(),
          RenderCommand::DrawBatch( db ) => self.cmd_draw_batch( db, &viewport )?,
          RenderCommand::DeleteBatch( db ) => self.cmd_delete_batch( db ),

          // Path — skip (qqq). Warn on the opener only (not MoveTo/LineTo/etc.)
          // so a 1000-segment path produces one message, not 1000. `capabilities()`
          // already advertises `paths: false`; this is a DX nudge for callers who
          // submitted anyway.
          RenderCommand::BeginPath( _ ) => web_sys::console::warn_1
          (
            &"WebGlBackend: path commands are not implemented; BeginPath..EndPath will be ignored (see capabilities().paths)".into()
          ),
          RenderCommand::MoveTo( _ )
          | RenderCommand::LineTo( _ )
          | RenderCommand::QuadTo( _ )
          | RenderCommand::CubicTo( _ )
          | RenderCommand::ArcTo( _ )
          | RenderCommand::ClosePath( _ )
          | RenderCommand::EndPath( _ ) => {}

          // Text — skip (qqq). See note above re: opener-only warning.
          RenderCommand::BeginText( _ ) => web_sys::console::warn_1
          (
            &"WebGlBackend: text commands are not implemented; BeginText..EndText will be ignored (see capabilities().text)".into()
          ),
          RenderCommand::Char( _ )
          | RenderCommand::EndText( _ ) => {}

          // Grouping — skip (qqq).
          RenderCommand::BeginGroup( _ ) => web_sys::console::warn_1
          (
            &"WebGlBackend: group commands are not implemented; BeginGroup..EndGroup will be ignored (see capabilities().effects)".into()
          ),
          RenderCommand::EndGroup( _ ) => {}
        }
      }

      Ok( () )
    }

    fn output( &self ) -> Result< Output, RenderError >
    {
      Ok( Output::Presented )
    }

    fn resize( &mut self, width : u32, height : u32 )
    {
      self.config.width = width;
      self.config.height = height;
      self.gl.viewport( 0, 0, width as i32, height as i32 );
    }

    fn capabilities( &self ) -> Capabilities
    {
      Capabilities
      {
        paths : false,       // qqq: tessellation / GPU curves
        text : false,        // qqq: glyph atlas / SDF fonts
        meshes : true,
        sprites : true,
        batches : true,
        gradients : false,   // qqq: not yet loaded or rendered
        patterns : false,    // qqq: not yet loaded or rendered
        clip_masks : false,  // qqq: not yet loaded or rendered
        effects : false,     // qqq: requires FBO post-processing
        // `blend_modes` means "all variants correct"; Overlay silently falls back
        // to Normal in `apply_blend` (needs FBO / custom shader), so this is false.
        // Callers needing per-mode info should check `supported_blend_modes`.
        blend_modes : false,
        supported_blend_modes : &[ BlendMode::Normal, BlendMode::Add, BlendMode::Multiply, BlendMode::Screen ],
        text_on_path : false,
        max_texture_size : self.max_texture_size,
      }
    }
  }

  // ============================================================================
  // Shared utilities
  // ============================================================================

  /// Like `gl::texture::d2::upload_image_from_path`, but updates
  /// `GpuTexture.width` / `height` cells once the image loads.
  fn upload_image_from_path
  (
    gl : &gl::GL,
    src : &str,
    id : ResourceId< asset::Image >,
    resources : &Rc< RefCell< GpuResources > >,
    filter : SamplerFilter,
    mipmap : MipmapMode,
    generation : u32,
  ) -> web_sys::WebGlTexture
  {
    let document = web_sys::window().expect( "no window" ).document().expect( "no document" );

    let texture = gl.create_texture().expect( "failed to create texture" );

    let img : HtmlImageElement = document.create_element( "img" )
      .expect( "can't create img" )
      .dyn_into()
      .expect( "not an HtmlImageElement" );
    img.style().set_property( "display", "none" ).expect( "can't hide img" );

    // The browser fires `load` XOR `error` exactly once for a given `set_src`
    // call — there is no retry path here — so FnOnce handlers are the right
    // shape. `Closure::once_into_js` takes ownership of the Rust closure,
    // returns a JsValue that we hand to the img element, and arranges for the
    // captured state (notably the `Rc<RefCell<GpuResources>>` clone in
    // `on_load`) to be freed after the single invocation, or via finalizer if
    // the event never fires and the JS function is GC'd. This is what lets a
    // `WebGlBackend` drop actually release its GPU resources.
    let on_load = Closure::once_into_js(
    {
      let gl = gl.clone();
      let img = img.clone();
      let texture = texture.clone();
      let resources = Rc::clone( resources );
      move ||
      {
        // Bail out if `load_assets` ran again before the image finished loading —
        // this closure belongs to a previous cycle and must not touch the fresh
        // texture that now occupies this id.
        if resources.borrow().generation != generation
        {
          img.remove();
          return;
        }

        gl::texture::d2::upload( &gl, Some( &texture ), &img );

        // Bind and apply all sampler state now that level 0 is populated. Binding
        // explicitly because upload() may leave a different texture bound, and
        // tex_parameteri / generate_mipmap act on whatever is bound to TEXTURE_2D.
        // Applying filter here (not only at texture creation) ensures the correct
        // mag/min filters are installed on the texture object regardless of any
        // intervening bind changes — belt-and-suspenders for the async path.
        gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
        apply_texture_filter( &gl, &filter, &mipmap );
        gl::texture::d2::wrap_clamp( &gl );
        if !matches!( mipmap, MipmapMode::Off )
        {
          gl.generate_mipmap( gl::TEXTURE_2D );
        }

        if let Some( gpu_tex ) = resources.borrow().texture( id )
        {
          gpu_tex.width.set( img.natural_width() );
          gpu_tex.height.set( img.natural_height() );
        }

        img.remove();
      }
    });

    let src_for_err = src.to_owned();
    let on_error = Closure::once_into_js(
    {
      let img = img.clone();
      move ||
      {
        web_sys::console::error_1
        (
          &format!( "tilemap_renderer: failed to load image from path {src_for_err:?}" ).into()
        );
        // Remove the element so the other (never-fired) handler becomes unreachable
        // and can be GC'd, rather than sitting on a detached img for the lifetime
        // of the document.
        img.remove();
      }
    });

    img.set_onload( Some( on_load.unchecked_ref() ) );
    img.set_onerror( Some( on_error.unchecked_ref() ) );
    img.set_src( src );
    // `on_load` / `on_error` are `JsValue`s produced by `Closure::once_into_js`.
    // The img element now holds JS-side references to both functions via its
    // `onload` / `onerror` properties, so dropping the local JsValue bindings
    // here does not free the functions. When either event fires, its Rust
    // closure is dropped (releasing its captures, including the cloned Rc for
    // `on_load`); the other handler — plus the img itself — becomes GC-eligible
    // once the fired handler calls `img.remove()` above.

    texture
  }
}

mod_interface::mod_interface!
{
  layer webgl_helpers;

  own use WebGlBackend;
}
