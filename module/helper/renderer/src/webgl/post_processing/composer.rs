mod private
{
  use minwebgl as gl;

  pub trait Pass
  {
    fn render
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      input_texture : Option< &gl::web_sys::WebGlTexture >
    ) -> Result< Option< gl::web_sys::WebGlTexture >, gl::WebglError >;
  }


  pub struct Composer
  {
    effects : Vec< Box< dyn Pass > >
  }

  impl Composer 
  {
    pub fn new() -> Self
    {
      let effects = Vec::new();

      Self
      {
        effects
      }
    }

    // pub fn render< 'a >
    // (
    //   &'a self,
    //   gl : &gl::WebGl2RenderingContext,
    //   mut input : Option< &'a gl::web_sys::WebGlTexture >
    // )
    // {
    //   for e in self.effects.iter()
    //   {
    //     input = e.render( gl, input );
    //   }
    // }
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