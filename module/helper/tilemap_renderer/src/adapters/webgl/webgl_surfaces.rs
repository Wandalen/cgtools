//! [`WebGlBackend`]'s offscreen render targets (cache surfaces), the scissor and
//! clear they are repainted with, and the world-space quads that composite them
//! (or a tiled image) back into the frame. A child of [`super`], so it reaches
//! the backend's fields directly; the methods stay inherent on `WebGlBackend`.

use wasm_bindgen::prelude::*;
use minwebgl as gl;
use super::private::{ RenderTargetId, WebGlBackend };
use super::webgl_helpers::{ GpuFramebuffer, blend_apply, mat3_mul };
use crate::types::{ BlendMode, ResourceId, asset };

impl WebGlBackend
{
  // ========================================================================
  // Offscreen render targets (cache surfaces)
  //
  // These let a consumer bake a subset of the frame into a texture once (or
  // occasionally) and composite it back as a single world-space quad every
  // frame, trading per-frame overdraw for a couple of texture-sample blits.
  // They are inherent methods (like `camera_set`), driven by the consumer
  // between `submit` calls — the `RenderCommand` stream and the other backends
  // are untouched. `submit` never sets the GL viewport: `new` (and a context
  // restore, through `gl_state_init`) and `resize` set it to the canvas size,
  // and every draw projects through `viewport_size`.
  // So `render_target_begin` sets both the GL viewport and `active_viewport`
  // (which `viewport_size` returns while a target is bound) to the target's
  // size, and `render_target_end` restores the canvas size.
  // ========================================================================

  /// Hardware `MAX_TEXTURE_SIZE`. A cache surface may not exceed this in either
  /// dimension; the caller clamps its bake resolution to fit.
  #[ inline ]
  #[ must_use ]
  pub fn max_texture_size( &self ) -> u32 { self.max_texture_size }

  /// Whether every registered atlas/image texture has finished loading (a
  /// non-zero size). Images upload ASYNCHRONOUSLY: a freshly registered texture
  /// is a `0×0` placeholder until its bytes arrive, and a draw referencing a
  /// still-zero sheet is skipped (`draw_batch` early-returns). A consumer that
  /// bakes the scene ONCE into a cache must wait for this — a bake fired before
  /// the sheets load rasterizes nothing yet never repeats. Returns `false`
  /// while no texture is registered at all, so it cannot pass before the
  /// spec's assets are loaded.
  #[ must_use ]
  pub fn textures_loaded( &self ) -> bool
  {
    let res = self.resources.borrow();
    !res.textures.is_empty()
      && res.textures.values().all( | t | t.width.get() > 0 && t.height.get() > 0 )
  }

  /// Allocate an offscreen cache surface (`width×height`, RGBA8 + depth) and
  /// return its handle, or `None` if allocation / completeness fails. The
  /// handle stays valid until the next [`Self::render_targets_reset`].
  pub fn render_target_create( &mut self, width : u32, height : u32 ) -> Option< RenderTargetId >
  {
    let fb = GpuFramebuffer::new( &self.gl, width, height )?;
    self.render_targets.push( fb )
  }

  /// Free every cache surface and invalidate every [`RenderTargetId`] handed
  /// out so far. A context restore (`assets_load` after a loss) does this
  /// too. The GL objects are released via each `GpuFramebuffer`'s
  /// `Drop`. Call before rebuilding a fresh set of targets (e.g. a chunk grid
  /// at a new extent / map load) so the old textures don't leak.
  pub fn render_targets_reset( &mut self )
  {
    self.render_targets.clear();
  }

  /// Runs `bake` with cache surface `id` bound, as [`Self::render_target_begin`]
  /// binds it, and always restores the canvas with [`Self::render_target_end`]
  /// afterwards, whatever `bake` returns. Prefer it to calling the two by hand:
  /// a bake whose `submit` fails part-way would otherwise return before the
  /// `render_target_end`, and every later frame would draw into the cache
  /// surface at its viewport.
  ///
  /// Returns `bake`'s result, or `None` without calling it when `id` is stale.
  pub fn render_target_with< R >
  (
    &mut self,
    id : RenderTargetId,
    clear : Option< [ f32; 4 ] >,
    bake : impl FnOnce( &mut Self ) -> R,
  )
  -> Option< R >
  {
    if !self.render_target_begin( id, clear )
    {
      return None;
    }
    let result = bake( self );
    self.render_target_end();
    Some( result )
  }

  /// Bind cache surface `id` as the draw target and set the viewport to its
  /// size. `clear` optionally clears it (colour + depth) — pass a transparent
  /// colour for an overlay cache that must composite over another. Pair with
  /// [`Self::render_target_end`], or use [`Self::render_target_with`], which
  /// pairs them for you.
  ///
  /// Returns `false`, binding nothing, if `id` is stale (made before the last
  /// [`Self::render_targets_reset`] or context restore): the draws that
  /// follow would then land on the canvas, so the caller should skip its bake
  /// and create the surface again.
  pub fn render_target_begin( &mut self, id : RenderTargetId, clear : Option< [ f32; 4 ] > ) -> bool
  {
    // Copy out size + a handle clone before mutating `active_viewport` (which
    // would otherwise conflict with the `render_targets` borrow).
    let ( w, h, fbh ) = match self.render_targets.get( id )
    {
      Some( fb ) => ( fb.width, fb.height, fb.framebuffer.clone() ),
      None => return false,
    };
    self.gl.bind_framebuffer( gl::FRAMEBUFFER, Some( &fbh ) );
    self.gl.viewport( 0, 0, w as i32, h as i32 );
    self.active_viewport = Some( [ w as f32, h as f32 ] );
    if let Some( c ) = clear
    {
      // Tell a tiled (mobile) GPU it need not LOAD the prior attachment
      // contents into tile memory before this pass — the `fill` below fully
      // redefines both colour and depth immediately, so discarding first is
      // safe and skips the load-in bandwidth.
      let discard = gl::js_sys::Array::new();
      discard.push( &JsValue::from_f64( f64::from( gl::COLOR_ATTACHMENT0 ) ) );
      discard.push( &JsValue::from_f64( f64::from( gl::DEPTH_ATTACHMENT ) ) );
      let _ = self.gl.invalidate_framebuffer( gl::FRAMEBUFFER, &discard );
      self.fill( c );
    }
    true
  }

  /// Restore the default framebuffer and the canvas-sized viewport after a
  /// cache-surface pass.
  pub fn render_target_end( &mut self )
  {
    // The depth attachment is write-only scratch for the bake's early-Z; only
    // the colour texture is sampled when compositing. Discard depth before
    // unbinding so a tiled GPU skips the store-back of the depth tile to main
    // memory. Guarded on `active_viewport` so this only touches a bound cache
    // FBO (whose `DEPTH_ATTACHMENT` token is valid), never the default one.
    if self.active_viewport.is_some()
    {
      let discard = gl::js_sys::Array::new();
      discard.push( &JsValue::from_f64( f64::from( gl::DEPTH_ATTACHMENT ) ) );
      let _ = self.gl.invalidate_framebuffer( gl::FRAMEBUFFER, &discard );
    }
    self.active_viewport = None;
    self.gl.disable( gl::SCISSOR_TEST );
    self.gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    self.gl.viewport( 0, 0, self.config.width as i32, self.config.height as i32 );
  }

  /// Restrict every following draw and clear — [`Self::fill`] included — to
  /// the pixel rectangle `[ x, y, width, height ]` of the current draw target
  /// (GL convention: origin at the bottom-left), or lift the restriction with
  /// `None`. Lets a cache pass repaint only part of a surface: bind it with
  /// [`Self::render_target_begin`]`( id, None )` to keep its contents, scissor
  /// the changed area, `fill`, then redraw. [`Self::render_target_end`] always
  /// lifts the scissor, so it never leaks past the cache pass.
  pub fn scissor_set( &mut self, rect : Option< [ i32; 4 ] > )
  {
    match rect
    {
      Some( [ x, y, w, h ] ) =>
      {
        self.gl.enable( gl::SCISSOR_TEST );
        self.gl.scissor( x, y, w.max( 0 ), h.max( 0 ) );
      },
      None => self.gl.disable( gl::SCISSOR_TEST ),
    }
  }

  /// Clear the current draw target (colour + depth) to `color`.
  pub fn fill( &self, color : [ f32; 4 ] )
  {
    let [ r, g, b, a ] = color;
    // A depth clear is a no-op while `depthMask` is false (e.g. left off by a
    // prior transparent pass), which would leave the FBO's depth buffer at its
    // stale value and make the opaque bake pass fail its `LEQUAL` test. Force
    // writes on so the clear actually resets depth to the far plane.
    self.gl.depth_mask( true );
    self.gl.clear_color( r, g, b, a );
    self.gl.clear_depth( 1.0 );
    self.gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );
  }

  /// Composite cache surface `id` as a textured quad covering the world-pixel
  /// rectangle `[ min, max ]`, projected through the current camera (so it pans
  /// and zooms with the board). The cache is treated as premultiplied "over".
  /// No-op if `id` is stale.
  ///
  /// V-orientation: uploaded atlases use `UNPACK_FLIP_Y`, so their visual top
  /// sits at the high texel row — the same as an FBO render — so the stock
  /// sprite shader composites the cache upright with a plain `[0,0,w,h]` region.
  /// If a cache ever renders vertically mirrored, negate the region height
  /// (`[0, h, w, -h]`).
  pub fn render_target_draw( &self, id : RenderTargetId, min : [ f32; 2 ], max : [ f32; 2 ] )
  {
    let Some( fb ) = self.render_targets.get( id ) else { return; };
    let ( tw, th ) = ( fb.width as f32, fb.height as f32 );
    if tw == 0.0 || th == 0.0 { return; }

    // Map the unit quad (scaled by region.zw = [tw, th] in the vertex shader)
    // onto the world rect: scale by world_extent / tex_px, translate to `min`.
    // Column-major 3×3 (index j*3 + i), same layout as `Transform::to_mat3`.
    let sx = ( max[ 0 ] - min[ 0 ] ) / tw;
    let sy = ( max[ 1 ] - min[ 1 ] ) / th;
    let world_mat = [ sx, 0.0, 0.0, 0.0, sy, 0.0, min[ 0 ], min[ 1 ], 1.0 ];
    let mat = mat3_mul( &self.camera, &world_mat );

    self.gl.active_texture( gl::TEXTURE0 );
    self.gl.bind_texture( gl::TEXTURE_2D, Some( &fb.color ) );
    blend_apply( &self.gl, &BlendMode::Normal, true );

    let region = [ 0.0, 0.0, tw, th ];
    let tint = [ 1.0_f32, 1.0, 1.0, 1.0 ];
    let viewport = self.viewport_size();
    self.sprite.draw( &self.gl, &mat, &region, [ tw, th ], &tint, true, viewport, 0.0, self.config.max_depth );
  }

  /// Draw `image` repeated over the world rectangle `[ min, max ]` as ONE quad,
  /// one copy per `tile_size` world pixels, with a copy's top-left corner at
  /// `tile_top_left` (any copy — only its position modulo `tile_size` matters),
  /// multiplied by `tint`, through the current camera at depth 0. A whole
  /// tiled backdrop then costs one draw call and one layer of fill instead of an
  /// instanced sprite per tile. The image must be loaded with
  /// `WrapMode::Repeat`, otherwise the texture edge smears across the rect.
  /// No-op if `image` is unknown or not loaded yet.
  pub fn image_tiled_draw
  (
    &self,
    image : ResourceId< asset::Image >,
    min : [ f32; 2 ],
    max : [ f32; 2 ],
    tile_top_left : [ f32; 2 ],
    tile_size : [ f32; 2 ],
    tint : [ f32; 4 ],
  )
  {
    let res = self.resources.borrow();
    let Some( tex ) = res.texture( image ) else { return; };
    let ( tw, th ) = ( tex.width.get() as f32, tex.height.get() as f32 );
    let ( w, h ) = ( max[ 0 ] - min[ 0 ], max[ 1 ] - min[ 1 ] );
    if tw == 0.0 || th == 0.0 || w <= 0.0 || h <= 0.0 || tile_size[ 0 ] <= 0.0 || tile_size[ 1 ] <= 0.0 { return; }

    // The sprite shader samples `uv = region / tex_size` with image rows running
    // top-down and draws the unit quad scaled to `region.zw`. A region larger
    // than the texture therefore repeats it. `region.xy` is where the rect's
    // top-left corner falls inside a tile, in texels — taken modulo one tile so
    // the numbers stay small (and precise) far from the world origin.
    let tiles_x = ( min[ 0 ] - tile_top_left[ 0 ] ) / tile_size[ 0 ];
    let tiles_y = ( tile_top_left[ 1 ] - max[ 1 ] ) / tile_size[ 1 ];
    let region =
    [
      tiles_x.rem_euclid( 1.0 ) * tw,
      tiles_y.rem_euclid( 1.0 ) * th,
      w / tile_size[ 0 ] * tw,
      h / tile_size[ 1 ] * th,
    ];
    // Scale the region-sized quad back onto the world rect.
    let world_mat = [ w / region[ 2 ], 0.0, 0.0, 0.0, h / region[ 3 ], 0.0, min[ 0 ], min[ 1 ], 1.0 ];
    let mat = mat3_mul( &self.camera, &world_mat );

    self.gl.active_texture( gl::TEXTURE0 );
    self.gl.bind_texture( gl::TEXTURE_2D, Some( &tex.texture ) );
    blend_apply( &self.gl, &BlendMode::Normal, tex.premultiplied );
    let viewport = self.viewport_size();
    self.sprite.draw( &self.gl, &mat, &region, [ tw, th ], &tint, tex.premultiplied, viewport, 0.0, self.config.max_depth );
  }
}
