mod private
{
  use minwebgl as gl;

  pub trait Pass
  {
    fn render
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      input_texture : Option< gl::web_sys::WebGlTexture >
    ) -> Result< Option< gl::web_sys::WebGlTexture >, gl::WebglError >;
  }


  pub struct Composer
  {
    effects : Vec< Box< dyn Pass > >,
    framebuffer : gl::web_sys::WebGlFramebuffer
  }

  impl Composer 
  {
    pub fn new( gl : &gl::WebGl2RenderingContext ) -> Self
    {
      let effects = Vec::new();
      let framebuffer = gl.create_framebuffer().expect( "Failed to create a framebuffer for the composer" );

      Self
      {
        effects,
        framebuffer
      }
    }

    pub fn add_pass< T : Into< Box< dyn Pass > > >( &mut self, pass : T )
    {
      self.effects.push( pass.into() );
    }

    pub fn get_framebuffer( &self ) -> &gl::web_sys::WebGlFramebuffer
    {
      &self.framebuffer
    }

    pub fn render
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      mut input : Option< gl::web_sys::WebGlTexture >
    ) -> Result< Option< gl::web_sys::WebGlTexture >, gl::WebglError >
    {
      for e in self.effects.iter()
      {
        input = e.render( gl, input )?;
      }

      Ok( input )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Composer,
    Pass
  };
}