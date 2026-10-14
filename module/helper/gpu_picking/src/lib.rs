//! GPU ID-buffer object picking: render every pickable part's small integer
//! id into an off-screen `R32I` texture through [`IdProgram`], then read a
//! single pixel back at a click location via [`PickBuffer::pick`] to find
//! out what's there. No CPU-side ray/AABB intersection math needed — the
//! GPU already rasterized exactly what's visible at that pixel.
//!
//! Callers implement [`Pickable`] for whatever their own "one drawable
//! part" type already is (own VAO, index count, world transform, pick id)
//! — this crate never needs to know anything else about it.

use minwebgl as gl;
use gl::GL;

/// Anything that can be drawn into an id-picking pass: its own VAO, index
/// count, world transform, and pick id. Implement this directly on whatever
/// struct already represents "one drawable part" in the caller's own scene.
pub trait Pickable
{
  /// The VAO to bind before drawing this part.
  fn vao( &self ) -> &gl::WebGlVertexArrayObject;
  /// Index count for the `TRIANGLES` `drawElements` call — must match
  /// `vao`'s bound element array buffer.
  fn index_count( &self ) -> i32;
  /// This part's current world transform.
  fn model( &self ) -> gl::F32x4x4;
  /// The id written into the id texture wherever this part is visible.
  /// Read back by [`PickBuffer::pick`].
  fn pick_id( &self ) -> i32;
}

struct IdUniforms
{
  view_proj : Option< gl::WebGlUniformLocation >,
  model : Option< gl::WebGlUniformLocation >,
  id : Option< gl::WebGlUniformLocation >,
}

/// Draws [`Pickable`] parts' ids into whatever framebuffer is currently
/// bound — normally [`PickBuffer`]'s own, via [`PickBuffer::render`].
pub struct IdProgram
{
  program : gl::WebGlProgram,
  uniforms : IdUniforms,
}

impl IdProgram
{
  /// # Panics
  /// Panics if the id shader fails to compile or link.
  #[ must_use ]
  pub fn new( gl : &GL ) -> Self
  {
    let vertex_shader = include_str!( "shaders/id.vert" );
    let fragment_shader = include_str!( "shaders/id.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( gl )
    .unwrap();

    let uniforms = IdUniforms
    {
      view_proj : gl.get_uniform_location( &program, "u_view_proj" ),
      model : gl.get_uniform_location( &program, "u_model" ),
      id : gl.get_uniform_location( &program, "u_id" ),
    };

    Self { program, uniforms }
  }

  fn begin_frame( &self, gl : &GL, view_proj : gl::F32x4x4 )
  {
    gl.use_program( Some( &self.program ) );
    gl::uniform::matrix_upload( gl, self.uniforms.view_proj.clone(), view_proj.to_array().as_slice(), true ).unwrap();
  }

  fn draw_part< P : Pickable >( &self, gl : &GL, part : &P )
  {
    let u = &self.uniforms;
    let id = part.pick_id();
    // Fix(BUG-513): a `Pickable::pick_id()` of `-1` (or any negative value)
    // rendered successfully but could never be read back by `PickBuffer::pick`.
    // Root cause: `pick_id`'s doc comment never stated the `>= 0` constraint
    // implied by `readback_to_pick_id` treating `-1` as the reserved
    // background sentinel, so nothing caught a caller violating it.
    // Pitfall: silently clamping or re-mapping a negative id here instead of
    // asserting would hide the caller's bug behind a part that renders but
    // stays permanently unpickable, with no diagnostic at all.
    assert_pick_id_valid( id );
    gl::uniform::matrix_upload( gl, u.model.clone(), part.model().to_array().as_slice(), true ).unwrap();
    gl.uniform1i( u.id.as_ref(), id );

    gl.bind_vertex_array( Some( part.vao() ) );
    gl.draw_elements_with_i32( GL::TRIANGLES, part.index_count(), GL::UNSIGNED_INT, 0 );
  }
}

/// Off-screen `R32I` id texture + depth renderbuffer, sized to match the
/// canvas. Render on demand (e.g. once per click) rather than every frame
/// unless parts move between picks and staleness would matter.
pub struct PickBuffer
{
  // Fix(BUG-521): needed so `impl Drop` below can free `framebuffer` /
  // `id_texture` / `depth_renderbuffer` -- `Drop::drop` takes no extra
  // arguments, so a GL handle to call `delete_*` with must be owned here.
  // Root cause: none of this struct's fields was a GL context handle, so no
  // `impl Drop` was even possible without this field.
  // Pitfall: `resize`'s own `gl : &GL` parameter is a *different* borrow
  // each call -- don't remove this owned clone thinking that parameter can
  // substitute for it; `Drop::drop` cannot take parameters at all.
  gl : GL,
  framebuffer : Option< gl::web_sys::WebGlFramebuffer >,
  id_texture : Option< gl::web_sys::WebGlTexture >,
  depth_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
  width : i32,
  height : i32,
  readback : gl::js_sys::Int32Array,
}

impl PickBuffer
{
  /// Creates the id texture + depth renderbuffer at `width`x`height`.
  #[ must_use ]
  pub fn new( gl : &GL, width : i32, height : i32 ) -> Self
  {
    let framebuffer = gl.create_framebuffer();
    let mut buf = Self
    {
      gl : gl.clone(),
      framebuffer,
      id_texture : None,
      depth_renderbuffer : None,
      width : 0,
      height : 0,
      readback : gl::js_sys::Int32Array::new_with_length( 1 ),
    };
    buf.resize( gl, width, height );
    buf
  }

  /// Recreates the id texture/depth buffer at the new size - `tex_storage_2d`
  /// is immutable-storage, so a resize means delete-and-recreate rather than
  /// an in-place reallocation. No-op if the size hasn't actually changed.
  pub fn resize( &mut self, gl : &GL, width : i32, height : i32 )
  {
    if width == self.width && height == self.height { return; }
    self.width = width;
    self.height = height;

    if let Some( tex ) = self.id_texture.take() { gl.delete_texture( Some( &tex ) ); }
    if let Some( rb ) = self.depth_renderbuffer.take() { gl.delete_renderbuffer( Some( &rb ) ); }

    let id_texture = gl.create_texture();
    gl.bind_texture( GL::TEXTURE_2D, id_texture.as_ref() );
    gl.tex_storage_2d( GL::TEXTURE_2D, 1, GL::R32I, width, height );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

    let depth_renderbuffer = gl.create_renderbuffer();
    gl.bind_renderbuffer( GL::RENDERBUFFER, depth_renderbuffer.as_ref() );
    gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT16, width, height );

    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, id_texture.as_ref(), 0 );
    gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, depth_renderbuffer.as_ref() );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    self.id_texture = id_texture;
    self.depth_renderbuffer = depth_renderbuffer;
  }

  /// Re-renders every part's id at its current transform. Caller is
  /// responsible for restoring the viewport afterward - this always sets it
  /// to the buffer's own size while drawing.
  ///
  /// `overlay_part`, if given, is drawn last with depth test off - useful
  /// for a handle/gizmo that should stay pickable through the object it's
  /// attached to (its *visible* draw pass presumably also renders it with
  /// depth test off, for the same reason; without matching that here, the
  /// id pass would report the underlying object's id instead of the
  /// handle's wherever the two overlap).
  ///
  /// # Panics
  /// Panics if any drawn part's [`Pickable::pick_id`] is negative — `-1`
  /// (and every other negative value) is reserved as the background
  /// sentinel (see [`readback_to_pick_id`]) and can never be read back by
  /// [`PickBuffer::pick`].
  pub fn render< 'a, P : Pickable + 'a >
  (
    &self, gl : &GL, id_program : &IdProgram, view_proj : gl::F32x4x4,
    parts : impl Iterator< Item = &'a P >, overlay_part : Option< &P >,
  )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.viewport( 0, 0, self.width, self.height );
    gl.clear_bufferiv_with_i32_array( gl::COLOR, 0, [ -1, -1, -1, -1 ].as_slice() );
    gl.clear( GL::DEPTH_BUFFER_BIT );

    id_program.begin_frame( gl, view_proj );
    for part in parts { id_program.draw_part( gl, part ); }

    if let Some( part ) = overlay_part
    {
      gl.disable( GL::DEPTH_TEST );
      id_program.draw_part( gl, part );
      gl.enable( GL::DEPTH_TEST );
    }

    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
  }

  /// Reads the id at `(x, y)` - canvas-local, bottom-up pixel coordinates
  /// (matching `read_pixels`'s own origin), same size as the buffer itself.
  /// Returns `None` for the "nothing here" background id (`-1`) *and* for
  /// any `(x, y)` outside `[0, width) x [0, height)`.
  ///
  /// # Panics
  /// Panics if the underlying `read_pixels` call fails (e.g. a lost
  /// context).
  #[ must_use ]
  pub fn pick( &self, gl : &GL, x : i32, y : i32 ) -> Option< i32 >
  {
    // Fix(BUG-530): out-of-range `(x, y)` were passed straight to
    // `read_pixels` with no validation against the buffer's own bounds.
    // Root cause: no bounds check existed at all -- an out-of-range read's
    // outcome was left entirely to driver-specific `read_pixels` behavior,
    // and on the very first pick of a freshly-created buffer `self.readback`
    // starts zero-filled (JS `TypedArray`s always zero-initialize), so an
    // out-of-range read that leaves it untouched reads back as a false
    // `Some(0)` instead of `None`.
    // Pitfall: re-adding an out-of-range `pick` call path (e.g. a new
    // convenience wrapper) without routing it through `pick_in_bounds` first
    // would silently reopen this.
    if !pick_in_bounds( x, y, self.width, self.height ) { return None; }

    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.read_buffer( GL::COLOR_ATTACHMENT0 );
    gl.read_pixels_with_array_buffer_view_and_dst_offset
    (
      x, y, 1, 1, GL::RED_INTEGER, GL::INT, &self.readback, 0
    ).unwrap();
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    let id = self.readback.to_vec()[ 0 ];
    readback_to_pick_id( id )
  }
}

/// The three GL handles [`PickBuffer`]'s [`Drop`] frees, exposed only under
/// `test_internals`.
///
/// Fields cannot be re-exported the way functions can, so the gate takes the
/// shape of accessors rather than a `pub use` in [`internal`]. Each returns a
/// clone: the point is to hold a handle *past* the buffer's own lifetime and
/// ask the context whether the object still exists, which a borrow could not
/// outlive.
#[ cfg( feature = "test_internals" ) ]
#[ doc( hidden ) ]
impl PickBuffer
{
  #[ must_use ]
  pub fn framebuffer_for_test( &self ) -> Option< gl::web_sys::WebGlFramebuffer >
  {
    self.framebuffer.clone()
  }

  #[ must_use ]
  pub fn id_texture_for_test( &self ) -> Option< gl::web_sys::WebGlTexture >
  {
    self.id_texture.clone()
  }

  #[ must_use ]
  pub fn depth_renderbuffer_for_test( &self ) -> Option< gl::web_sys::WebGlRenderbuffer >
  {
    self.depth_renderbuffer.clone()
  }
}

// Fix(BUG-521): `PickBuffer` allocated a framebuffer, id texture, and depth
// renderbuffer but never freed any of them on drop.
// Root cause: no `impl Drop` existed at all -- `resize` deletes the
// *previous* texture/renderbuffer before replacing them, but nothing ever
// freed the *last* one, nor the framebuffer itself, on final teardown.
// Pitfall: adding a new GL-resource field to this struct without also
// extending this `Drop` impl reintroduces the same leak for that field.
impl Drop for PickBuffer
{
  fn drop( &mut self )
  {
    if let Some( fb ) = self.framebuffer.take() { self.gl.delete_framebuffer( Some( &fb ) ); }
    if let Some( tex ) = self.id_texture.take() { self.gl.delete_texture( Some( &tex ) ); }
    if let Some( rb ) = self.depth_renderbuffer.take() { self.gl.delete_renderbuffer( Some( &rb ) ); }
  }
}

/// The crate's pure, context-free logic — everything decidable without a live
/// `WebGl2RenderingContext`.
///
/// Private, so none of it is surface. Each item is `pub` *within* this module
/// rather than crate-private, which is what lets the `test_internals` gate
/// below re-export it to `tests/` without any of it becoming reachable by
/// default.
mod pick_logic
{
  /// Maps a raw id-texture readback value to a picked id: `-1` is the
  /// "nothing here" background sentinel written by `PickBuffer::render`'s
  /// `clear_bufferiv_with_i32_array`; anything else is a genuine `pick_id`
  /// (see [`crate::Pickable::pick_id`]). Pulled out of
  /// [`crate::PickBuffer::pick`] as its own function so this sentinel mapping
  /// — the one piece of interpretive logic in this crate that isn't a direct
  /// GL call — is testable without a live `WebGl2RenderingContext`.
  #[ must_use ]
  pub fn readback_to_pick_id( raw : i32 ) -> Option< i32 >
  {
    ( raw >= 0 ).then_some( raw )
  }

  /// Whether `(x, y)` names an in-bounds pixel of a `width`x`height` buffer.
  /// Pulled out of [`crate::PickBuffer::pick`] so this bounds check is
  /// testable without a live `WebGl2RenderingContext` — same rationale as
  /// [`readback_to_pick_id`] just above.
  #[ must_use ]
  pub fn pick_in_bounds( x : i32, y : i32, width : i32, height : i32 ) -> bool
  {
    x >= 0 && y >= 0 && x < width && y < height
  }

  /// Panics if `id` is negative — `-1` (and every other negative value) is
  /// reserved as [`crate::PickBuffer::render`]'s background-clear sentinel
  /// (see [`readback_to_pick_id`]), so a [`crate::Pickable::pick_id`] using
  /// one would render successfully but could never be read back by
  /// [`crate::PickBuffer::pick`]. Pulled out of
  /// [`crate::IdProgram::draw_part`] so this validation is testable without a
  /// live `WebGl2RenderingContext`.
  ///
  /// # Panics
  /// Panics if `id` is negative. That is the function's entire purpose — the
  /// caller has violated [`crate::Pickable::pick_id`]'s contract, and failing
  /// loudly here is what stops the part from rendering fine while being
  /// permanently unpickable.
  pub fn assert_pick_id_valid( id : i32 )
  {
    assert!
    (
      id >= 0,
      "Pickable::pick_id() returned {id}, but pick ids must be >= 0 -- negative \
       values (starting with -1) are reserved as PickBuffer's \"nothing picked\" \
       background sentinel and can never be read back by PickBuffer::pick()"
    );
  }
}

use pick_logic::{ assert_pick_id_valid, pick_in_bounds, readback_to_pick_id };

/// The internals `tests/` reaches, exposed only under `test_internals`.
///
/// Not part of the surface. With the feature off — the default, and what every
/// dependent gets — this module does not exist and neither
/// [`pick_logic`] nor `PickBuffer`'s GL handles are reachable from outside.
#[ cfg( feature = "test_internals" ) ]
#[ doc( hidden ) ]
pub mod internal
{
  pub use crate::pick_logic::{ assert_pick_id_valid, pick_in_bounds, readback_to_pick_id };
}
