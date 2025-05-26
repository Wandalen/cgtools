mod private
{
  use minwebgl as gl;

  pub trait Pass
  {
    fn renders_to_input( &self ) -> bool;

    fn render
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      input_texture : Option< gl::web_sys::WebGlTexture >,
      output_texture : Option< gl::web_sys::WebGlTexture >
    ) -> Result< Option< gl::web_sys::WebGlTexture >, gl::WebglError >;
  }

  pub struct SwapFramebuffer
  {
    framebuffer : Option< gl::web_sys::WebGlFramebuffer >,
    renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
    original_output : Option< gl::web_sys::WebGlTexture >,
    input_texture : Option< gl::web_sys::WebGlTexture >,
    output_texture : Option< gl::web_sys::WebGlTexture >
  }

  impl SwapFramebuffer 
  {
    pub fn new( gl : &gl::WebGl2RenderingContext, width : u32, height : u32 ) -> Self
    {
      let framebuffer = gl.create_framebuffer();
      gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
      gl::drawbuffers::drawbuffers( &gl, &[ gl::COLOR_ATTACHMENT0 ] );

      let renderbuffer = gl.create_renderbuffer();
      gl.bind_renderbuffer( gl::RENDERBUFFER, renderbuffer.as_ref() );
      gl.renderbuffer_storage( gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width as i32, height as i32 );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, renderbuffer.as_ref() );

      gl.bind_renderbuffer( gl::RENDERBUFFER, None );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      let output_texture = gl.create_texture();
      gl.bind_texture( gl::TEXTURE_2D, output_texture.as_ref() );
      gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA16F, width as i32, height as i32 );
      gl::texture::d2::filter_linear( gl );
      gl.bind_texture( gl::TEXTURE_2D, None );

      let original_output = output_texture.clone();
      let input_texture = None;

      Self
      {
        framebuffer,
        renderbuffer,
        input_texture,
        output_texture,
        original_output
      }
    }

    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.framebuffer.as_ref() );
    }

    pub fn swap( &mut self )
    {
      let tmp = self.input_texture.clone();
      self.input_texture = self.output_texture.clone();
      self.output_texture = tmp;
    }

    pub fn reset( &mut self )
    {
      self.output_texture = self.original_output.clone();
    }

    pub fn unbind_attachment( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.bind( gl );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, None, 0 );
    }

    pub fn set_input( &mut self, texture : Option< gl::web_sys::WebGlTexture > )
    {
      self.input_texture = texture;
    }

    pub fn set_output( &mut self, texture : Option< gl::web_sys::WebGlTexture > )
    {
      self.output_texture = texture;
    }


    pub fn get_input( &self ) -> Option< gl::web_sys::WebGlTexture >
    {
      self.input_texture.clone()
    }

    pub fn get_output( &self ) -> Option< gl::web_sys::WebGlTexture >
    {
      self.output_texture.clone()
    }
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

    // pub fn render
    // (
    //   &self,
    //   gl : &gl::WebGl2RenderingContext,
    //   mut input : Option< gl::web_sys::WebGlTexture >
    // ) -> Result< Option< gl::web_sys::WebGlTexture >, gl::WebglError >
    // {
    //   for e in self.effects.iter()
    //   {
    //     input = e.render( gl, input )?;
    //   }

    //   Ok( input )
    // }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Composer,
    Pass,
    SwapFramebuffer
  };
}