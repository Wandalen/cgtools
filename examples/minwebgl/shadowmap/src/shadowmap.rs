use minwebgl as gl;
use gl::{ GL, Program };
use web_sys::{ WebGlFramebuffer, WebGlTexture };

pub struct Shadowmap
{
  framebuffer   : Option< WebGlFramebuffer >,
  depth_texture : Option< WebGlTexture >, // Now a color texture storing depth values
  depth_buffer  : Option< WebGlTexture >, // Actual depth buffer for depth testing
  program       : Program,
  resolution    : i32,
  gl            : GL,
}

impl Shadowmap
{
  pub fn new( gl : &GL, resolution : u32 ) -> Result< Self, gl::WebglError >
  {
    let resolution = resolution as i32;
    // Create color texture to store depth values (workaround for Chrome depth texture sampling issues)
    let mip_levels = ( resolution as f32 ).log2().floor() as i32 + 1;

    let depth_texture = gl.create_texture();
    gl.bind_texture( gl::TEXTURE_2D, depth_texture.as_ref() );
    gl.tex_storage_2d( GL::TEXTURE_2D, mip_levels, gl::R16F, resolution, resolution );
    gl::texture::d2::wrap_clamp( gl );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );

    // Create depth renderbuffer for actual depth testing
    let depth_buffer = gl.create_texture();
    gl.bind_texture( gl::TEXTURE_2D, depth_buffer.as_ref() );
    gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::DEPTH_COMPONENT24, resolution, resolution );
    gl::texture::d2::wrap_clamp( gl );
    gl::texture::d2::filter_nearest( gl );

    // Setup framebuffer with both attachments
    let framebuffer = gl.create_framebuffer();
    gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
    gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, depth_texture.as_ref(), 0 );
    gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_buffer.as_ref(), 0 );

    let status = gl.check_framebuffer_status( gl::FRAMEBUFFER );
    if status != gl::FRAMEBUFFER_COMPLETE
    {
      gl::browser::error!( "Framebuffer incomplete: {:?}", status );
    }

    gl.bind_framebuffer( gl::FRAMEBUFFER, None );

    let vertex = include_str!( "shaders/shadowmap.vert" );
    let fragment = include_str!( "shaders/shadowmap.frag" );
    let program = gl::Program::new( gl.clone(), vertex, fragment )?;

    Ok
    (
      Self
      {
        framebuffer,
        depth_texture,
        depth_buffer,
        program,
        resolution,
        gl : gl.clone()
      }
    )
  }

  pub fn bind( &self )
  {
    self.gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    self.gl.viewport( 0, 0, self.resolution, self.resolution );
    self.program.activate();
  }

  pub fn upload_mvp( &self, mvp : gl::F32x4x4 )
  {
    self.program.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
  }

  pub fn depth_texture( &self ) -> Option< &WebGlTexture >
  {
    self.depth_texture.as_ref()
  }

  pub fn depth_buffer( &self ) -> Option< &WebGlTexture >
  {
    self.depth_buffer.as_ref()
  }

  /// Generates mipmaps for the shadow map texture
  ///
  /// Call this after rendering to the shadow map to generate mipmaps for better
  /// filtering quality when sampling the shadow map at different distances.
  pub fn generate_mipmaps( &self )
  {
    self.gl.bind_texture( gl::TEXTURE_2D, self.depth_texture.as_ref() );
    self.gl.generate_mipmap( gl::TEXTURE_2D );
  }

  pub fn clear( &self )
  {
    self.gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
    self.gl.clear_color( 1.0, 1.0, 1.0, 1.0 );
    self.gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );
  }
}

/// Shadow baker for pre-computing PCSS shadows into a lightmap texture
///
/// This struct manages the baking process where geometry is rendered with UV-based positioning
/// so that each triangle rasterizes to its corresponding lightmap region. The fragment shader
/// calculates high-quality PCSS shadows and outputs shadow values (0 = lit, 1 = shadowed).
///
/// Unlike the Shadowmap struct which owns its texture, ShadowBaker uses external textures
/// that you provide via `set_target()`, allowing flexible reuse for multiple lightmaps.
pub struct ShadowRenderer
{
  framebuffer : Option< WebGlFramebuffer >,
  program     : Program,
  gl          : GL,
}

impl ShadowRenderer
{
  /// Creates a new shadow baker
  ///
  /// # Arguments
  /// * `gl` - WebGL context
  ///
  /// # Returns
  /// Result containing the ShadowBaker or a WebGL error
  pub fn new( gl : &GL ) -> Result< Self, gl::WebglError >
  {
    // Create framebuffer (texture will be attached later via set_target)
    let framebuffer = gl.create_framebuffer();

    // Load shadow baker shaders
    let vertex = include_str!( "shaders/shadow.vert" );
    let fragment = include_str!( "shaders/shadow.frag" );
    let program = gl::Program::new( gl.clone(), vertex, fragment )?;

    Ok
    (
      Self
      {
        framebuffer,
        program,
        gl : gl.clone(),
      }
    )
  }

  /// Sets the target texture to render shadows into
  ///
  /// # Arguments
  /// * `texture` - The lightmap texture to render into
  ///
  /// # Recommended Formats
  /// - **RGBA16F**: Best quality, requires `EXT_color_buffer_float` extension
  /// - **RGBA8**: Good quality, always supported, memory efficient
  /// - **R32F**: High precision but requires `EXT_color_buffer_float` and may not be supported
  ///
  /// # Note
  /// The texture must be created and sized before calling this method.
  /// Shadow values are stored in the R channel (0 = lit, 1 = shadowed).
  pub fn set_target( &self, texture : Option< &WebGlTexture > )
  {
    self.gl.bind_framebuffer( gl::FRAMEBUFFER, self.framebuffer.as_ref() );
    self.gl.framebuffer_texture_2d
    (
      gl::FRAMEBUFFER,
      gl::COLOR_ATTACHMENT0,
      gl::TEXTURE_2D,
      texture,
      0
    );

    // Check framebuffer completeness
    let status = self.gl.check_framebuffer_status( gl::FRAMEBUFFER );
    if status != gl::FRAMEBUFFER_COMPLETE
    {
      gl::browser::error!( "Shadow baker framebuffer incomplete: {:?}", status );
    }
  }

  /// Binds the shadow baker framebuffer and activates the shader program
  /// Call this before rendering geometry for baking
  ///
  /// # Arguments
  /// * `resolution` - Width and height of the target texture (for viewport)
  pub fn bind( &self, width : u32, height : u32 )
  {
    self.gl.bind_framebuffer( gl::FRAMEBUFFER, self.framebuffer.as_ref() );
    self.gl.viewport( 0, 0, width as i32, height as i32 );
    self.program.activate();
  }

  /// Uploads the model matrix for the geometry being baked
  pub fn upload_model( &self, model : gl::F32x4x4 )
  {
    self.program.uniform_matrix_upload( "u_model", model.raw_slice(), true );
  }

  pub fn set_shadowmap( &self, shadowmap : Option< &WebGlTexture > )
  {
    self.gl.active_texture( gl::TEXTURE0 );
    self.gl.bind_texture( gl::TEXTURE_2D, shadowmap );
  }

  /// Uploads all light-related data from a LightSource
  ///
  /// This method extracts and uploads:
  /// - Light view-projection matrix
  /// - Light direction (world space) - used for orthographic projections
  /// - Light position (world space) - used for perspective projections
  /// - Orthographic/perspective flag
  /// - Light size (world space) - controls shadow softness/penumbra size
  ///
  /// For perspective projections, the shader calculates per-fragment light direction
  /// based on the light position, simulating a point light source.
  ///
  /// # Arguments
  /// * `light_source` - Reference to the light source (mutable for view_projection caching)
  pub fn upload_light_source( &self, light_source : &mut LightSource )
  {
    // Upload view-projection matrix
    let light_vp = light_source.view_projection();
    self.program.uniform_matrix_upload( "u_light_view_projection", light_vp.raw_slice(), true );

    // Upload light direction (used for orthographic)
    let light_dir = light_source.direction();
    self.program.uniform_upload( "u_light_dir", light_dir.as_slice() );

    // Upload light position (used for perspective)
    let light_pos = light_source.position();
    self.program.uniform_upload( "u_light_position", light_pos.as_slice() );

    // Upload orthographic flag
    let is_ortho = if light_source.is_orthographic() { 1.0f32 } else { 0.0f32 };
    self.program.uniform_upload( "u_is_orthographic", &is_ortho );

    // Upload light size (controls penumbra/shadow softness)
    let light_size = light_source.light_size();
    self.program.uniform_upload( "u_light_size", &light_size );
  }
}

pub struct LightSource
{
  position    : gl::F32x3,
  orientation : gl::QuatF32,
  projection  : gl::F32x4x4,
  light_size  : f32,
  mvp         : Option< gl::F32x4x4 >,
}

impl LightSource
{
  pub fn new
  (
    position : gl::F32x3,
    orientation : gl::QuatF32,
    projection : gl::F32x4x4,
    light_size : f32
  ) -> Self
  {
    Self { position, orientation, projection, light_size, mvp : None,  }
  }

  pub fn light_size( &self ) -> f32
  {
    self.light_size
  }

  pub fn position( &self ) -> gl::F32x3
  {
    self.position
  }

  pub fn orientation( &self ) -> gl::QuatF32
  {
    self.orientation
  }

  pub fn projection( &self ) -> gl::F32x4x4
  {
    self.projection
  }

  /// Determines if this light source uses orthographic projection
  ///
  /// Inspects the projection matrix to detect projection type:
  /// - Orthographic: `matrix[3][3] == 1.0` (no perspective division)
  /// - Perspective: `matrix[3][3] == 0.0` (perspective division occurs)
  pub fn is_orthographic( &self ) -> bool
  {
    // In GL projection matrices:
    // - Orthographic has [3][3] = 1.0 (bottom-right element)
    // - Perspective has [3][3] = 0.0
    let m = self.projection.raw_slice();
    let w_component = m[ 15 ]; // [3][3] in column-major order

    ( w_component - 1.0 ).abs() < 0.01
  }

  /// Returns the light direction in world space (forward vector)
  pub fn direction( &self ) -> gl::F32x3
  {
    let local_forward = gl::F32x3::new( 0.0, 0.0, -1.0 );
    let rotation_matrix = self.orientation.to_matrix();
    rotation_matrix * local_forward
  }

  pub fn view_projection( &mut self ) -> gl::F32x4x4
  {
    if let Some( mvp ) = self.mvp
    {
      mvp
    }
    else
    {
      let view = self.orientation.invert().to_matrix().to_homogenous() * gl::math::mat3x3h::translation( -self.position );
      let view_projection = self.projection * view;
      self.mvp = Some( view_projection );

      view_projection
    }
  }
}
