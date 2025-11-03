mod private
{
  use minwebgl as gl;
  use gl::{ GL, Program, js_sys, JsValue };
  use web_sys::{ WebGlFramebuffer, WebGlTexture };
  use std::rc::Rc;
  use core::cell::RefCell;

  #[ derive( Debug ) ]
  pub struct ShadowMap
  {
    framebuffer   : Option< WebGlFramebuffer >,
    depth_buffer  : Option< WebGlTexture >,
    program       : Program,
    resolution    : i32,
    gl            : GL,
  }

  impl ShadowMap
  {
    pub fn new( gl : &GL, resolution : u32 ) -> Result< Self, gl::WebglError >
    {
      let resolution = resolution as i32;

      let depth_buffer = gl.create_texture();
      gl.bind_texture( gl::TEXTURE_2D, depth_buffer.as_ref() );
      gl.tex_storage_2d( gl::TEXTURE_2D, 1, gl::DEPTH_COMPONENT24, resolution, resolution );
      gl::texture::d2::filter_nearest( gl );
      gl::texture::d2::wrap_clamp( gl );

      let framebuffer = gl.create_framebuffer();
      gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER,
        gl::DEPTH_ATTACHMENT,
        gl::TEXTURE_2D,
        depth_buffer.as_ref(),
        0
      );

      let arr = js_sys::Array::from_iter( [ JsValue::from_f64( gl::NONE as f64 ) ].into_iter() );
      gl.draw_buffers( &arr );
      gl.read_buffer( gl::NONE );

      let status = gl.check_framebuffer_status( gl::FRAMEBUFFER );
      if status != gl::FRAMEBUFFER_COMPLETE
      {
        gl::browser::error!( "Framebuffer incomplete: {:?}", status );
      }

      gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      let vertex = include_str!( "shaders/depth.vert" );
      let fragment = include_str!( "shaders/depth.frag" );
      let program = gl::Program::new( gl.clone(), vertex, fragment )?;

      Ok
      (
        Self
        {
          framebuffer,
          depth_buffer,
          program,
          resolution,
          gl : gl.clone(),
        }
      )
    }

    pub fn bind( &self )
    {
      self.gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
      self.program.activate();
      self.gl.enable( gl::DEPTH_TEST );
      self.gl.enable( gl::CULL_FACE );
      self.gl.cull_face( gl::FRONT );
      self.gl.viewport( 0, 0, self.resolution, self.resolution );
    }

    pub fn upload_mvp( &self, mvp : gl::F32x4x4 )
    {
      self.program.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
    }

    pub fn depth_buffer( &self ) -> Option< &WebGlTexture >
    {
      self.depth_buffer.as_ref()
    }

    pub fn clear( &self )
    {
      self.gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
      self.gl.clear( gl::DEPTH_BUFFER_BIT );
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
  #[ derive( Debug ) ]
  pub struct ShadowBaker
  {
    framebuffer : Option< WebGlFramebuffer >,
    program     : Program,
    width       : i32,
    height      : i32,
    gl          : GL,
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
      let framebuffer = gl.create_framebuffer();

      let vertex = include_str!( "shaders/bake.vert" );
      let fragment = include_str!( "shaders/bake.frag" );
      let program = gl::Program::new( gl.clone(), vertex, fragment )?;

      Ok
      (
        Self
        {
          framebuffer,
          program,
          width : 0,
          height : 0,
          gl : gl.clone(),
        }
      )
    }

    /// Sets the target texture to render shadows into
    ///
    /// # Arguments
    /// * `texture` - The lightmap texture to render into
    ///
    /// The texture must be created and sized before calling this method.
    /// Shadow values are stored in the R channel (0 = lit, 1 = shadowed).
    pub fn set_target( &mut self, texture : Option< &WebGlTexture >, width : u32, height : u32 )
    {
      self.width = width as i32;
      self.height = height as i32;

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
    pub fn bind( &self )
    {
      self.program.activate();
      self.gl.bind_framebuffer( gl::FRAMEBUFFER, self.framebuffer.as_ref() );
      self.gl.viewport( 0, 0, self.width, self.height );
      self.gl.disable( gl::DEPTH_TEST );
      self.gl.disable( gl::CULL_FACE );
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
    pub fn upload_light( &self, light : &mut Light )
    {
      // Upload view-projection matrix
      let light_vp = light.view_projection();
      self.program.uniform_matrix_upload( "u_light_view_projection", light_vp.raw_slice(), true );

      // Upload light direction (used for orthographic)
      let light_dir = light.direction();
      self.program.uniform_upload( "u_light_dir", light_dir.as_slice() );

      // Upload light position (used for perspective)
      let light_pos = light.position();
      self.program.uniform_upload( "u_light_position", light_pos.as_slice() );

      // Upload orthographic flag
      let is_ortho = if light.is_orthographic() { 1.0f32 } else { 0.0f32 };
      self.program.uniform_upload( "u_is_orthographic", &is_ortho );

      // Upload light size (controls penumbra/shadow softness)
      let light_size = light.light_size();
      self.program.uniform_upload( "u_light_size", &light_size );

      // Upload near and far planes for depth linearization
      let ( near, far ) = light.near_far_planes();

      self.program.uniform_upload( "u_near", &near );
      self.program.uniform_upload( "u_far", &far );
    }
  }

  #[ derive( Debug, Clone, Copy ) ]
  pub struct Light
  {
    position        : gl::F32x3,
    orientation     : gl::QuatF32,
    projection      : gl::F32x4x4,
    light_size      : f32,
    view_projection : Option< gl::F32x4x4 >,
  }

  impl Light
  {
    pub fn new
    (
      position : gl::F32x3,
      orientation : gl::QuatF32,
      projection : gl::F32x4x4,
      light_size : f32
    ) -> Self
    {
      Self
      {
        position,
        orientation,
        projection,
        light_size,
        view_projection : None,
      }
    }

    pub fn light_size( &self ) -> f32
    {
      self.light_size
    }

    pub fn near_far_planes( &self ) -> ( f32, f32 )
    {
      let m = self.projection.raw_slice();
      let m10 = m[ 10 ];  // [2][2] in column-major
      let m14 = m[ 14 ];  // [3][2] in column-major

      if self.is_orthographic()
      {
        // Orthographic projection: m[15] = 1.0
        // m[10] = -2 / (far - near)
        // m[14] = -(far + near) / (far - near)
        // Solving:
        //   far = (m[14] - 1) / m[10]
        //   near = (1 + m[14]) / m[10]
        let far = ( m14 - 1.0 ) / m10;
        let near = ( 1.0 + m14 ) / m10;
        ( near, far )
      }
      else
      {
        // Perspective projection: m[15] = 0.0
        // m[10] = -(far + near) / (far - near)
        // m[14] = -2 * far * near / (far - near)
        // Solving:
        //   near = m[14] / (m[10] - 1)
        //   far = m[14] / (m[10] + 1)
        let near = m14 / ( m10 - 1.0 );
        let far = m14 / ( m10 + 1.0 );
        ( near, far )
      }
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
      if let Some( mvp ) = self.view_projection
      {
        mvp
      }
      else
      {
        let view = self.orientation.invert().to_matrix().to_homogenous()
          * gl::math::mat3x3h::translation( -self.position );
        let view_projection = self.projection * view;
        self.view_projection = Some( view_projection );

        view_projection
      }
    }
  }

  /// Bakes shadows from a light source into lightmaps for all shadow-receiving meshes in the scene.
  ///
  /// This function performs a two-pass shadow baking process:
  /// 1. **Shadow Map Pass**: Recursively traverses all shadow-casting nodes and renders them to a depth buffer
  /// 2. **Lightmap Baking Pass**: Recursively traverses all shadow-receiving nodes and bakes PCSS soft shadows into their lightmaps
  ///
  /// The function automatically handles scene graph hierarchies, processing all descendants of root nodes.
  ///
  /// # Arguments
  /// * `gl` - WebGL context
  /// * `scene` - The scene containing nodes to process
  /// * `light` - The light source casting shadows (mutable for view-projection caching)
  /// * `lightmap_res` - Resolution of lightmap textures (e.g., 2048, 4096, 8192)
  /// * `shadowmap_res` - Resolution of the shadow map depth buffer
  ///
  /// # Returns
  /// Result indicating success or WebGL error
  pub fn bake_shadows
  (
    gl : &GL,
    scene : &crate::webgl::Scene,
    light : &mut Light,
    lightmap_res : u32,
    shadowmap_res : u32
  ) -> Result< (), gl::WebglError >
  {
    // Create shadow map and render all shadow casters from light's perspective
    let shadow_map = ShadowMap::new( gl, shadowmap_res )?;
    shadow_map.bind();
    shadow_map.clear();

    let view_projection = light.view_projection();

    // Recursively traverse scene and render all shadow-casting meshes
    scene.traverse( &mut | node |
    {
      let node = node.borrow();

      if !node.is_shadow_caster
      {
        return Ok( () );
      }

      if let crate::webgl::Object3D::Mesh( mesh ) = &node.object
      {
        let model = node.get_world_matrix();
        let mvp = view_projection * model;
        shadow_map.upload_mvp( mvp );

        for primitive in &mesh.borrow().primitives
        {
          let primitive = primitive.borrow();
          primitive.geometry.borrow().bind( gl );
          primitive.draw( gl );
        }
      }

      Ok( () )
    } )?;

    gl.bind_framebuffer( gl::FRAMEBUFFER, None );

    let mip_levels = ( lightmap_res as f32 ).log2().floor() as i32 + 1;

    let mut shadow_baker = ShadowBaker::new( gl )?;
    shadow_baker.bind();
    shadow_baker.upload_light( light );
    shadow_baker.set_shadowmap( shadow_map.depth_buffer() );
    gl.active_texture( gl::TEXTURE1 );

    scene.traverse( &mut | node |
    {
      let node = node.borrow_mut();

      if !node.is_shadow_receiver
      {
        return Ok( () );
      }

      if let crate::webgl::Object3D::Mesh( mesh ) = &node.object
      {
        let model = node.get_world_matrix();
        shadow_baker.upload_model( model );

        for primitive in &mesh.borrow().primitives
        {
          let light_map = create_texture(gl, lightmap_res, mip_levels);

          shadow_baker.set_target( light_map.as_ref(), lightmap_res, lightmap_res );
          shadow_baker.bind();

          let primitive_ref = primitive.borrow_mut();
          primitive_ref.geometry.borrow().bind( gl );
          primitive_ref.draw( gl );

          gl.bind_texture( gl::TEXTURE_2D, light_map.as_ref() );
          gl.generate_mipmap( gl::TEXTURE_2D );

          let texture_info = crate::webgl::TextureInfo
          {
            texture : Rc::new( RefCell::new( crate::webgl::Texture
            {
              target : gl::TEXTURE_2D,
              source : light_map,
              sampler : crate::webgl::Sampler::default(),
            } ) ),
            uv_position : 0,
          };
          primitive_ref.material.borrow_mut().light_map = Some( texture_info );
        }
      }

      Ok( () )
    } )?;

    Ok( () )
  }

  fn create_texture( gl : &GL, lightmap_res : u32, mip_levels : i32 ) -> Option< WebGlTexture >
  {
    let light_map = gl.create_texture();
    gl.bind_texture( gl::TEXTURE_2D, light_map.as_ref() );
    gl.tex_storage_2d( gl::TEXTURE_2D, mip_levels, gl::R16F, lightmap_res as i32, lightmap_res as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
    gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
    gl::texture::d2::wrap_clamp( &gl );
    light_map
  }
}


crate::mod_interface!
{
  own use
  {
    ShadowBaker,
    ShadowMap,
    Light,
    bake_shadows
  };
}
