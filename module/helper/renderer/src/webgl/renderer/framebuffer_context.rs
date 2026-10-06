mod private
{
  use minwebgl as gl;
  use gl::GL;

  /// Manages WebGL2 framebuffers and associated renderbuffers/textures for a rendering
  /// context, specifically designed for multisampling and post-processing effects.
  ///
  /// This struct handles the setup and management of two primary framebuffers:
  /// - A **multisample framebuffer** for rendering with anti-aliasing.
  /// - A **resolved framebuffer** for storing the anti-aliased result as textures,
  ///   ready for further post-processing or display.
  ///
  /// It supports two color attachments: a 'main' color buffer and an 'emission' color buffer,
  /// allowing for separate rendering of different visual components.
  pub struct FramebufferContext
  {
    /// The width of the textures attached to the resolved framebuffer.
    pub texture_width : u32,
    /// The height of the textures attached to the resolved framebuffer.
    pub texture_height : u32,
    /// The WebGL framebuffer used for multisampled rendering. This framebuffer
    /// renders into `multisample_main_renderbuffer` and `multisample_emission_renderbuffer`.
    pub multisample_framebuffer : Option< gl::web_sys::WebGlFramebuffer >,
    /// The WebGL framebuffer used to store the resolved (non-multisampled)
    /// textures. This is where the results of the multisample framebuffer
    /// are blitted to, making them ready for sampling.
    pub resolved_framebuffer : Option< gl::web_sys::WebGlFramebuffer >,
    /// The renderbuffer used for depth and stencil testing in the multisample framebuffer.
    #[ allow( dead_code, reason = "never read back after attachment; held so the GPU resource outlives the framebuffer" ) ]
    pub multisample_depth_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
    /// The renderbuffer that receives the main color output during multisampled rendering.
    pub multisample_main_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
    /// The renderbuffer that receives the emission color output during multisampled rendering.
    pub multisample_emission_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer>,
    /// The renderbuffer that accumulates color during blending pass.
    pub multisample_transparent_accumulate_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer>,
     /// The renderbuffer that calculates total revealage during blending pass.
    pub multisample_transparent_revealage_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer>,
    /// The 2D texture that receives the resolved main color output after multisample resolution.
    /// This texture can be sampled in shaders.
    pub main_texture : Option< gl::web_sys::WebGlTexture >,
    /// The 2D texture that receives the resolved emission color output after multisample resolution.
    /// This texture can be sampled in shaders.
    pub emission_texture : Option< gl::web_sys::WebGlTexture >,
    /// The 2D texture that accumulates color during blending pass.
    pub transparent_accumulate_texture : Option< gl::web_sys::WebGlTexture >,
    /// The 2D texture that calculates total revealage during blending pass.
    pub transparent_revealage_texture : Option< gl::web_sys::WebGlTexture >,
    /// The depth-stencil renderbuffer attached to the resolved framebuffer.
    #[ allow( dead_code, reason = "never read back after attachment; held so the GPU resource outlives the framebuffer" ) ]
    pub depth_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
  }

  /// Creates a multisampled renderbuffer and allocates `format` storage for it.
  fn renderbuffer_multisample_create
  (
    gl : &gl::WebGl2RenderingContext,
    samples : i32,
    format : u32,
    width : u32,
    height : u32
  )
  -> Option< gl::web_sys::WebGlRenderbuffer >
  {
    let renderbuffer = gl.create_renderbuffer();
    gl.bind_renderbuffer( gl::RENDERBUFFER, renderbuffer.as_ref() );
    gl.renderbuffer_storage_multisample
    (
      gl::RENDERBUFFER,
      samples,
      format,
      width as i32,
      height as i32
    );
    renderbuffer
  }

  /// Creates a 2D texture with immutable `format` storage, linear filtering, and clamped wrap.
  fn texture_2d_create
  (
    gl : &gl::WebGl2RenderingContext,
    format : u32,
    width : u32,
    height : u32
  )
  -> Option< gl::web_sys::WebGlTexture >
  {
    let texture = gl.create_texture();
    gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
    gl.tex_storage_2d( gl::TEXTURE_2D, 1, format, width as i32, height as i32 );
    gl::texture::d2::filter_linear( gl );
    gl::texture::d2::wrap_clamp( gl );
    texture
  }

  impl FramebufferContext
  {
    /// Creates a new `FramebufferContext` instance, initializing all necessary
    /// WebGL2 framebuffers, renderbuffers, and textures.
    ///
    /// This constructor sets up:
    /// - A multisampled framebuffer with a depth/stencil renderbuffer and two
    ///   multisampled color renderbuffers (`RGBA16F` format).
    /// - A resolved framebuffer with two 2D textures (`RGBA16F` format) to store
    ///   the anti-aliased results.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    /// * `width` - The desired width of the framebuffers and textures.
    /// * `height` - The desired height of the framebuffers and textures.
    /// * `samples` - The number of samples to use for multisample anti-aliasing (e.g., 4, 8).
    ///
    /// # Returns
    ///
    /// A new `FramebufferContext` instance.
    pub fn new( gl : &gl::WebGl2RenderingContext, width : u32, height : u32, samples : i32 ) -> Self
    {
      // Create the core framebuffer objects.
      let multisample_framebuffer = gl.create_framebuffer();
      let resolved_framebuffer = gl.create_framebuffer();

      // Multisampled renderbuffers : depth/stencil plus the four color attachments.
      let multisample_depth_renderbuffer = renderbuffer_multisample_create( gl, samples, gl::DEPTH24_STENCIL8, width, height );
      let multisample_main_renderbuffer = renderbuffer_multisample_create( gl, samples, gl::RGBA16F, width, height );
      let multisample_emission_renderbuffer = renderbuffer_multisample_create( gl, samples, gl::RGBA16F, width, height );
      let multisample_transparent_accumulate_renderbuffer = renderbuffer_multisample_create( gl, samples, gl::RGBA16F, width, height );
      let multisample_transparent_revealage_renderbuffer = renderbuffer_multisample_create( gl, samples, gl::R16F, width, height );
      gl.bind_renderbuffer( gl::RENDERBUFFER, None );

      // Resolved ( non-multisampled ) targets that will receive the blit.
      let depth_renderbuffer = gl.create_renderbuffer();
      gl.bind_renderbuffer( gl::RENDERBUFFER, depth_renderbuffer.as_ref() );
      gl.renderbuffer_storage( gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width as i32, height as i32 );
      let main_texture = texture_2d_create( gl, gl::RGBA16F, width, height );
      let emission_texture = texture_2d_create( gl, gl::RGBA16F, width, height );
      let transparent_accumulate_texture = texture_2d_create( gl, gl::RGBA16F, width, height );
      let transparent_revealage_texture = texture_2d_create( gl, gl::R16F, width, height );

      // --- Attach Renderbuffers to Multisample Framebuffer ---
      // Bind the multisample framebuffer to configure its attachments.
      gl.bind_framebuffer( gl::FRAMEBUFFER, multisample_framebuffer.as_ref() );
      // Attach the depth/stencil renderbuffer.
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, multisample_depth_renderbuffer.as_ref() );
      // Attach the main and emission color renderbuffers as color attachments.
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, multisample_main_renderbuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::RENDERBUFFER, multisample_emission_renderbuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, gl::RENDERBUFFER, multisample_transparent_accumulate_renderbuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT3, gl::RENDERBUFFER, multisample_transparent_revealage_renderbuffer.as_ref() );
      // Specify which color attachments are active for drawing.
      gl::drawbuffers::drawbuffers( gl, &[ 0, 1 ] );

      // --- Attach Textures to Resolved Framebuffer ---
      // Bind the resolved framebuffer to configure its attachments.
      gl.bind_framebuffer( gl::FRAMEBUFFER, resolved_framebuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, depth_renderbuffer.as_ref() );
      // Attach the main and emission textures as color attachments.
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, main_texture.as_ref(), 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, emission_texture.as_ref(), 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, gl::TEXTURE_2D, transparent_accumulate_texture.as_ref(), 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT3, gl::TEXTURE_2D, transparent_revealage_texture.as_ref(), 0 );
      // Specify which color attachments are active for drawing (though for resolved,
      // these will typically be written to via `blit_framebuffer`).
      gl::drawbuffers::drawbuffers( gl, &[ 0, 1 ] );

      // Unbind all resources to clean up the global WebGL state.
      gl.bind_texture( gl::TEXTURE_2D, None );
      gl.bind_renderbuffer( gl::RENDERBUFFER, None );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      let texture_width = width;
      let texture_height = height;

      Self
      {
        texture_width,
        texture_height,
        multisample_framebuffer,
        resolved_framebuffer,
        multisample_depth_renderbuffer,
        multisample_main_renderbuffer,
        multisample_emission_renderbuffer,
        multisample_transparent_accumulate_renderbuffer,
        multisample_transparent_revealage_renderbuffer,
        main_texture,
        emission_texture,
        transparent_accumulate_texture,
        transparent_revealage_texture,
        depth_renderbuffer,
      }
    }

    /// Resolves the multisampled framebuffer into the non-multisampled textures
    /// of the resolved framebuffer using `gl.blit_framebuffer`.
    ///
    /// This operation performs the anti-aliasing process, taking the averaged
    /// color values from the multisample renderbuffers and writing them into
    /// the corresponding textures.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    pub fn resolve( &self, gl : &gl::WebGl2RenderingContext, use_emission : bool, has_transparent : bool )
    {
      self.multisample_bind( gl );
      self.resolved_bind( gl );

      gl.bind_framebuffer( gl::READ_FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
      gl.bind_framebuffer( gl::DRAW_FRAMEBUFFER, self.resolved_framebuffer.as_ref() );

      gl.clear_bufferfi( gl::DEPTH_STENCIL, 0, 1.0, 0 );
      gl.blit_framebuffer
      (
        0, 0, self.texture_width as i32, self.texture_height as i32,
        0, 0, self.texture_width as i32, self.texture_height as i32,
        gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT,
        gl::NEAREST
      );

      gl.read_buffer( gl::COLOR_ATTACHMENT0 );
      gl::drawbuffers::drawbuffers( gl, &[ 0 ] );
      gl.blit_framebuffer
      (
        0, 0, self.texture_width as i32, self.texture_height as i32,
        0, 0, self.texture_width as i32, self.texture_height as i32,
        gl::COLOR_BUFFER_BIT,
        gl::LINEAR
      );

      if use_emission
      {
        gl.read_buffer( gl::COLOR_ATTACHMENT1 );
        gl::drawbuffers::drawbuffers( gl, &[ 1 ] );
        gl.blit_framebuffer
        (
          0, 0, self.texture_width as i32, self.texture_height as i32,
          0, 0, self.texture_width as i32, self.texture_height as i32,
          gl::COLOR_BUFFER_BIT,
          gl::LINEAR
        );
      }

      if has_transparent
      {
        gl.read_buffer( gl::COLOR_ATTACHMENT2 );
        gl::drawbuffers::drawbuffers( gl, &[ 2 ] );
        gl.blit_framebuffer
        (
          0, 0, self.texture_width as i32, self.texture_height as i32,
          0, 0, self.texture_width as i32, self.texture_height as i32,
          gl::COLOR_BUFFER_BIT, gl::LINEAR
        );

        gl.read_buffer( gl::COLOR_ATTACHMENT3 );
        gl::drawbuffers::drawbuffers( gl, &[ 3 ] );
        gl.blit_framebuffer
        (
          0, 0, self.texture_width as i32, self.texture_height as i32,
          0, 0, self.texture_width as i32, self.texture_height as i32,
          gl::COLOR_BUFFER_BIT, gl::LINEAR
        );
      }

      gl.bind_framebuffer( gl::READ_FRAMEBUFFER, None );
      gl.bind_framebuffer( gl::DRAW_FRAMEBUFFER, None );
      self.multisample_unbind( gl );
      self.resolved_unbind( gl );
    }

    /// Binds the `multisample_framebuffer` and attaches its renderbuffers.
    ///
    /// This function should be called before rendering operations that require
    /// multisampling. It ensures that subsequent drawing commands write to
    /// the multisampled color and depth renderbuffers.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    pub fn multisample_bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.viewport( 0, 0, self.texture_width as i32, self.texture_height as i32 );
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
    }

    /// Binds the `resolved_framebuffer` and attaches its textures.
    ///
    /// This function should be called when you want to render directly into
    /// the resolved textures (e.g., for post-processing that doesn't require
    /// multisampling, or after a `resolve` operation).
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    pub fn resolved_bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.viewport( 0, 0, self.texture_width as i32, self.texture_height as i32 );
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.resolved_framebuffer.as_ref() );
    }

    /// Unbinds the color renderbuffers from the `multisample_framebuffer`
    /// and then unbinds the framebuffer itself.
    ///
    /// This function is useful for resetting the framebuffer state after
    /// rendering to the multisample target, preventing accidental draws to it.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    #[ expect( clippy::unused_self, reason = "kept as a method, not an associated fn, for API symmetry with `multisample_bind` -- which does need `self` -- not because this body needs it" ) ]
    pub fn multisample_unbind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    }

    /// Unbinds the color textures from the `resolved_framebuffer` and then
    /// unbinds the framebuffer itself.
    ///
    /// This function is useful for resetting the framebuffer state after
    /// operations that write to the resolved textures, allowing them to be
    /// used as input textures in subsequent rendering passes.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    #[ expect( clippy::unused_self, reason = "kept as a method, not an associated fn, for API symmetry with `resolved_bind` -- which does need `self` -- not because this body needs it" ) ]
    pub fn resolved_unbind( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    }

    /// Free [`FramebufferContext`] WebGL resources
    pub fn gl_resources_free( &mut self, gl : &GL )
    {
      gl.delete_framebuffer( self.resolved_framebuffer.as_ref() );
      gl.delete_framebuffer( self.multisample_framebuffer.as_ref() );
      gl.delete_renderbuffer( self.multisample_depth_renderbuffer.as_ref() );
      gl.delete_renderbuffer( self.multisample_emission_renderbuffer.as_ref() );
      gl.delete_renderbuffer( self.multisample_main_renderbuffer.as_ref() );
      gl.delete_renderbuffer( self.multisample_transparent_accumulate_renderbuffer.as_ref() );
      gl.delete_renderbuffer( self.multisample_transparent_revealage_renderbuffer.as_ref() );
      gl.delete_renderbuffer( self.depth_renderbuffer.as_ref() );
      gl.delete_texture( self.main_texture.as_ref() );
      gl.delete_texture( self.emission_texture.as_ref() );
      gl.delete_texture( self.transparent_accumulate_texture.as_ref() );
      gl.delete_texture( self.transparent_revealage_texture.as_ref() );
    }
  }
}

crate::mod_interface!
{
  own use
  {
    FramebufferContext
  };
}
