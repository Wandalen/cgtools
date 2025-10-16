mod private
{
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use minwebgl as gl;

  use crate::webgl::
  {
    post_processing::
    {
      BlendPass,
      Pass,
      SwapFramebuffer,
      UnrealBloomPass,
      VS_TRIANGLE
    },
    program::{ self, CompositeShader },
    AlphaMode,
    Camera,
    Node,
    Object3D,
    Primitive,
    ProgramInfo,
    Scene,
    IBL
  };

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
    #[ allow( dead_code ) ]
    pub multisample_depth_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
    /// The renderbuffer that receives the main color output during multisampled rendering.
    pub multisample_main_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
    /// The renderbuffer that receives the emission color output during multisampled rendering.
    pub multisample_emission_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer>,
    /// The renderbuffer that accumulates color during blending pass.
    pub multisample_transparent_accumulate_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer>,
     /// The renderbuffer that caclulates total revealage during blending pass.
    pub multisample_transparent_revealage_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer>,
    /// The 2D texture that receives the resolved main color output after multisample resolution.
    /// This texture can be sampled in shaders.
    pub main_texture : Option< gl::web_sys::WebGlTexture >,
    /// The 2D texture that receives the resolved emission color output after multisample resolution.
    /// This texture can be sampled in shaders.
    pub emission_texture : Option< gl::web_sys::WebGlTexture >,
    /// The 2D texture that accumulates color during blending pass.
    pub transparent_accumulate_texture : Option< gl::web_sys::WebGlTexture >,
    /// The 2D texture that caclulates total revealage during blending pass.
    pub transparent_revealage_texture : Option< gl::web_sys::WebGlTexture >,
    #[ allow( dead_code ) ]
    pub depth_renderbuffer : Option< gl::web_sys::WebGlRenderbuffer >,
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

      // --- Setup Depth/Stencil Renderbuffer ---
      // Create and bind a renderbuffer for depth and stencil information.
      let multisample_depth_renderbuffer = gl.create_renderbuffer();
      gl.bind_renderbuffer( gl::RENDERBUFFER, multisample_depth_renderbuffer.as_ref() );
      // Allocate storage for the depth/stencil renderbuffer with multisampling.
      gl.renderbuffer_storage_multisample
      (
        gl::RENDERBUFFER,
        samples,
        gl::DEPTH24_STENCIL8,
        width as i32,
        height as i32
      );

      // --- Setup Multisample Main Color Renderbuffer ---
      // Create and bind a renderbuffer for the main color attachment,
      // which will be multisampled.
      let multisample_main_renderbuffer = gl.create_renderbuffer();
      gl.bind_renderbuffer( gl::RENDERBUFFER, multisample_main_renderbuffer.as_ref() );
      // Allocate storage for the main color renderbuffer with multisampling.
      gl.renderbuffer_storage_multisample
      (
        gl::RENDERBUFFER,
        samples,
        gl::RGBA16F,
        width as i32,
        height as i32
      );

      // --- Setup Multisample Emission Color Renderbuffer ---
      // Create and bind a renderbuffer for the emission color attachment,
      // also multisampled.
      let multisample_emission_renderbuffer = gl.create_renderbuffer();
      gl.bind_renderbuffer( gl::RENDERBUFFER, multisample_emission_renderbuffer.as_ref() );
      // Allocate storage for the emission color renderbuffer with multisampling.
      gl.renderbuffer_storage_multisample
      (
        gl::RENDERBUFFER,
        samples,
        gl::RGBA16F,
        width as i32,
        height as i32
      );

      let multisample_transparent_accumulate_renderbuffer = gl.create_renderbuffer();
      gl.bind_renderbuffer( gl::RENDERBUFFER, multisample_transparent_accumulate_renderbuffer.as_ref() );
      // Allocate storage for the emission color renderbuffer with multisampling.
      gl.renderbuffer_storage_multisample
      (
        gl::RENDERBUFFER,
        samples,
        gl::RGBA16F,
        width as i32,
        height as i32
      );

      let multisample_transparent_revealage_renderbuffer = gl.create_renderbuffer();
      gl.bind_renderbuffer( gl::RENDERBUFFER, multisample_transparent_revealage_renderbuffer.as_ref() );
      // Allocate storage for the emission color renderbuffer with multisampling.
      gl.renderbuffer_storage_multisample
      (
        gl::RENDERBUFFER,
        samples,
        gl::R16F,
        width as i32,
        height as i32
      );

      gl.bind_renderbuffer( gl::RENDERBUFFER, None );


      // --- Create Resolved Textures ---
      // These textures will store the final, resolved (non-multisampled)
      // color information after blitting.
      let depth_renderbuffer = gl.create_renderbuffer();
      let main_texture = gl.create_texture();
      let emission_texture = gl.create_texture();
      let transparent_accumulate_texture = gl.create_texture();
      let transparent_revealage_texture = gl.create_texture();

      gl.bind_renderbuffer( gl::RENDERBUFFER, depth_renderbuffer.as_ref() );
      gl.renderbuffer_storage( gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width as i32, height as i32 );

      // Configure the main texture.
      gl.bind_texture( gl::TEXTURE_2D, main_texture.as_ref() );
      gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA16F, width as i32, height  as i32 );
      gl::texture::d2::filter_linear( gl );
      gl::texture::d2::wrap_clamp( gl );

      // Configure the emission texture.
      gl.bind_texture( gl::TEXTURE_2D, emission_texture.as_ref() );
      gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA16F, width  as i32, height  as i32 );
      gl::texture::d2::filter_linear( gl );
      gl::texture::d2::wrap_clamp( gl );

      // Configure the  texture.
      gl.bind_texture( gl::TEXTURE_2D, transparent_accumulate_texture.as_ref() );
      gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::RGBA16F, width as i32, height  as i32 );
      gl::texture::d2::filter_linear( gl );
      gl::texture::d2::wrap_clamp( gl );

      // Configure the  texture.
      gl.bind_texture( gl::TEXTURE_2D, transparent_revealage_texture.as_ref() );
      gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::R16F, width  as i32, height  as i32 );
      gl::texture::d2::filter_linear( gl );
      gl::texture::d2::wrap_clamp( gl );


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
        texture_height,
        texture_width,
        resolved_framebuffer,
        multisample_framebuffer,
        multisample_depth_renderbuffer,
        multisample_emission_renderbuffer,
        multisample_main_renderbuffer,
        multisample_transparent_accumulate_renderbuffer,
        multisample_transparent_revealage_renderbuffer,
        depth_renderbuffer,
        main_texture,
        emission_texture,
        transparent_accumulate_texture,
        transparent_revealage_texture
      }
    }

    /// Configures both the multisample and resolved framebuffers to allow
    /// drawing to both `COLOR_ATTACHMENT0` (main) and `COLOR_ATTACHMENT1` (emission).
    ///
    /// This is useful when you want to render both normal scene data and
    /// emissive properties in a single pass.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    pub fn enable_emission_texture( &self, gl : &gl::WebGl2RenderingContext )
    {
      // Enable both attachments for the multisample framebuffer.
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
      gl::drawbuffers::drawbuffers( gl, &[ 0, 1 ] );

      // Enable both attachments for the resolved framebuffer.
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.resolved_framebuffer.as_ref() );
      gl::drawbuffers::drawbuffers( gl, &[ 0, 1 ] );

      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    }

    /// Configures both the multisample and resolved framebuffers to allow
    /// drawing only to `COLOR_ATTACHMENT0` (main), effectively disabling
    /// rendering to the emission texture.
    ///
    /// This can be used when the emission channel is not needed or to optimize
    /// rendering passes.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    pub fn disable_emission_texture( &self, gl : &gl::WebGl2RenderingContext )
    {
      // Disable emission attachment for the multisample framebuffer.
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
      gl::drawbuffers::drawbuffers( gl, &[ 0 ] );

      // Disable emission attachment for the resolved framebuffer.
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.resolved_framebuffer.as_ref() );
      gl::drawbuffers::drawbuffers( gl, &[ 0 ] );

      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
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
    pub fn resolve( &self, gl : &gl::WebGl2RenderingContext, use_emission : bool )
    {
      self.bind_multisample( gl );
      self.bind_resolved( gl );

      // Ensure the multisample framebuffer is bound for reading and
      // the resolved framebuffer for drawing.
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
      // Clear the color buffer of the resolved framebuffer before blitting.
      // This is good practice to ensure clean results, especially if partial updates
      // are not intended.
      //gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, &[ 0.0, 0.0, 0.0, 1.0 ] );
      // Blit the multisampled color buffer (COLOR_ATTACHMENT0) from the read framebuffer
      // to the draw framebuffer. The `gl::LINEAR` filter ensures a smooth resolution.
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
        //gl.clear_bufferfv_with_f32_array( gl::COLOR, 1, &[ 0.0, 0.0, 0.0, 0.0 ] );
        gl.blit_framebuffer
        (
          0, 0, self.texture_width as i32, self.texture_height as i32,
          0, 0, self.texture_width as i32, self.texture_height as i32,
          gl::COLOR_BUFFER_BIT,
          gl::LINEAR
        );
      }

      gl.read_buffer( gl::COLOR_ATTACHMENT2 );
      gl::drawbuffers::drawbuffers( gl, &[ 2 ] );
      //gl.clear_bufferfv_with_f32_array( gl::COLOR, 2, &[ 0.0, 0.0, 0.0, 1.0 ] );
      gl.blit_framebuffer
      (
        0, 0, self.texture_width as i32, self.texture_height as i32,
        0, 0, self.texture_width as i32, self.texture_height as i32,
        gl::COLOR_BUFFER_BIT, gl::LINEAR
      );

      gl.read_buffer( gl::COLOR_ATTACHMENT3 );
      gl::drawbuffers::drawbuffers( gl, &[ 3 ] );
      //gl.clear_bufferfv_with_f32_array( gl::COLOR, 3, &[ 0.0, 0.0, 0.0, 1.0 ] );
      gl.blit_framebuffer
      (
        0, 0, self.texture_width as i32, self.texture_height as i32,
        0, 0, self.texture_width as i32, self.texture_height as i32,
        gl::COLOR_BUFFER_BIT, gl::LINEAR
      );

      // Unbind framebuffers to reset global state.
      gl.bind_framebuffer( gl::READ_FRAMEBUFFER, None );
      gl.bind_framebuffer( gl::DRAW_FRAMEBUFFER, None );
      self.unbind_multisample( gl );
      self.unbind_resolved( gl );
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
    pub fn bind_multisample( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.viewport( 0, 0, self.texture_width as i32, self.texture_height as i32 );
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, self.multisample_main_renderbuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::RENDERBUFFER, self.multisample_emission_renderbuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, gl::RENDERBUFFER, self.multisample_transparent_accumulate_renderbuffer.as_ref() );
      gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT3, gl::RENDERBUFFER, self.multisample_transparent_revealage_renderbuffer.as_ref() );
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
    pub fn bind_resolved( &self, gl : &gl::WebGl2RenderingContext )
    {
      gl.viewport( 0, 0, self.texture_width as i32, self.texture_height as i32 );
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.resolved_framebuffer.as_ref() );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, self.main_texture.as_ref(), 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, self.emission_texture.as_ref(), 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, gl::TEXTURE_2D, self.transparent_accumulate_texture.as_ref(), 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT3, gl::TEXTURE_2D, self.transparent_revealage_texture.as_ref(), 0 );
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
    pub fn unbind_multisample( &self, gl : &gl::WebGl2RenderingContext )
    {
      // gl.bind_framebuffer( gl::FRAMEBUFFER, self.multisample_framebuffer.as_ref() );
      // gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, None );
      // gl.framebuffer_renderbuffer( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::RENDERBUFFER, None );
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
    pub fn unbind_resolved( &self, gl : &gl::WebGl2RenderingContext )
    {
      //  gl.bind_framebuffer( gl::FRAMEBUFFER, self.resolved_framebuffer.as_ref() );
      // gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, None, 0 );
      // gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, None, 0 );
      gl.bind_framebuffer( gl::FRAMEBUFFER, None );
    }
  }

  /// The source code for the main vertex shader.
  const MAIN_VERTEX_SHADER : &'static str = include_str!( "shaders/main.vert" );
  /// The source code for the main fragment shader.
  const MAIN_FRAGMENT_SHADER : &'static str = include_str!( "shaders/main.frag" );

  /// Manages the rendering process, including program management, IBL setup, and drawing objects in the scene.
  pub struct Renderer
  {
    /// A map of compiled WebGL programs, keyed by a combination of the material ID and vertex shader defines.
    programs : HashMap< String, ProgramInfo< program::PBRShader > >,
    /// Holds the precomputed textures used for Image-Based Lighting.
    ibl : Option< IBL >,
    /// A list of nodes with transparent primitives, sorted by distance to the camera for correct rendering order.
    transparent_nodes : Vec< ( Rc< RefCell< Node > >, Rc< RefCell< Primitive > > ) >,
    /// If set to true, the renderer will add blur to the original image
    use_emission : bool,
    /// The **framebuffer context** used for multisampling and post-processing. This
    /// manages the multisample framebuffer, resolved framebuffer, and their associated
    /// renderbuffers and textures (main color and emission). It allows for anti-aliasing
    /// and the separation of main scene rendering from emissive components.
    framebuffer_ctx : FramebufferContext,
    /// Bloom pass for the emissive texutre
    bloom_effect : UnrealBloomPass,
    /// Blend pass to combined the blurred emissive texture with the main image
    blend_effect : BlendPass,
    /// Swap buffer to control rendering of the effects
    swap_buffer : SwapFramebuffer,
    exposure : f32,
    composite_shader : ProgramInfo< CompositeShader >
  }

  impl Renderer
  {
    /// Creates a new `Renderer` instance with default settings.
    pub fn new( gl : &gl::WebGl2RenderingContext, width : u32, height : u32, samples : i32 ) -> Result< Self, gl::WebglError >
    {
      let framebuffer_ctx = FramebufferContext::new( gl, width, height, samples );
      let use_emission = false;
      let programs = HashMap::new();
      let ibl = None;
      let transparent_nodes = Vec::new();
      let bloom_effect = UnrealBloomPass::new( gl, width, height, gl::RGBA16F )?;
      let mut blend_effect = BlendPass::new( gl )?;
      blend_effect.dst_factor = gl::ONE;
      blend_effect.src_factor = gl::ONE;
      blend_effect.blend_texture = framebuffer_ctx.main_texture.clone();
      let swap_buffer = SwapFramebuffer::new( gl, width, height );
      let exposure = 0.0;

      let composite_program = gl::ProgramFromSources::new( VS_TRIANGLE, include_str!( "shaders/composite.frag" ) ).compile_and_link( gl )?;
      let composite_shader = ProgramInfo::< CompositeShader >::new( gl, composite_program );
      let locations = composite_shader.get_locations();
      composite_shader.bind( gl );
      gl.uniform1i( locations.get( "transparentA" ).unwrap().clone().as_ref() , 0 );
      gl.uniform1i( locations.get( "transparentB" ).unwrap().clone().as_ref() , 1 );

      Ok
      (
        Self
        {
          programs,
          ibl,
          transparent_nodes,
          use_emission,
          framebuffer_ctx,
          blend_effect,
          bloom_effect,
          swap_buffer,
          exposure,
          composite_shader
        }
      )
    }

    /// Sets the Image-Based Lighting (IBL) textures to be used for rendering.
    ///
    /// * `ibl`: The `IBL` struct containing the diffuse and specular environment maps and the BRDF integration texture.
    pub fn set_ibl( &mut self, ibl : IBL )
    {
      self.ibl = Some( ibl );
    }

    /// Sets whether the renderer should use the emission texture for post-processing effects.
    pub fn set_use_emission( &mut self, use_emission : bool )
    {
      self.use_emission = use_emission;
    }

    /// Returns the current exposure value.
    pub fn get_exposure( &self ) -> f32
    {
      self.exposure
    }

    /// Sets a new exposure value.
    pub fn set_exposure( &mut self, exposure : f32 )
    {
      self.exposure = exposure;
    }

    /// Sets the radius of the bloom effect.
    ///
    /// This determines how far the light "bleeds" from bright areas. A larger radius
    /// results in a more expansive and softer glow.
    pub fn set_bloom_radius( &mut self, radius : f32 )
    {
      self.bloom_effect.set_bloom_radius( radius );
    }

    /// Gets the radius of the bloom effect.
    pub fn get_bloom_radius( &self ) -> f32
    {
      self.bloom_effect.get_bloom_radius()
    }

    /// Sets the strength (intensity) of the bloom effect.
    ///
    /// This controls how bright or prominent the glow appears. A higher strength
    /// makes the bloom more visible.
    pub fn set_bloom_strength( &mut self, strength : f32  )
    {
      self.bloom_effect.set_bloom_strength( strength );
    }

    /// Gets the strength (intensity) of the bloom effect.
    pub fn get_bloom_strength( &self  ) -> f32
    {
      self.bloom_effect.get_bloom_strength()
    }

    /// Retrieves a clone of the main color texture from the internal framebuffer context.
    pub fn get_main_texture( &self ) -> Option< gl::web_sys::WebGlTexture >
    {
      self.framebuffer_ctx.main_texture.clone()
    }

    /// Renders the scene using the provided camera.
    ///
    /// * `gl`: The `WebGl2RenderingContext` to use for rendering.
    /// * `scene`: A mutable reference to the `Scene` to be rendered.
    /// * `camera`: A reference to the `Camera` defining the viewpoint.
    pub fn render
    (
      &mut self,
      gl : &gl::WebGl2RenderingContext,
      scene : &mut Scene,
      camera : &Camera
    ) -> Result< (), gl::WebglError >
    {
      scene.update_world_matrix();

      gl.enable( gl::DEPTH_TEST );
      gl.disable( gl::CULL_FACE );
      gl.disable( gl::BLEND );
      gl.depth_mask( true );
      gl.clear_depth( 1.0 );
      gl.clear_stencil( 0 );
      gl.front_face( gl::CCW );

      if self.use_emission
      {
        self.framebuffer_ctx.enable_emission_texture( gl );
      }
      else
      {
        self.framebuffer_ctx.disable_emission_texture( gl );
      }

      self.framebuffer_ctx.bind_multisample( gl );
      gl::drawbuffers::drawbuffers( gl, &[ 0, 1, 2, 3 ] );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, &[ 0.0, 0.0, 0.0, 1.0 ] );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 1, &[ 0.0, 0.0, 0.0, 0.0 ] );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 2, &[ 0.0, 0.0, 0.0, 1.0 ] );
      gl.clear_bufferfv_with_f32_array( gl::COLOR, 3, &[ 0.0, 0.0, 0.0, 1.0 ] );
      gl.clear( gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT );
      gl::drawbuffers::drawbuffers( gl, &[ 0, 1 ] );


      // Clear the list of transparent nodes before each render.
      self.transparent_nodes.clear();

      for program in self.programs.values()
      {
        let locations = program.get_locations();
        program.bind( gl );
        camera.upload( gl, locations );
        gl::uniform::upload( gl, locations.get( "exposure" ).unwrap().clone(), &self.exposure )?;
      }

      // Define a closure to handle the drawing of each node in the scene.
      let mut draw_node =
      |
        node : Rc< RefCell< Node > >
      | -> Result< (), gl::WebglError >
      {
        // If the node contains a mesh...
        if let Object3D::Mesh( ref mesh ) = node.borrow().object
        {
          // Iterate over each primitive in the mesh.
          for primitive_rc in mesh.borrow().primitives.iter()
          {
            let primitive = primitive_rc.borrow();
            let material = primitive.material.borrow();
            let geometry = primitive.geometry.borrow();
            let vs_defines = geometry.get_defines();
            // Generate a unique ID for the program based on the material ID and vertex shader defines.
            let program_id = format!( "{}{}", material.id, vs_defines );

            // Retrieve the program info if it already exists, otherwise compile and link a new program.
            let program_info =
            if let Some( ref program_info ) = self.programs.get( &program_id )
            {
             program_info
            }
            else
            {
              let ibl_define = if self.ibl.is_some()
              {
                "#define USE_IBL\n"
              }
              else
              {
                ""
              };


              // Compile and link a new WebGL program from the vertex and fragment shaders with the appropriate defines.
              let program = gl::ProgramFromSources::new
              (
                &format!( "#version 300 es\n{}\n{}", vs_defines, MAIN_VERTEX_SHADER ),
                &format!
                (
                  "#version 300 es\n{}\n{}\n{}\n{}",
                  vs_defines,
                  ibl_define,
                  material.get_defines(),
                  MAIN_FRAGMENT_SHADER )
              ).compile_and_link( gl )?;
              let program_info = ProgramInfo::< program::PBRShader >::new( gl , program );

              // Configure and upload material properties and IBL textures for the new program.
              let locations = program_info.get_locations();
              program_info.bind( gl );
              const IBL_BASE_ACTIVE_TEXTURE : u32 = 10;
              material.configure( gl, locations, IBL_BASE_ACTIVE_TEXTURE );
              material.upload( gl, locations )?;
              camera.upload( gl, locations );
              if let Some( ref ibl ) = self.ibl
              {
                ibl.bind( gl, IBL_BASE_ACTIVE_TEXTURE );
              }

              // Store the new program info in the cache.
              self.programs.insert( program_id.clone(), program_info );
              self.programs.get( &program_id ).unwrap()
            };

            // Handle transparent objects by adding them to a separate list for later rendering.
            match material.alpha_mode
            {
              AlphaMode::Blend | AlphaMode::Mask =>
              {
                self.transparent_nodes.push( ( node.clone(), primitive_rc.clone() ) );
                continue; // Skip the immediate drawing of transparent objects.
              },
              _ => {}
            }

            // Get the uniform locations for the current program.
            let locations = program_info.get_locations();

            // Bind the program, upload camera and node matrices, bind the primitive, and draw it.
            program_info.bind( gl );

            node.borrow().upload( gl, locations );
            primitive.bind( gl );
            primitive.draw( gl );
          }
        }

        Ok( () )
      };

      // Traverse the scene and draw all opaque objects.
      scene.traverse( &mut draw_node )?;

      gl::drawbuffers::drawbuffers( gl, &[ 2, 3 ] );
      gl.enable( gl::BLEND );
      gl.depth_mask( false );
      gl.blend_equation( gl::FUNC_ADD );
      gl.blend_func_separate( gl::ONE, gl::ONE, gl::ZERO, gl::ONE_MINUS_SRC_ALPHA );

      let bind = | node : std::cell::Ref< '_, Node >, primitive : std::cell::Ref< '_, Primitive > | -> Result< (), gl::WebglError >
      {
        let primitive = primitive;
        let material = primitive.material.borrow();
        let geometry = primitive.geometry.borrow();
        let vs_defines = geometry.get_defines();
        let program_info = self.programs.get( &format!( "{}{}",  material.id, vs_defines ) ).unwrap();

        let locations = program_info.get_locations();

        program_info.bind( gl );

        node.upload( gl, locations );
        primitive.bind( gl );

       Ok( () )
      };

      // Render the transparent nodes.
      for ( node, primitive ) in self.transparent_nodes.iter()
      {
        let primitive = primitive.borrow();
        let node = node.borrow();
        bind( node, std::cell::Ref::clone( &primitive ) )?;
        primitive.draw( gl );
      }

      self.framebuffer_ctx.resolve( gl, self.use_emission );
      self.framebuffer_ctx.unbind_multisample( gl );


      // self.transparent_nodes.sort_by( | a, b |
      // {
      //   let dist1 = a.0.borrow().center().distance_squared( &camera.get_eye() );
      //   let dist2 = b.0.borrow().center().distance_squared( &camera.get_eye() );

      //   dist1.partial_cmp( &dist2 ).unwrap()
      // });


      // gl.enable( gl::BLEND );
      // gl.depth_mask( false );
      // gl.depth_func( gl::LEQUAL );
      // gl.enable( gl::CULL_FACE );
      // gl.blend_equation( gl::FUNC_ADD );
      // gl.blend_func( gl::ONE, gl::ONE_MINUS_SRC_ALPHA );

      // let bind = | node : &std::cell::Ref< '_, Node >, primitive : &std::cell::Ref< '_, Primitive > | -> Result< (), gl::WebglError >
      // {
      //   let primitive = primitive;
      //   let material = primitive.material.borrow();
      //   let geometry = primitive.geometry.borrow();
      //   let vs_defines = geometry.get_defines();
      //   let program_info = self.programs.get( &format!( "{}{}",  material.id, vs_defines ) ).unwrap();

      //   let locations = program_info.get_locations();

      //   program_info.bind( gl );

      //   node.upload( gl, locations );
      //   primitive.bind( gl );

      //  Ok( () )
      // };

      // // Render the transparent nodes.
      // for ( node, primitive ) in self.transparent_nodes.iter()
      // {
      //   let primitive = primitive.borrow();
      //   let node = node.borrow();
      //   bind( &node, &primitive )?;

      //   gl.cull_face( gl::FRONT );
      //   primitive.draw( gl );

      //   gl.cull_face( gl::BACK );
      //   primitive.draw( gl );
      // }


      // gl.disable( gl::CULL_FACE );

      // self.framebuffer_ctx.resolve( gl, self.use_emission );
      // self.framebuffer_ctx.unbind_multisample( gl );

      if self.use_emission
      {
        self.swap_buffer.reset();
        self.swap_buffer.bind( gl );
        self.swap_buffer.set_input( self.framebuffer_ctx.emission_texture.clone() );

        self.bloom_effect.render( gl, self.swap_buffer.get_input(), self.swap_buffer.get_output() )?;
        self.blend_effect.blend_texture = self.swap_buffer.get_output();
        self.blend_effect.render( gl, None, self.framebuffer_ctx.main_texture.clone() )?;
      }

      self.composite_shader.bind( gl );
      gl.bind_framebuffer( gl::FRAMEBUFFER, self.framebuffer_ctx.resolved_framebuffer.as_ref() );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, None, 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, None, 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, gl::TEXTURE_2D, None, 0 );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT3, gl::TEXTURE_2D, None, 0 );
      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, self.framebuffer_ctx.transparent_accumulate_texture.as_ref() );
      gl.active_texture( gl::TEXTURE1 );
      gl.bind_texture( gl::TEXTURE_2D, self.framebuffer_ctx.transparent_revealage_texture.as_ref() );


      gl::drawbuffers::drawbuffers( gl, &[ 0 ] );

      gl.blend_func( gl::ONE, gl::ONE_MINUS_SRC_ALPHA );
      gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, self.framebuffer_ctx.main_texture.as_ref(), 0 );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      Ok( () )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Renderer
  };
}
