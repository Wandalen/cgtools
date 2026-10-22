//! WebGL backend adapter.
//!
//! Hardware-accelerated 2D rendering via WebGL2 (wasm32 target).
//! Uses `minwebgl` for GL calls. Quad vertices are generated in
//! the vertex shader via `gl_VertexID` — no quad VAO needed.

mod private
{
  use std::rc::Rc;
  use core::cell::{ Cell, RefCell };
  use wasm_bindgen::prelude::*;
  use minwebgl as gl;
  pub use super::render_target::RenderTargetId;
  use super::render_target::TargetSlots;
  use super::webgl_renderers::{ SpriteRenderer, MeshRenderer };
  use super::webgl_textures::{ bitmap_texture_upload, image_upload_from_path };
  use super::webgl_helpers::
  {
    ArrayBuffer,
    SpriteInstanceData,
    MeshInstanceData,
    GpuResources,
    GpuTexture,
    GpuSprite,
    GpuFramebuffer,
    GpuGeometry,
    GpuBatch,
    sprite_batch_vao_setup,
    mesh_batch_vao_setup,
    blend_apply,
    source_to_loadable,
    loadable_resolve,
    index_format,
    texture_filter_apply,
    texture_wrap_apply,
    topology_to_gl,
    mat3_mul,
  };
  use crate::assets::Assets;
  use crate::backend::{ RenderError, Backend, Output, Capabilities };
  use crate::commands::{ Clear, Mesh, Sprite, CreateSpriteBatch, CreateMeshBatch, BindBatch, AddSpriteInstance, AddMeshInstance, SetSpriteInstance, SetMeshInstance, RemoveInstance, SetSpriteBatchParams, SetMeshBatchParams, DrawBatch, DeleteBatch, RenderCommand };
  use crate::types::{ FillRef, RenderConfig, ResourceId, Batch, MipmapMode, BlendMode, asset };

  // ============================================================================
  // Camera / matrix helpers
  // ============================================================================

  /// Column-major identity 3×3 — the default view matrix (world pixels map
  /// straight to viewport pixels).
  const IDENTITY_MAT3 : [ f32; 9 ] = [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ];

  // ============================================================================
  // Backend struct
  // ============================================================================

  /// WebGL renderer backend.
  ///
  /// ```ignore
  /// let config = RenderConfig { width: 800, height: 600, ..Default::default() };
  /// let gl_ctx = minwebgl::context::from_canvas( &canvas )?;
  /// let mut backend = WebGlBackend::new( config, gl_ctx )?;
  /// backend.assets_load( &assets )?;
  /// backend.submit( &commands )?;
  /// ```
  pub struct WebGlBackend
  {
    pub( super ) config : RenderConfig,
    pub( super ) gl : gl::GL,
    pub( super ) resources : Rc< RefCell< GpuResources > >,
    pub( super ) sprite : SpriteRenderer,
    pub( super ) mesh : MeshRenderer,
    pub( super ) max_texture_size : u32,

    /// World→screen view matrix (column-major 3×3), pre-multiplied into every
    /// world-space draw's parent transform. Set once per frame via
    /// [`Self::camera_set`]; identity until the caller supplies one. Keeping the
    /// camera here — rather than baked into per-instance transforms — is what
    /// makes a pan/zoom a single-uniform update instead of a full instance-buffer
    /// re-upload.
    pub( super ) camera : [ f32; 9 ],

    // -- batch editing state --
    pub( super ) recording_batch : Option< ResourceId< Batch > >,

    /// Offscreen cache surfaces (see [`Self::render_target_create`]), addressed
    /// by [`RenderTargetId`]. Used by consumers that bake a subset of the frame
    /// into a texture and composite it back as a world quad via
    /// [`Self::render_target_draw`].
    pub( super ) render_targets : TargetSlots< GpuFramebuffer >,

    /// Viewport override in force while a render-target pass is bound. The
    /// vertex shaders divide world coords by `u_viewport` to reach NDC, so during
    /// a bake this MUST be the target's pixel size, not the canvas size — else the
    /// baked geometry projects to the wrong clip range and lands off-texture.
    /// `None` outside a bake (shaders use the canvas size).
    pub( super ) active_viewport : Option< [ f32; 2 ] >,

    // -- context state --
    // Set by the `webglcontextlost` listener, cleared by `webglcontextrestored`.
    // `submit`/`output` check this before issuing any GL call. `Rc` so the listener
    // closures (which outlive `new`) can share it with the backend instance.
    pub( super ) context_lost : Rc< Cell< bool > >,
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
      let ( sprite, mesh ) = Self::renderers_new( &gl )?;
      Self::gl_state_init( &gl, &config );

      // Query the actual hardware limit; fall back to the WebGL2 guaranteed minimum.
      // get_parameter returns a JsValue; as_f64() is the idiomatic way to extract it.
      // u32::try_from(i64) rejects negatives; the i64 intermediate avoids cast_sign_loss.
      let max_texture_size : u32 = gl
        .get_parameter( gl::MAX_TEXTURE_SIZE )
        .ok()
        .and_then( | v | v.as_f64() )
        .and_then( | v | u32::try_from( v as i64 ).ok() )
        .unwrap_or( 2048 );

      let context_lost = Rc::new( Cell::new( false ) );
      Self::context_loss_listeners_register( &gl, &context_lost );

      Ok( Self
      {
        config,
        gl,
        resources : Rc::new( RefCell::new( GpuResources::new() ) ),
        sprite,
        mesh,
        max_texture_size,
        camera : IDENTITY_MAT3,
        recording_batch : None,
        render_targets : TargetSlots::new(),
        active_viewport : None,
        context_lost,
      })
    }

    /// Set the world→screen view matrix (column-major 3×3) used to project every
    /// world-space draw this frame. Call once per frame before [`Backend::submit`]
    /// (a pan/zoom only changes this matrix — the command stream and instance
    /// buffers stay identical, so panning costs one uniform per batch, not a
    /// scene re-upload). `ScreenSpaceSprite`s are unaffected. Identity leaves
    /// world coordinates mapped straight to viewport pixels.
    ///
    /// The camera is WebGL only: this is an inherent method, not part of
    /// [`Backend`], and no [`crate::commands::RenderCommand`] carries a camera,
    /// so the other adapters draw world coordinates as viewport pixels.
    #[ inline ]
    pub fn camera_set( &mut self, camera : [ f32; 9 ] )
    {
      self.camera = camera;
    }

    /// Compiles the sprite and mesh shader programs.
    fn renderers_new( gl : &gl::GL ) -> Result< ( SpriteRenderer, MeshRenderer ), RenderError >
    {
      let map_err = | e : gl::WebglError | RenderError::BackendError( format!( "{e:?}" ) );
      Ok( ( SpriteRenderer::new( gl ).map_err( map_err )?, MeshRenderer::new( gl ).map_err( map_err )? ) )
    }

    /// Sets the GL state every draw relies on but none re-applies: viewport,
    /// blending and depth testing. Run by `new` and again by `assets_load` after
    /// a context restore, which resets all of it to the defaults.
    fn gl_state_init( gl : &gl::GL, config : &RenderConfig )
    {
      gl.viewport( 0, 0, config.width as i32, config.height as i32 );
      gl.enable( gl::BLEND );
      // Use separate factors for the alpha channel so the framebuffer alpha follows
      // the Porter-Duff "over" rule: a = src_a + dst_a * (1 - src_a). Using the same
      // SRC_ALPHA factor on alpha would yield src_a^2 + dst_a*(1-src_a), corrupting
      // alpha when the canvas is composited against a transparent page or read via
      // readPixels.
      //
      // This is just the initial state; `blend_apply` reprograms the blend func
      // per draw from each sprite/mesh's `BlendMode` and its texture's
      // premultiplied flag (see `blend_apply`).
      gl.blend_func_separate( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ONE_MINUS_SRC_ALPHA );

      // LEQUAL (not LESS) so equal-depth draws fall back to submission order rather
      // than rejecting the second one — keeps the default (all depth = 0) case
      // rendering identically to the pre-depth implementation.
      gl.enable( gl::DEPTH_TEST );
      gl.depth_func( gl::LEQUAL );
    }

    /// Registers persistent `webglcontextlost` / `webglcontextrestored` listeners on the
    /// canvas reachable from `gl`. Context can be lost and restored more than once in a
    /// session (unlike the one-shot `Closure::once_into_js` image-load idiom elsewhere in
    /// this file), so both closures are leaked via `.forget()` rather than consumed on first
    /// invocation. `webglcontextlost` calls `prevent_default()` — the WebGL spec's own opt-in
    /// signal for the browser to attempt restoration at all; omitting it means the browser
    /// never tries. If `gl`'s canvas cannot be resolved (e.g. a non-`HtmlCanvasElement`
    /// rendering target), listener registration is skipped and a diagnostic is logged —
    /// context loss then remains silently unrecoverable, same as before this method existed.
    fn context_loss_listeners_register( gl : &gl::GL, context_lost : &Rc< Cell< bool > > )
    {
      let Some( canvas ) = gl.canvas().and_then( | c | c.dyn_into::< web_sys::HtmlCanvasElement >().ok() ) else
      {
        web_sys::console::warn_1
        (
          &"WebGlBackend: could not resolve an HtmlCanvasElement from the GL context; context-loss detection is disabled".into()
        );
        return;
      };

      let lost_flag = Rc::clone( context_lost );
      let on_lost = Closure::< dyn FnMut( web_sys::Event ) >::new( move | event : web_sys::Event |
      {
        event.prevent_default();
        lost_flag.set( true );
      });
      let _ = canvas.add_event_listener_with_callback( "webglcontextlost", on_lost.as_ref().unchecked_ref() );
      on_lost.forget();

      // Fix(BUG-441): `webglcontextrestored` no longer clears `context_lost` itself — it only
      // logs. Clearing happens in `assets_load` instead, once GPU state has actually been
      // re-uploaded. See that method for the full rationale.
      let on_restored = Closure::< dyn FnMut( web_sys::Event ) >::new( move | _event : web_sys::Event |
      {
        web_sys::console::warn_1
        (
          &"WebGlBackend: WebGL context restored; submit()/output() remain blocked until assets_load() is called again to re-upload GPU state".into()
        );
      });
      let _ = canvas.add_event_listener_with_callback( "webglcontextrestored", on_restored.as_ref().unchecked_ref() );
      on_restored.forget();
    }

    pub( super ) fn viewport_size( &self ) -> [ f32; 2 ]
    {
      self.active_viewport.unwrap_or( [ self.config.width as f32, self.config.height as f32 ] )
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
      // A depth clear is a no-op while `depthMask` is false. A frame whose
      // `submit` bailed out between `SetDepthWrite( false )` and the restore
      // would leave writes off, and this clear would then keep last frame's
      // depth. Forcing writes on also makes `true` the state every frame
      // starts from, which the command stream assumes.
      self.gl.depth_mask( true );
      self.gl.clear_color( r, g, b, a );
      self.gl.clear_depth( 1.0 );
      self.gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );
    }

    // Fix(BUG-209): `res.geometry`/`res.sprite` lookups here used to fail
    // silently ( `if let Some(..) = .. { .. }` with no `else`, or `else {
    // return }` ) instead of surfacing `RenderError::MissingAsset` the way
    // `native.rs`/`webgpu.rs`'s sibling sprite-draw functions already do --
    // a caller referencing a never-loaded sprite/mesh got a silently
    // dropped draw instead of a diagnosable error. Root cause: these two
    // functions predate `submit`'s command loop propagating `?`, and were
    // never updated to match once every other fallible command handler in
    // this file adopted `Result`.
    fn cmd_mesh( &self, m : &Mesh, viewport : [ f32; 2 ] ) -> Result< (), RenderError >
    {
      let res = self.resources.borrow();
      let Some( geom ) = res.geometry( m.geometry ) else
      {
        return Err( RenderError::MissingAsset( m.geometry.inner() ) );
      };

      let mat = mat3_mul( &self.camera, &m.transform.to_mat3() );
      let color = match m.fill { FillRef::Solid( c ) => c, _ => [ 1.0, 1.0, 1.0, 1.0 ] };
      // Untextured meshes are straight-alpha; a textured mesh inherits its
      // texture's premultiplied flag.
      let premultiplied = res.mesh_premultiplied( m.texture );
      blend_apply( &self.gl, &m.blend, premultiplied );

      let mut use_texture = false;
      if let Some( tex_id ) = m.texture && let Some( gpu_tex ) = res.texture( tex_id )
      {
        // Fix(BUG-537): skipped like `cmd_sprite` skips a pending sheet: the
        // image's async decode hasn't landed ( or failed ), and a texture with
        // no level-0 image samples as opaque black.
        // Root cause: `Path` / `Encoded` images are registered 0×0 until
        // `on_load` uploads them, and only the sprite paths checked the size.
        // Pitfall: every draw path that binds an image texture has to check it.
        if gpu_tex.width.get() == 0 || gpu_tex.height.get() == 0 { return Ok( () ); }
        self.gl.active_texture( gl::TEXTURE0 );
        self.gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );
        use_texture = true;
      }

      self.mesh.draw( &self.gl, geom, &mat, &color, topology_to_gl( &m.topology ), viewport, use_texture, premultiplied, m.transform.depth, self.config.max_depth );
      Ok( () )
    }

    /// Draw one sprite. `apply_camera` selects the coordinate space of
    /// `s.transform`: `true` for a world-space `Sprite` (the view matrix is
    /// pre-multiplied so the GPU projects it), `false` for a `ScreenSpaceSprite`
    /// whose transform is already in viewport pixels and must bypass the camera.
    fn cmd_sprite( &self, s : &Sprite, apply_camera : bool, viewport : [ f32; 2 ] ) -> Result< (), RenderError >
    {
      let res = self.resources.borrow();
      let Some( gpu_sprite ) = res.sprite( s.sprite ) else
      {
        return Err( RenderError::MissingAsset( s.sprite.inner() ) );
      };
      let Some( gpu_tex ) = res.texture( gpu_sprite.sheet ) else
      {
        return Err( RenderError::MissingAsset( gpu_sprite.sheet.inner() ) );
      };

      let tw = gpu_tex.width.get();
      let th = gpu_tex.height.get();
      // Not a caller bug -- the sheet is loaded but its async decode ( the
      // `ImageSource::Path` -> `HtmlImageElement` path ) hasn't landed yet.
      if tw == 0 || th == 0 { return Ok( () ); }

      self.gl.active_texture( gl::TEXTURE0 );
      self.gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );

      let tex_size = [ tw as f32, th as f32 ];

      let mat = if apply_camera { mat3_mul( &self.camera, &s.transform.to_mat3() ) } else { s.transform.to_mat3() };
      blend_apply( &self.gl, &s.blend, gpu_tex.premultiplied );
      self.sprite.draw( &self.gl, &mat, &gpu_sprite.region, tex_size, &s.tint, gpu_tex.premultiplied, viewport, s.transform.depth, self.config.max_depth );
      Ok( () )
    }

    fn cmd_create_sprite_batch( &mut self, cmd : &CreateSpriteBatch ) -> Result< (), RenderError >
    {
      let map_err = | e : gl::WebglError | RenderError::BackendError( format!( "{e:?}" ) );
      let gl = &self.gl;
      let instances = ArrayBuffer::< SpriteInstanceData >::new( gl, 16 ).map_err( map_err )?;
      let vao = gl::vao::create( gl ).map_err( map_err )?;
      sprite_batch_vao_setup( gl, &vao, instances.buffer() );
      self.resources.borrow_mut().batch_store( cmd.batch, GpuBatch::Sprite
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
        mesh_batch_vao_setup
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
      self.resources.borrow_mut().batch_store( cmd.batch, GpuBatch::Mesh
      {
        gl : self.gl.clone(),
        instances,
        vao,
        params : cmd.params,
      });
      Ok( () )
    }

    fn cmd_bind_batch( &mut self, cmd : BindBatch ) -> Result< (), RenderError >
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
      let data = MeshInstanceData { transform : mi.transform.to_mat3(), depth : mi.transform.depth, tint : mi.tint };
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
      let data = MeshInstanceData { transform : mi.transform.to_mat3(), depth : mi.transform.depth, tint : mi.tint };
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

    fn cmd_remove_instance( &mut self, ri : RemoveInstance ) -> Result< (), RenderError >
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

    fn cmd_unbind_batch( &mut self ) -> Result< (), RenderError >
    {
      if let Some( batch_id ) = self.recording_batch.take()
      {
        // Unbind is the commit point: upload everything this recording staged in
        // one write per batch, before any `DrawBatch` can read it.
        {
          let mut res = self.resources.borrow_mut();
          let flushed = match res.batch_mut( batch_id )
          {
            Some( GpuBatch::Sprite { instances, .. } ) => instances.flush(),
            Some( GpuBatch::Mesh { instances, .. } ) => instances.flush(),
            None => Ok( () ),
          };
          flushed.map_err( | e | RenderError::BackendError( format!( "UnbindBatch({batch_id:?}): {e}" ) ) )?;
        }
        let res = self.resources.borrow();
        if let Some( batch ) = res.batch( batch_id )
        {
          match batch
          {
            GpuBatch::Sprite { instances, vao, .. } =>
            {
              sprite_batch_vao_setup( &self.gl, vao, instances.buffer() );
            }
            GpuBatch::Mesh { instances, vao, params, .. } =>
            {
              if let Some( geom ) = res.geometry( params.geometry )
              {
                mesh_batch_vao_setup
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
      Ok( () )
    }

    fn cmd_draw_batch( &self, db : DrawBatch, viewport : [ f32; 2 ] ) -> Result< (), RenderError >
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
      // A sprite batch inherits the premultiplied flag of its single sheet
      // texture; a mesh batch may carry a premultiplied texture too
      // (`MeshBatchParams::texture`), so hardcoding straight-alpha there would
      // double-scale its edges by alpha — route both through the same
      // `mesh_premultiplied` helper `cmd_mesh` uses so the two paths can't drift.
      // Resolved once here and passed to both `blend_apply` and `batch_draw`'s
      // `u_premultiplied` upload: the `ONE` source factor is only right while the
      // shader keeps its output premultiplied, so the two must never disagree.
      // Only sprite batches carry `occlude_overlap` (single-coverage mode).
      let ( blend, premultiplied, occlude ) = match gpu_batch
      {
        GpuBatch::Sprite { params, .. } =>
          ( &params.blend, res.texture( params.sheet ).is_some_and( | t | t.premultiplied ), params.occlude_overlap ),
        GpuBatch::Mesh { params, .. } =>
          ( &params.blend, res.mesh_premultiplied( params.texture ), false ),
      };
      blend_apply( &self.gl, blend, premultiplied );
      // Single-coverage mode: paint each pixel once regardless of how many
      // overlapping (bled) instances cover it. LESS rejects every fragment at
      // a depth already stored, so once the first instance writes the batch's
      // depth at a pixel the overlapping ones fail (the discard in the frag
      // shader keeps the transparent quad area from writing depth). There is
      // deliberately no depth clear here: it would also wipe the depth of
      // nearer opaque draws that must keep hiding this batch. The caller
      // instead places the batch strictly nearer than whatever it draws over
      // (the scene renderer gives every bucket its own depth slot) and keeps
      // depth writes on. Restored to LEQUAL after so following draws
      // composite normally.
      if occlude
      {
        self.gl.depth_func( gl::LESS );
      }
      match gpu_batch
      {
        GpuBatch::Sprite { .. } => self.sprite.batch_draw( &self.gl, gpu_batch, &res, &self.camera, premultiplied, viewport, self.config.max_depth ),
        GpuBatch::Mesh { .. } => self.mesh.batch_draw( &self.gl, gpu_batch, &res, &self.camera, premultiplied, viewport, self.config.max_depth ),
      }
      if occlude
      {
        self.gl.depth_func( gl::LEQUAL );
      }
      Ok( () )
    }

    fn cmd_delete_batch( &mut self, db : DeleteBatch )
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

    fn images_load( &mut self, images : &[ crate::assets::ImageAsset ] ) -> Result< (), RenderError >
    {
      let gl = &self.gl;
      self.resources.borrow_mut().textures.clear();

      for img in images
      {
        let ( texture, w, h ) = match &img.source
        {
          crate::assets::ImageSource::Bitmap { bytes, width, height, format } =>
          {
            let tex = bitmap_texture_upload( gl, bytes, *width, *height, *format, img.id )?;
            ( tex, *width, *height )
          }
          crate::assets::ImageSource::Encoded( bytes ) =>
          {
            let mime = crate::assets::image_mime_detect( bytes );
            let parts = gl::js_sys::Array::new();
            parts.push( &gl::js_sys::Uint8Array::from( bytes.as_slice() ) );
            let url = match gl::blob::blob_create( parts, mime )
            {
              Ok( url ) => url,
              Err( err ) =>
              {
                web_sys::console::error_1
                (
                  &format!( "WebGlBackend: failed to create Blob URL for image {:?}: {err:?}", img.id ).into()
                );
                continue;
              }
            };
            // Async path, same as `ImageSource::Path` below: sampler state is
            // applied inside the on_load callback once the image is actually
            // uploaded. `image_upload_from_path` revokes this `blob:` URL
            // (guarded by prefix) once the browser has decoded it — unlike a
            // real path, nothing else keeps the URL alive.
            let generation = self.resources.borrow().generation;
            let tex = image_upload_from_path( gl, &url, img, &self.resources, generation );
            gl.bind_texture( gl::TEXTURE_2D, Some( &tex ) );
            ( tex, 0, 0 )
          }
          crate::assets::ImageSource::Path( path ) =>
          {
            let path = path.as_path().to_str()
              .ok_or_else( || RenderError::BackendError( "non-UTF-8 image path".into() ) )?;
            // Async path: sampler state (filter, wrap, mipmap chain) is applied inside
            // the on_load callback after the image bytes are actually uploaded, so the
            // texture is guaranteed to be complete (esp. for mipmap modes, which leave
            // the texture incomplete until generate_mipmap runs).
            let generation = self.resources.borrow().generation;
            let tex = image_upload_from_path( gl, path, img, &self.resources, generation );
            gl.bind_texture( gl::TEXTURE_2D, Some( &tex ) );
            ( tex, 0, 0 )
          }
        };

        // Sync bitmap path: level 0 is already uploaded; apply sampler state and
        // generate the mip chain right away so the texture is immediately usable.
        // (The async Path branch does all of this inside on_load.)
        if matches!( img.source, crate::assets::ImageSource::Bitmap { .. } )
        {
          texture_filter_apply( gl, &img.filter, &img.mipmap );
          texture_wrap_apply( gl, img.wrap );
          if !matches!( img.mipmap, MipmapMode::Off )
          {
            gl.generate_mipmap( gl::TEXTURE_2D );
          }
        }

        self.resources.borrow_mut().texture_store( img.id, GpuTexture
        {
          gl : gl.clone(),
          texture,
          width : Cell::new( w ),
          height : Cell::new( h ),
          filter : img.filter,
          mipmap : img.mipmap,
          wrap : img.wrap,
          premultiplied : img.premultiplied,
        });
      }

      Ok( () )
    }

    // Returns () unlike images_load/geometries_load because sprite loading is
    // infallible — it only stores sub-regions of already-loaded textures (no GPU
    // upload, no allocation that can fail).
    fn sprites_load( &mut self, sprites : &[ crate::assets::SpriteAsset ] )
    {
      self.resources.borrow_mut().sprites.clear();

      for spr in sprites
      {
        self.resources.borrow_mut().sprite_store( spr.id, GpuSprite
        {
          sheet : spr.sheet,
          region : spr.region,
        });
      }
    }

    fn geometries_load( &mut self, geometries : &[ crate::assets::GeometryAsset ] ) -> Result< (), RenderError >
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
          // The placeholder owns its own VAO (never shared): when `geometry_store`
          // later replaces it, its `Drop` deletes *this* VAO, not the populated one.
          // The spawn_local future creates a separate VAO for the populated entry.
          let placeholder_vao = gl::vao::create( gl ).map_err( map_err )?;
          self.resources.borrow_mut().geometry_store( geom.id, GpuGeometry
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

            let positions = loadable_resolve( positions_source ).await;
            let uvs = match uvs_source { Some( s ) => Some( loadable_resolve( s ).await ), None => None };
            let indices = match indices_source { Some( s ) => Some( loadable_resolve( s ).await ), None => None };

            // Bail out if `assets_load` ran again while we were fetching — this future
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

            resources.borrow_mut().geometry_store( id, GpuGeometry
            {
              gl : gl.clone(), vao, position_buffer, uv_buffer, index_buffer, vertex_count, index_count,
            });

            mesh_batch_vaos_refresh( gl, &resources, id );
          });
        }
        else
        {
          self.geometry_sync_load( geom, idx_stride, idx_gl_type )?;
        }
      }

      Ok( () )
    }

    // Synchronous geometry load — all data already in memory (no `Source::Path` fields).
    // Split out of `geometries_load` to keep that function's async/sync dispatch readable.
    fn geometry_sync_load
    (
      &self,
      geom : &crate::assets::GeometryAsset,
      idx_stride : u32,
      idx_gl_type : u32,
    ) -> Result< (), RenderError >
    {
      let gl = &self.gl;
      let map_err = | e : gl::WebglError | RenderError::BackendError( format!( "{e:?}" ) );

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

      self.resources.borrow_mut().geometry_store( geom.id, GpuGeometry
      {
        gl : gl.clone(), vao, position_buffer, uv_buffer, index_buffer, vertex_count, index_count,
      });

      Ok( () )
    }

    /// Computes this backend's declared `Capabilities` from its one
    /// hardware-dependent input -- `max_texture_size` -- touching no
    /// `WebGl2RenderingContext`/`web_sys` state, so it is testable without a
    /// live GL context.
    #[ must_use ]
    pub fn declared_capabilities( max_texture_size : u32 ) -> Capabilities
    {
      Capabilities
      {
        paths : false,       // needs tessellation / GPU curves
        text : false,        // needs a glyph atlas / SDF fonts
        meshes : true,
        sprites : true,
        batches : true,
        gradients : false,   // not yet loaded or rendered
        patterns : false,    // not yet loaded or rendered
        clip_masks : false,  // not yet loaded or rendered
        effects : false,     // needs FBO post-processing
        // `blend_modes` means "all variants correct"; Overlay silently falls back
        // to Normal in `blend_apply` (needs FBO / custom shader), so this is false.
        // Callers needing per-mode info should check `supported_blend_modes`.
        blend_modes : false,
        supported_blend_modes : &[ BlendMode::Normal, BlendMode::Add, BlendMode::Multiply, BlendMode::Screen ],
        text_on_path : false,
        // `blend_apply` and `shaders/tint.glsl` honour `ImageAsset::premultiplied`.
        premultiplied_images : true,
        // `new` enables `DEPTH_TEST` / `LEQUAL`; this assumes the context has a
        // depth buffer, the WebGL default ( `depth : false` at context creation
        // leaves it out ).
        depth : true,
        max_texture_size,
      }
    }
  }

  /// The `context_lost` flag, reachable from `tests/` under `test_internals`.
  ///
  /// A loss/restore test needs to both set and read it: setting stands in for a
  /// real `webglcontextlost` DOM event ( nothing in this workspace synthesizes
  /// one, and `WEBGL_lose_context` is not reliably available under the headless
  /// runner ), and reading is how the test observes that `assets_load` — and
  /// nothing else — is what clears it again.
  #[ cfg( feature = "test_internals" ) ]
  impl WebGlBackend
  {
    #[ doc( hidden ) ]
    #[ must_use ]
    pub fn context_lost_for_test( &self ) -> bool
    {
      self.context_lost.get()
    }

    #[ doc( hidden ) ]
    pub fn context_lost_set_for_test( &self, lost : bool )
    {
      self.context_lost.set( lost );
    }
  }

  // ============================================================================
  // Backend trait impl
  // ============================================================================

  impl Backend for WebGlBackend
  {
    fn assets_load( &mut self, assets : &Assets ) -> Result< (), RenderError >
    {
      // Fix(BUG-538): after a loss, the GPU state this backend built in `new`
      // is gone too: a restored context has none of the objects created before
      // the loss and starts from default GL state. Rebuild the shader programs
      // and re-apply that state before re-uploading, or every draw would use a
      // program from the lost context with blending and depth testing off.
      // While the context is still lost nothing can be uploaded, so report that
      // instead of clearing `context_lost` over an empty context.
      // Root cause: only `new` built the programs and GL state, and BUG-441's
      // fix re-uploaded the assets alone before clearing the flag.
      // Pitfall: a restored context keeps nothing from before the loss, not
      // even programs or enabled capabilities.
      if self.context_lost.get()
      {
        if self.gl.is_context_lost()
        {
          return Err( RenderError::ContextLost );
        }
        ( self.sprite, self.mesh ) = Self::renderers_new( &self.gl )?;
        Self::gl_state_init( &self.gl, &self.config );
      }

      // Reset all GPU state: textures, sprites, geometries, and batches.
      // GpuBatch::drop calls delete_vertex_array; ArrayBuffer::drop calls delete_buffer.
      // Safe to call multiple times (e.g. level transitions).
      //
      // Bump the generation counter so any in-flight `spawn_local` futures from
      // a previous cycle notice they are stale and bail out before overwriting
      // entries belonging to this new cycle.
      //
      // ORDER MATTERS: batches must be cleared BEFORE geometries / textures (which
      // are cleared inside `images_load` / `geometries_load` below). A mesh batch's
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
      self.images_load( &assets.images )?;
      self.sprites_load( &assets.sprites );
      self.geometries_load( &assets.geometries )?;
      // Gradients, patterns, clip masks, and fonts are not loaded — the
      // matching `capabilities()` flags are false; roadmap.md owns the plan.

      // Fix(BUG-441): clear `context_lost` here, only after GPU state has actually been
      // re-uploaded above, instead of in the `webglcontextrestored` listener.
      // Root cause: the listener used to clear the flag the instant the browser fired
      // `webglcontextrestored` — but a restored context starts with all GPU objects gone
      // (textures, buffers, VAOs deleted). Between that event firing and the caller getting
      // around to re-calling `assets_load`, `submit`/`output` would see `context_lost ==
      // false` and proceed to issue GL calls against a context with no valid resources —
      // silently drawing nothing / erroring deep inside driver calls instead of returning the
      // documented `RenderError::ContextLost` the caller is supposed to be able to rely on.
      // Pitfall: `context_lost` now means "safe to issue GL calls, GPU state is known-good"
      // rather than merely "the context object is alive" — the two are NOT the same thing
      // across a restore. Any future code that re-populates GPU state some other way (not
      // through `assets_load`) must also clear this flag, or it will stay permanently stuck
      // rejecting `submit`/`output` after a real restoration.
      self.context_lost.set( false );

      Ok( () )
    }

    fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
    {
      if self.context_lost.get()
      {
        return Err( RenderError::ContextLost );
      }

      let viewport = self.viewport_size();

      // Each command stream is self-contained: the scene renderer always emits
      // balanced `BindBatch`…`UnbindBatch` pairs within one stream. Clear any
      // leftover bind state before starting so a prior submit that errored
      // part-way — or a DIFFERENT renderer sharing this backend (e.g. a main
      // renderer plus separate bake renderers) — cannot poison this one. Without
      // this, one mid-stream failure sticks `recording_batch` and every subsequent
      // submit fails on its first `BindBatch` ("batch N is already bound"),
      // cascading into an error spam that never recovers.
      //
      // The stuck batch is unbound the way its missing `UnbindBatch` would
      // have: its staged writes are flushed, so its GPU copy and `gpu_len`
      // match its contents again instead of lagging until it is next bound.
      if let Some( stuck ) = self.recording_batch
      {
        web_sys::console::warn_1
        (
          &format!( "submit: clearing leftover bound batch {stuck:?} from a prior stream" ).into()
        );
        if let Err( e ) = self.cmd_unbind_batch()
        {
          web_sys::console::warn_1( &format!( "submit: flushing leftover bound batch {stuck:?} failed: {e}" ).into() );
        }
      }

      for cmd in commands
      {
        // Unimplemented placeholder arms (Path/Text/Group) all map to {} and are
        // intentionally kept separate for readability and future expansion.
        #[ allow( clippy::match_same_arms, reason = "unimplemented placeholder arms (Path/Text/Group) intentionally kept separate for readability and future expansion" ) ]
        match cmd
        {
          RenderCommand::Clear( c ) => self.cmd_clear( c ),

          // Mesh & sprite
          RenderCommand::Mesh( m ) => self.cmd_mesh( m, viewport )?,
          RenderCommand::Sprite( s ) => self.cmd_sprite( s, true, viewport )?,
          // ScreenSpaceSprite is already in viewport-pixel space (the compile
          // layer anchored it to the viewport, not the world), so it bypasses
          // the camera view matrix — `apply_camera = false` — while a world
          // `Sprite` gets the camera pre-multiplied on the GPU.
          RenderCommand::ScreenSpaceSprite( s ) => self.cmd_sprite( s, false, viewport )?,

          // Batch lifecycle
          RenderCommand::CreateSpriteBatch( c ) => self.cmd_create_sprite_batch( c )?,
          RenderCommand::CreateMeshBatch( c ) => self.cmd_create_mesh_batch( c )?,
          RenderCommand::BindBatch( b ) => self.cmd_bind_batch( *b )?,
          RenderCommand::AddSpriteInstance( si ) => self.cmd_add_sprite_instance( si )?,
          RenderCommand::AddMeshInstance( mi ) => self.cmd_add_mesh_instance( mi )?,
          RenderCommand::SetSpriteInstance( si ) => self.cmd_set_sprite_instance( si )?,
          RenderCommand::SetMeshInstance( mi ) => self.cmd_set_mesh_instance( mi )?,
          RenderCommand::RemoveInstance( ri ) => self.cmd_remove_instance( *ri )?,
          RenderCommand::SetSpriteBatchParams( sp ) => self.cmd_set_sprite_batch_params( sp )?,
          RenderCommand::SetMeshBatchParams( mp ) => self.cmd_set_mesh_batch_params( mp )?,
          RenderCommand::UnbindBatch( _ ) => self.cmd_unbind_batch()?,
          RenderCommand::DrawBatch( db ) => self.cmd_draw_batch( *db, viewport )?,
          RenderCommand::DeleteBatch( db ) => self.cmd_delete_batch( *db ),
          // Opaque/transparent pass split: the scene renderer disables depth
          // writes for the transparent pass so back-to-front blending is not
          // corrupted, then restores them. No depth attachment ⇒ harmless no-op.
          RenderCommand::SetDepthWrite( s ) => self.gl.depth_mask( s.enabled ),

          // Path — skip (unimplemented; see capabilities().paths). Warn on the opener only (not MoveTo/LineTo/etc.)
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

          // Text — skip (unimplemented; see capabilities().text). See note above re: opener-only warning.
          RenderCommand::BeginText( _ ) => web_sys::console::warn_1
          (
            &"WebGlBackend: text commands are not implemented; BeginText..EndText will be ignored (see capabilities().text)".into()
          ),
          RenderCommand::Char( _ )
          | RenderCommand::EndText( _ ) => {}

          // Grouping — skip (unimplemented; see capabilities().effects).
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
      if self.context_lost.get()
      {
        return Err( RenderError::ContextLost );
      }

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
      Self::declared_capabilities( self.max_texture_size )
    }
  }

  // ============================================================================
  // Shared utilities
  // ============================================================================

  // Re-setup any mesh batch VAOs that reference geometry `id`.
  // Batches created before async load completed only have instance attribs;
  // now that geometry buffers are available, add geometry attribs too.
  fn mesh_batch_vaos_refresh( gl : &gl::GL, resources : &Rc< RefCell< GpuResources > >, id : ResourceId< asset::Geometry > )
  {
    let res = resources.borrow();
    if let Some( geom ) = res.geometry( id )
    {
      for batch in res.batches.values()
      {
        if let GpuBatch::Mesh { vao, params, instances, .. } = batch
          && params.geometry == id
        {
          mesh_batch_vao_setup
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
}

mod webgl_surfaces;

mod_interface::mod_interface!
{
  layer render_target;
  layer webgl_helpers;
  layer webgl_renderers;
  layer webgl_textures;

  own use WebGlBackend;
  own use RenderTargetId;
}
