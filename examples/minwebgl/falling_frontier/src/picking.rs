//! Real object picking (M5), replacing M3's ground-click focus stand-in.
//! GPU ID-buffer approach ported from `examples/minwebgl/object_picking`:
//! every pickable `HullPart` carries a small integer `pick_id`, rendered
//! into an off-screen `R32I` texture through `IdProgram`, then read back
//! with a single-pixel `read_pixels` at the click location.
//!
//! Two differences from `object_picking`'s own version: it draws with
//! `u_view_proj * u_model` (that example's camera sits fixed at the world
//! origin, so it only ever uploads a projection matrix) and it resizes
//! alongside the canvas (that example's canvas is a fixed 1280x720 and never
//! needs to).
//!
//! This module only knows about ids as opaque `i32`s - mapping a picked id
//! back to "asteroid 3" or "the station" is `main.rs`'s job (it owns the id
//! ranges handed out to each of `asteroids`/`ships`/`station`), same as
//! `object_picking` itself never claims stable meaning for ids beyond "index
//! into `objects`".

use minwebgl as gl;
use gl::GL;

use crate::hull::HullPart;

struct IdUniforms
{
  view_proj : Option< gl::WebGlUniformLocation >,
  model : Option< gl::WebGlUniformLocation >,
  id : Option< gl::WebGlUniformLocation >,
}

pub struct IdProgram
{
  program : gl::WebGlProgram,
  uniforms : IdUniforms,
}

impl IdProgram
{
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

  fn draw_part( &self, gl : &GL, part : &HullPart )
  {
    let u = &self.uniforms;
    gl::uniform::matrix_upload( gl, u.model.clone(), part.model.to_array().as_slice(), true ).unwrap();
    gl.uniform1i( u.id.as_ref(), part.pick_id );

    gl.bind_vertex_array( Some( &part.vao ) );
    gl.draw_elements_with_i32( GL::TRIANGLES, part.index_count, GL::UNSIGNED_INT, 0 );
  }
}

/// Off-screen `R32I` id texture + depth renderbuffer, sized to match the
/// canvas. Rendered on demand (once per click, not once per frame) since
/// nothing in the scene moves yet (M4 scope is static placement) - M7's
/// fleet motion will need this re-rendered right before each pick instead of
/// relying on a stale one, same as `object_picking`'s own comment on
/// `ids_render` notes.
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
  /// an in-place reallocation. No-op if the size hasn't actually changed
  /// (canvas resize fires on every observed frame-size check in `main.rs`,
  /// most of which aren't real size changes).
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
  /// `gizmo_part`, if given, is drawn last with depth test off - matching
  /// `Gizmo::draw`'s own depth-test-off visible pass (the handle is meant to
  /// stay clickable through the object it belongs to). Without this, the
  /// gizmo's flat handle geometry loses the depth test against its own
  /// object's hull most of the time (the hull's mesh usually has some
  /// geometry closer to the camera than the handle's paper-thin plane), so
  /// the id buffer would report the object's id instead of the handle's
  /// right where a drag is supposed to start.
  pub fn render< 'a >( &self, gl : &GL, id_program : &IdProgram, view_proj : gl::F32x4x4, parts : impl Iterator< Item = &'a HullPart >, gizmo_part : Option< &HullPart > )
  {
    gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    gl.viewport( 0, 0, self.width, self.height );
    gl.clear_bufferiv_with_i32_array( gl::COLOR, 0, [ -1, -1, -1, -1 ].as_slice() );
    gl.clear( GL::DEPTH_BUFFER_BIT );

    id_program.begin_frame( gl, view_proj );
    for part in parts { id_program.draw_part( gl, part ); }

    if let Some( part ) = gizmo_part
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
    ( id >= 0 ).then_some( id )
  }
}
