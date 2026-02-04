mod private
{
  use minwebgl as gl;

  /// A trait defining the interface for a rendering pass.
  pub trait Pass
  {
    /// Indicates whether this rendering pass renders to its input texture.
    fn renders_to_input( &self ) -> bool;

    /// Renders post-processing effect.
    fn render
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      input_texture : Option< gl::web_sys::WebGlTexture >,
      output_texture : Option< gl::web_sys::WebGlTexture >
    ) -> Result< Option< gl::web_sys::WebGlTexture >, gl::WebglError >;
  }

  // Manages a pair of textures within a single framebuffer to facilitate ping-pong
  /// rendering, where the output of one pass becomes the input for the next.
  pub struct SwapFramebuffer
  {
    /// The WebGL framebuffer used for rendering. It has a single color attachment.
    framebuffer : Option< gl::web_sys::WebGlFramebuffer >,
    /// A clone of the initial `output_texture` created upon instantiation.
    /// This is used to `reset` the output texture to its original state.
    original_output : Option< gl::web_sys::WebGlTexture >,
    /// The texture currently serving as the input for a rendering pass.
    /// After a `swap`, this will hold the result from the previous `output_texture`.
    input_texture : Option< gl::web_sys::WebGlTexture >,
    /// The texture currently serving as the output for a rendering pass.
    /// After a `swap`, its content will become the new `input_texture`.
    output_texture : Option< gl::web_sys::WebGlTexture >
  }

  impl SwapFramebuffer
  {
    /// Creates a new `SwapFramebuffer` instance, initializing its WebGL framebuffer,
    /// renderbuffer, and the primary output texture.
    ///
    /// The framebuffer is configured with a single color attachment point and a
    /// depth/stencil renderbuffer. An initial `output_texture` is created with
    /// `RGBA16F` format for high precision.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    /// * `width` - The width of the textures and framebuffer.
    /// * `height` - The height of the textures and framebuffer.
    pub fn new( gl : &gl::WebGl2RenderingContext, width : u32, height : u32 ) -> Self
    {
      // Create and configure the framebuffer.
      let framebuffer = gl.create_framebuffer();
      gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
      // Specify that only COLOR_ATTACHMENT0 is used for drawing.
      gl::drawbuffers::drawbuffers( &gl, &[ 0 ] );

      // Unbind renderbuffer and framebuffer to clean up global state.
      gl.bind_renderbuffer( gl::RENDERBUFFER, None );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      // Create the primary output texture.
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
        input_texture,
        output_texture,
        original_output
      }
    }

    /// Binds the internal framebuffer of this `SwapFramebuffer` instance.
    ///
    /// This should be called before any drawing commands that are intended
    /// to write to the `output_texture` managed by this `SwapFramebuffer`.
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.framebuffer.as_ref() );
      gl::drawbuffers::drawbuffers( gl, &[ 0 ] );
    }

    /// Swaps the `input_texture` and `output_texture`.
    pub fn swap( &mut self )
    {
      let tmp = self.input_texture.clone();
      self.input_texture = self.output_texture.clone();
      self.output_texture = tmp;
    }

    /// Resets the `output_texture` to its original texture.
    pub fn reset( &mut self )
    {
      self.output_texture = self.original_output.clone();
    }

    /// Unbinds the color attachment from the internal framebuffer.
    pub fn unbind_attachment( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.bind( gl );
      gl::clean::framebuffer_texture_2d( gl );
    }

    /// Sets the `input_texture` of the `SwapFramebuffer`.
    pub fn set_input( &mut self, texture : Option< gl::web_sys::WebGlTexture > )
    {
      self.input_texture = texture;
    }

    /// Sets the `output_texture` of the `SwapFramebuffer`.
    pub fn set_output( &mut self, texture : Option< gl::web_sys::WebGlTexture > )
    {
      self.output_texture = texture;
    }


    /// Returns the current `input_texture`.
    pub fn get_input( &self ) -> Option< gl::web_sys::WebGlTexture >
    {
      self.input_texture.clone()
    }

    /// Returns the current `output_texture`.
    pub fn get_output( &self ) -> Option< gl::web_sys::WebGlTexture >
    {
      self.output_texture.clone()
    }

    /// Free [`SwapFramebuffer`] WebGL resources
    ///
    /// Because [`SwapFramebuffer`] can use textures that are shared with other structs,
    /// only the framebuffer is deleted here to avoid accidental deletion of shared textures.
    pub fn free_gl_resources( &mut self, gl : &gl::GL )
    {
      gl.delete_framebuffer( self.framebuffer.as_ref() );
    }
  }


  /// The `Composer` struct manages a chain of rendering passes
  pub struct Composer
  {
    effects : Vec< Box< dyn Pass > >,
    framebuffer : gl::web_sys::WebGlFramebuffer
  }

  impl Composer
  {
    /// Creates a new `Composer` instance.
    ///
    /// This function initializes an empty vector for effects and creates a
    /// new WebGL framebuffer. The framebuffer is essential for rendering
    /// post-processing effects to textures.
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

    /// Adds a new rendering pass to the composer's effect chain.
    pub fn add_pass< T : Into< Box< dyn Pass > > >( &mut self, pass : T )
    {
      self.effects.push( pass.into() );
    }

    /// Returns a reference to the WebGL framebuffer.
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
