use crate::
{
  filters,
  framebuffer,
  wasm_bindgen,
};
use filters::{ FilterRenderer, Filter };
use framebuffer::Framebuffer;
use minwebgl as gl;
use gl::GL;
use wasm_bindgen::JsCast;
use web_sys::
{
  WebGlProgram,
  WebGlTexture,
};

pub struct Renderer
{
  gl : GL,
  program : Option< WebGlProgram >,
  framebuffer : Framebuffer,
  image_texture : Option< WebGlTexture >,
  original_texture : Option< WebGlTexture >,
  previous_texture : Option< WebGlTexture >,
  previous_canvas_size : Option< ( u32, u32 ) >,
  current_filter_source : String,
}

impl Renderer
{
  const VERTEX_SOURCE : &'static str = include_str!( "shaders/main.vert" );

  pub fn new( gl : &GL, image_texture : Option< WebGlTexture > ) -> Self
  {
    let width = gl.drawing_buffer_width();
    let height = gl.drawing_buffer_height();
    let framebuffer = Framebuffer::new( gl, width, height ).expect( "Can't create framebuffer" );

    Self
    {
      gl : gl.clone(),
      program : None,
      framebuffer,
      image_texture,
      original_texture : None,
      previous_texture : None,
      previous_canvas_size : None,
      current_filter_source : String::new(),
    }
  }

  // Fix(BUG-463): delete the outgoing texture before overwriting the handle,
  // unless `original_texture` still aliases the exact same GL object ( true
  // right after upload, where both fields are cloned from the same fresh
  // texture -- see `main.rs`'s `image_handler_create` ).
  // Root cause: replacing `self.image_texture`/`self.original_texture` only
  // drops the Rust-side JS handle wrapper -- it never tells the GL driver to
  // free the underlying GPU texture, which requires an explicit
  // `gl.delete_texture` call. Every image upload, drag-drop, Apply click, and
  // Revert replaced these fields without ever issuing that call, leaking a
  // texture every time.
  // Pitfall: `image_texture`/`original_texture` can alias the same underlying
  // GL object ( immediately after upload, or after `original_texture_restore` )
  // -- deleting unconditionally on every replace would delete a texture the
  // *sibling* field still needs, corrupting whatever renders from it next.
  // Always check the sibling field before deleting; never delete blindly.
  //
  // Fix(BUG-503): also skip the delete when `image_texture` ( the incoming
  // replacement ) is the exact same handle as `old` ( the outgoing one ) --
  // a self-assignment through this setter, which the aliasing guard above
  // does not cover since it only ever compares against `original_texture`.
  // Root cause: `previous_texture_restore` always calls this setter with a
  // clone of whatever `previous_texture_save` cloned out of `image_texture`
  // earlier -- and nothing between those two calls ever mutates
  // `image_texture` ( every `Filter::draw` takes `&impl FilterRenderer`, an
  // immutable borrow with no setter access, so a filter preview can only draw
  // to the canvas/framebuffer, never replace this field ). So `old` and the
  // incoming `image_texture` are the *same* handle on every real Cancel, and
  // without this check the pre-existing guard alone deletes that shared
  // handle the instant `original_texture` no longer aliases it too ( i.e.
  // any time at least one "Apply" has already baked a new base texture in
  // this session ) -- leaving `self.image_texture` pointing at a texture
  // `gl.delete_texture` just freed.
  // Pitfall: an aliasing guard that only special-cases one *other* field
  // (`original_texture`) can still miss the simplest aliasing case of all --
  // the incoming value aliasing the very value it's replacing. Always check
  // self-assignment first, before reasoning about any other field.
  pub fn image_texture_set( &mut self, image_texture : Option< WebGlTexture > )
  {
    if let Some( old ) = self.image_texture.take()
    {
      let aliases_original = self.original_texture.as_ref() == Some( &old );
      let is_self_assign = image_texture.as_ref() == Some( &old );
      if !aliases_original && !is_self_assign
      {
        self.gl.delete_texture( Some( &old ) );
      }
    }
    self.image_texture = image_texture;
  }

  // Fix(BUG-463): see `image_texture_set` -- same leak, same aliasing guard, mirrored.
  // Fix(BUG-503): see `image_texture_set` -- same self-assignment guard, mirrored
  // defensively ( no currently-reachable call site drives `original_texture_set`
  // into the self-assignment case the way `previous_texture_restore` drives
  // `image_texture_set`, but this function is documented as mirroring the other
  // one field-for-field, so its guard completeness is kept in lockstep too ).
  pub fn original_texture_set( &mut self, original_texture : Option< WebGlTexture > )
  {
    if let Some( old ) = self.original_texture.take()
    {
      let aliases_image = self.image_texture.as_ref() == Some( &old );
      let is_self_assign = original_texture.as_ref() == Some( &old );
      if !aliases_image && !is_self_assign
      {
        self.gl.delete_texture( Some( &old ) );
      }
    }
    self.original_texture = original_texture;
  }

  pub fn original_texture_restore( &mut self )
  {
    // Fix(BUG-463): route through `image_texture_set` instead of assigning
    // `self.image_texture` directly, so the outgoing texture ( e.g. an applied
    // filter result ) is properly deleted instead of silently leaked.
    if let Some( original ) = self.original_texture.clone()
    {
      self.image_texture_set( Some( original ) );
    }
  }

  pub fn previous_texture_save( &mut self )
  {
    self.previous_texture = self.image_texture.clone();
    // Save current canvas dimensions so they can be restored on cancel
    if let Some( canvas ) = self.gl.canvas()
    {
      if let Ok( canvas ) = canvas.dyn_into::< web_sys::HtmlCanvasElement >()
      {
        self.previous_canvas_size = Some( ( canvas.width(), canvas.height() ) );
      }
    }
  }

  pub fn previous_texture_restore( &mut self )
  {
    // Fix(BUG-463): route through `image_texture_set` -- see `original_texture_restore`.
    // Fix(BUG-503): `previous` is always the same handle `image_texture` already
    // holds at this point ( nothing between the matching `previous_texture_save`
    // call and this one ever mutates `image_texture` -- see `image_texture_set`'s
    // own Fix(BUG-503) note ), so this call depends on that setter's
    // self-assignment guard to avoid deleting the handle it's restoring to.
    if let Some( previous ) = self.previous_texture.take()
    {
      self.image_texture_set( Some( previous ) );
    }
    // Restore canvas dimensions
    if let Some( ( w, h ) ) = self.previous_canvas_size.take()
    {
      if let Some( canvas ) = self.gl.canvas()
      {
        if let Ok( canvas ) = canvas.dyn_into::< web_sys::HtmlCanvasElement >()
        {
          canvas.set_width( w );
          canvas.set_height( h );
        }
      }
    }
  }

  pub fn previous_state_clear( &mut self )
  {
    self.previous_texture = None;
    self.previous_canvas_size = None;
  }

  pub fn framebuffer_size_update( &mut self, width : i32, height : i32 )
  {
    self.framebuffer = Framebuffer::new( &self.gl, width, height ).expect( "Can't create framebuffer" );
  }

  pub fn filter_apply( &mut self, filter : &impl Filter )
  {
    if self.image_texture.is_none()
    {
      return;
    }

    let filter_source = filter.glsl_fragment_source();
    if self.current_filter_source != filter_source
    {
      // Recompile program
      self.program = Some( Self::program_create( &self.gl, &filter_source ) );
      self.current_filter_source = filter_source;
    }

    filter.draw( self );

    // If a filter changed the canvas dimensions (e.g. Transpose), sync the framebuffer so
    // subsequent filters that use it (e.g. blur two-pass) see the correct size.
    if let Some( canvas ) = self.gl.canvas()
    {
      if let Ok( canvas ) = canvas.dyn_into::< web_sys::HtmlCanvasElement >()
      {
        let w = canvas.width() as i32;
        let h = canvas.height() as i32;
        if w != self.framebuffer.width() || h != self.framebuffer.height()
        {
          self.framebuffer_size_update( w, h );
        }
      }
    }
  }

  fn program_create( gl : &GL, filter_source : &str ) -> WebGlProgram
  {
    gl::ProgramFromSources::new( Self::VERTEX_SOURCE, filter_source )
    .compile_and_link( gl )
    .expect( "Unable to compile program" )
  }
}

impl FilterRenderer for Renderer
{
  fn gl( &self ) -> &GL
  {
    &self.gl
  }

  fn get_image_texture( &self ) -> Option< &WebGlTexture >
  {
    self.image_texture.as_ref()
  }

  fn get_program( &self ) -> &WebGlProgram
  {
    self.program.as_ref().expect( "No filter was provided" )
  }

  fn draw( &self )
  {
    self.gl.draw_arrays( GL::TRIANGLES, 0, 3 );
  }

  fn get_framebuffer( &self ) -> &Framebuffer
  {
    &self.framebuffer
  }
}
