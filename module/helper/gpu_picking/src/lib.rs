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
    gl::uniform::matrix_upload( gl, u.model.clone(), part.model().to_array().as_slice(), true ).unwrap();
    gl.uniform1i( u.id.as_ref(), part.pick_id() );

    gl.bind_vertex_array( Some( part.vao() ) );
    gl.draw_elements_with_i32( GL::TRIANGLES, part.index_count(), GL::UNSIGNED_INT, 0 );
  }
}

/// Off-screen `R32I` id texture + depth renderbuffer, sized to match the
/// canvas. Render on demand (e.g. once per click) rather than every frame
/// unless parts move between picks and staleness would matter.
pub struct PickBuffer
{
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
  /// Returns `None` for the "nothing here" background id (`-1`).
  ///
  /// # Panics
  /// Panics if the underlying `read_pixels` call fails (e.g. a lost
  /// context).
  #[ must_use ]
  pub fn pick( &self, gl : &GL, x : i32, y : i32 ) -> Option< i32 >
  {
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

// `IdProgram`/`PickBuffer`'s own methods all require a live
// `WebGl2RenderingContext` to construct or call (framebuffers, textures,
// shader compilation), which a native `cargo nextest` run cannot provide —
// same Wasm Native-Check Blind Spot already established in this workspace
// (see `primitive_generation/tests/geometry_normal_attribute_test.rs`).
// `readback_to_pick_id` above is the only pure, context-free logic this
// crate has to test natively.
#[ cfg( test ) ]
mod tests
{
  use super::readback_to_pick_id;

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
}
