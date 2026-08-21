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

/// Maps a raw id-texture readback value to a picked id: `-1` is the
/// "nothing here" background sentinel written by `PickBuffer::render`'s
/// `clear_bufferiv_with_i32_array`; anything else is a genuine `pick_id`
/// (see [`Pickable::pick_id`]). Pulled out of [`PickBuffer::pick`] as its
/// own function so this sentinel mapping — the one piece of interpretive
/// logic in this crate that isn't a direct GL call — is testable without a
/// live `WebGl2RenderingContext`.
fn readback_to_pick_id( raw : i32 ) -> Option< i32 >
{
  ( raw >= 0 ).then_some( raw )
}

/// Whether `(x, y)` names an in-bounds pixel of a `width`x`height` buffer.
/// Pulled out of [`PickBuffer::pick`] so this bounds check is testable
/// without a live `WebGl2RenderingContext` — same rationale as
/// [`readback_to_pick_id`] just above.
fn pick_in_bounds( x : i32, y : i32, width : i32, height : i32 ) -> bool
{
  x >= 0 && y >= 0 && x < width && y < height
}

/// Panics if `id` is negative — `-1` (and every other negative value) is
/// reserved as [`PickBuffer::render`]'s background-clear sentinel (see
/// [`readback_to_pick_id`]), so a [`Pickable::pick_id`] using one would
/// render successfully but could never be read back by [`PickBuffer::pick`].
/// Pulled out of [`IdProgram::draw_part`] so this validation is testable
/// without a live `WebGl2RenderingContext`.
fn assert_pick_id_valid( id : i32 )
{
  assert!
  (
    id >= 0,
    "Pickable::pick_id() returned {id}, but pick ids must be >= 0 -- negative \
     values (starting with -1) are reserved as PickBuffer's \"nothing picked\" \
     background sentinel and can never be read back by PickBuffer::pick()"
  );
}

// `IdProgram`/`PickBuffer`'s own methods all require a live
// `WebGl2RenderingContext` to construct or call (framebuffers, textures,
// shader compilation), which a native `cargo nextest` run cannot provide —
// same Wasm Native-Check Blind Spot already established in this workspace
// (see `primitive_generation/tests/geometry_normal_attribute_test.rs`).
// `readback_to_pick_id`, `pick_in_bounds`, and `assert_pick_id_valid` above
// are the only pure, context-free logic this crate has to test natively;
// live-GL-context tests (e.g. GPU-resource-teardown checks) live in the
// wasm32-only `live_gl_tests` module below instead.
#[ cfg( test ) ]
mod tests
{
  use super::{ readback_to_pick_id, pick_in_bounds, assert_pick_id_valid };

  #[ test ]
  fn background_sentinel_maps_to_none()
  {
    assert_eq!( readback_to_pick_id( -1 ), None, "-1 is the documented background sentinel" );
  }

  #[ test ]
  fn zero_and_positive_ids_map_to_some()
  {
    assert_eq!( readback_to_pick_id( 0 ), Some( 0 ), "id 0 is a valid, pickable id, not background" );
    assert_eq!( readback_to_pick_id( 7 ), Some( 7 ) );
    assert_eq!( readback_to_pick_id( i32::MAX ), Some( i32::MAX ) );
  }

  /// ## Root Cause
  /// `PickBuffer::pick` passed `(x, y)` straight to `read_pixels` with no
  /// validation against the buffer's own `(width, height)` -- an
  /// out-of-range coordinate's outcome was left entirely to driver-specific
  /// `read_pixels` behavior instead of being handled by this crate, and a
  /// freshly-created buffer's `readback` starts zero-filled (JS
  /// `TypedArray`s always zero-initialize), so an out-of-range read that
  /// leaves it untouched reads back as a false `Some(0)` instead of `None`.
  ///
  /// ## Why Not Caught
  /// `gpu_picking` had zero test coverage of any kind before this sweep --
  /// nothing exercised out-of-range pick coordinates, and the bug only
  /// manifests as a wrong *value* (not a panic or compile error), so it
  /// would silently misreport picks near/past the canvas edge.
  ///
  /// ## Fix Applied
  /// Added `pick_in_bounds`, called at the top of `PickBuffer::pick` --
  /// returns `None` immediately for any `(x, y)` outside
  /// `[0, width) x [0, height)`, before ever touching the GPU.
  ///
  /// ## Prevention
  /// These cases cover all four boundary edges (`x`/`y` each at `-1` and at
  /// exactly `width`/`height`, the classic off-by-one edge) plus the
  /// degenerate `0x0` buffer, so a regression re-widening or dropping the
  /// check trips at least one boundary case immediately.
  ///
  /// ## Pitfall
  /// `x < width` (not `x <= width`) is the correct upper check -- valid
  /// columns are `0..width`, so `x == width` is one column past the last
  /// valid one and must be rejected, not accepted.
  // test_kind: bug_reproducer(BUG-530)
  #[ test ]
  fn pick_in_bounds_rejects_out_of_range_coordinates()
  {
    assert!( pick_in_bounds( 0, 0, 4, 4 ), "(0,0) is the first valid pixel" );
    assert!( pick_in_bounds( 3, 3, 4, 4 ), "(3,3) is the last valid pixel of a 4x4 buffer" );

    assert!( !pick_in_bounds( -1, 0, 4, 4 ), "negative x must be rejected" );
    assert!( !pick_in_bounds( 0, -1, 4, 4 ), "negative y must be rejected" );
    assert!( !pick_in_bounds( 4, 0, 4, 4 ), "x == width is one past the last valid column" );
    assert!( !pick_in_bounds( 0, 4, 4, 4 ), "y == height is one past the last valid row" );
    assert!( !pick_in_bounds( 0, 0, 0, 0 ), "a 0x0 buffer has no valid pixel at all" );
  }

  /// ## Root Cause
  /// `Pickable::pick_id`'s doc comment never stated the `>= 0` constraint
  /// implied by `readback_to_pick_id` treating `-1` as the reserved
  /// background sentinel -- a part using `-1` (or any negative id) rendered
  /// successfully but could never be read back by `PickBuffer::pick`,
  /// silently making it permanently unpickable.
  ///
  /// ## Why Not Caught
  /// `gpu_picking` had zero test coverage of any kind before this sweep, and
  /// the failure mode is silent (no panic, no error -- the part just never
  /// gets picked), so nothing would surface it short of a user noticing an
  /// object simply doesn't respond to clicks.
  ///
  /// ## Fix Applied
  /// Added `assert_pick_id_valid`, called from `IdProgram::draw_part` for
  /// every part drawn -- panics loudly (instead of silently producing an
  /// unpickable part) the moment a negative `pick_id` is used.
  ///
  /// ## Prevention
  /// Covers the exact boundary (`-1`, the reserved sentinel itself) plus the
  /// valid boundary (`0`) and two representative valid values, so a
  /// regression loosening the check to allow negatives trips immediately.
  ///
  /// ## Pitfall
  /// `id >= 0` (not `id > 0`) is the correct check -- `0` is a valid,
  /// pickable id (see `zero_and_positive_ids_map_to_some` above); only
  /// negative values are reserved.
  // test_kind: bug_reproducer(BUG-513)
  #[ test ]
  #[ should_panic( expected = "pick ids must be >= 0" ) ]
  fn negative_pick_id_panics()
  {
    assert_pick_id_valid( -1 );
  }

  #[ test ]
  fn zero_and_positive_pick_ids_are_accepted()
  {
    assert_pick_id_valid( 0 );
    assert_pick_id_valid( 42 );
    assert_pick_id_valid( i32::MAX );
  }
}

// Live-GL-context test, so wasm32-only; needs `PickBuffer`'s private fields
// to capture handles before drop -- see `rulebook.md § Test placement`.
#[ cfg( all( test, target_arch = "wasm32" ) ) ]
mod live_gl_tests
{
  use super::*;
  use wasm_bindgen_test::wasm_bindgen_test;

  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap()
  }

  /// ## Root Cause
  /// `PickBuffer` allocated a framebuffer, id texture, and depth
  /// renderbuffer in `new`/`resize` but had no `impl Drop` -- every
  /// construct/drop cycle permanently leaked all three for the GL context's
  /// lifetime (`resize` deletes the *previous* texture/renderbuffer before
  /// replacing them, but nothing ever freed the *last* one, nor the
  /// framebuffer itself, on final teardown).
  ///
  /// ## Why Not Caught
  /// `gpu_picking` had zero test coverage of any kind before this sweep --
  /// nothing exercised `PickBuffer`'s construction or destruction, so a
  /// missing `Drop` impl produced no observable failure.
  ///
  /// ## Fix Applied
  /// Added a `gl : GL` field (an owned clone, populated in `new`, matching
  /// `renderer::webgl::ShadowBaker`'s identical precedent in this workspace
  /// -- see BUG-432) plus `impl Drop for PickBuffer` deleting `framebuffer`,
  /// `id_texture`, and `depth_renderbuffer`.
  ///
  /// ## Prevention
  /// This test captures clones of all three private GL handles right after
  /// construction, asserts each is a live GL object, drops the `PickBuffer`,
  /// then asserts none of the three are live any more -- the same
  /// deterministic `gl.is_*` existence-check pattern used by this
  /// workspace's other GPU-teardown reproducer tests (e.g. BUG-432's
  /// `shadow_baker_drop_frees_framebuffer`).
  ///
  /// ## Pitfall
  /// A GPU handle wrapper (`Option< WebGlTexture >` etc.) is just a JS-object
  /// reference -- letting the Rust value go out of scope does not call
  /// `gl.delete*` for you; only an explicit delete call (here, via
  /// `impl Drop`) reclaims the actual GPU-side allocation.
  // test_kind: bug_reproducer(BUG-521)
  #[ wasm_bindgen_test ]
  fn pick_buffer_drop_frees_gl_resources()
  {
    let gl = gl_init();
    let buffer = PickBuffer::new( &gl, 4, 4 );

    let framebuffer = buffer.framebuffer.clone();
    let id_texture = buffer.id_texture.clone();
    let depth_renderbuffer = buffer.depth_renderbuffer.clone();

    // `new`'s own `resize` call already binds `framebuffer` while attaching
    // `id_texture`/`depth_renderbuffer`, so it already reports as a live GL
    // object here with no extra bind step needed (contrast BUG-432's
    // `ShadowBaker`, which only binds its framebuffer later, in `target_set`).
    assert!( gl.is_framebuffer( framebuffer.as_ref() ), "framebuffer must be a live GL object right after construction" );
    assert!( gl.is_texture( id_texture.as_ref() ), "id_texture must be a live GL object right after construction" );
    assert!( gl.is_renderbuffer( depth_renderbuffer.as_ref() ), "depth_renderbuffer must be a live GL object right after construction" );

    drop( buffer );

    assert!( !gl.is_framebuffer( framebuffer.as_ref() ), "PickBuffer::drop must delete its framebuffer" );
    assert!( !gl.is_texture( id_texture.as_ref() ), "PickBuffer::drop must delete its id_texture" );
    assert!( !gl.is_renderbuffer( depth_renderbuffer.as_ref() ), "PickBuffer::drop must delete its depth_renderbuffer" );
  }
}
