use minwebgl as gl;
use gl::{ GL, Program };
use web_sys::{ WebGlFramebuffer, WebGlTexture };

pub struct Shadowmap
{
  framebuffer : Option< WebGlFramebuffer >,
  depth_texture : Option< WebGlTexture >,
  program : Program,
  resolution : i32,
  gl : GL,
}

impl Shadowmap
{
  pub fn new( gl : &GL, resolution : u32 ) -> Result< Self, gl::WebglError >
  {
    let depth_texture = gl.create_texture();
    gl.bind_texture( gl::TEXTURE_2D, depth_texture.as_ref() );
    gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::DEPTH_COMPONENT24, resolution as i32, resolution as i32 );
    gl::texture::d2::wrap_clamp( gl );
    gl::texture::d2::filter_nearest( gl );

    let framebuffer = gl.create_framebuffer();
    gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
    gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_texture.as_ref(), 0 );
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
        program,
        resolution : resolution as i32,
        gl : gl.clone()
      }
    )
  }

  pub fn bind( &self )
  {
    self.gl.viewport( 0, 0, self.resolution, self.resolution );
    self.gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
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
}

pub struct LightSource
{
  position : gl::F32x3,
  orientation : gl::QuatF32,
  projection : gl::F32x4x4,
  mvp : Option< gl::F32x4x4 >,
}

impl LightSource
{
  pub fn new
  (
    position : gl::F32x3,
    orientation : gl::QuatF32,
    projection : gl::F32x4x4,
  ) -> Self
  {
    Self { position, orientation, projection, mvp : None }
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

  pub fn set_position( &mut self, position : gl::F32x3 )
  {
    self.position = position;
    self.mvp = None;
  }

  pub fn set_orientation( &mut self, orientation : gl::QuatF32 )
  {
    self.orientation = orientation;
    self.mvp = None;
  }

  pub fn set_projection( &mut self, projection : gl::F32x4x4 )
  {
    self.projection = projection;
    self.mvp = None;
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

/// Shadow baker for pre-computing PCSS shadows into a lightmap texture
///
/// This struct manages the baking process where geometry is rendered with UV-based positioning
/// so that each triangle rasterizes to its corresponding lightmap region. The fragment shader
/// calculates high-quality PCSS shadows and outputs shadow values (0 = lit, 1 = shadowed).
///
/// Unlike the Shadowmap struct which owns its texture, ShadowBaker uses external textures
/// that you provide via `set_target()`, allowing flexible reuse for multiple lightmaps.
pub struct ShadowBaker
{
  framebuffer : Option< WebGlFramebuffer >,
  program : Program,
  gl : GL,
}

impl ShadowBaker
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
    let vertex = include_str!( "shaders/shadow_baker.vert" );
    let fragment = include_str!( "shaders/shadow_baker.frag" );
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

    self.gl.bind_framebuffer( gl::FRAMEBUFFER, None );
  }

  /// Binds the shadow baker framebuffer and activates the shader program
  /// Call this before rendering geometry for baking
  ///
  /// # Arguments
  /// * `resolution` - Width and height of the target texture (for viewport)
  pub fn bind( &self, resolution : u32 )
  {
    self.gl.bind_framebuffer( gl::FRAMEBUFFER, self.framebuffer.as_ref() );
    self.gl.viewport( 0, 0, resolution as i32, resolution as i32 );
    self.program.activate();
  }

  /// Uploads the model matrix for the geometry being baked
  pub fn upload_model( &self, model : gl::F32x4x4 )
  {
    self.program.uniform_matrix_upload( "u_model", model.raw_slice(), true );
  }

  /// Uploads all light-related data from a LightSource
  ///
  /// This method extracts and uploads:
  /// - Light view-projection matrix
  /// - Light direction (world space) - used for orthographic projections
  /// - Light position (world space) - used for perspective projections
  /// - Orthographic/perspective flag
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
  }

  /// Sets the shadow map texture for shadow calculation
  /// Call this after binding to specify which texture unit the shadow map is on
  pub fn set_shadow_map_unit( &self, texture_unit : i32 )
  {
    self.program.uniform_upload( "u_shadow_map", &texture_unit );
  }
}
