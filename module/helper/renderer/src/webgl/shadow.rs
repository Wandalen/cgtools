//! Module

mod private
{
  use minwebgl as gl;
  use gl::{ GL, Program, js_sys, JsValue, math::mat3x3h };
  use web_sys::{ WebGlFramebuffer, WebGlTexture };
  use crate::webgl::Node;

  /// Shadow map for rendering depth from light's perspective
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
    /// Creates shadow map with specified resolution
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

      gl::drawbuffers::drawbuffers( gl, &[] );
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

    /// Activates shadow map for depth rendering
    pub fn bind( &self )
    {
      self.gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
      self.program.activate();
      self.gl.enable( gl::DEPTH_TEST );
      self.gl.enable( gl::CULL_FACE );
      self.gl.cull_face( gl::FRONT );
      self.gl.viewport( 0, 0, self.resolution, self.resolution );
    }

    /// Sets model-view-projection matrix
    pub fn upload_mvp( &self, mvp : gl::F32x4x4 )
    {
      self.program.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
    }

    /// Returns depth texture for sampling
    pub fn depth_buffer( &self ) -> Option< &WebGlTexture >
    {
      self.depth_buffer.as_ref()
    }

    /// Clears depth buffer
    pub fn clear( &self )
    {
      self.gl.bind_framebuffer( GL::FRAMEBUFFER, self.framebuffer.as_ref() );
      self.gl.clear( gl::DEPTH_BUFFER_BIT );
    }

    /// Renders shadow map from light's perspective
    pub fn render_shadow_map
    (
      &self,
      scene : &crate::webgl::Scene,
      mut light : Light
    ) -> Result< (), gl::WebglError >
    {
      self.bind();
      self.clear();

      // Recursively traverse scene and render all shadow-casting meshes
      scene.traverse
      (
        &mut | node |
        {
          let node = node.borrow();

          if let crate::webgl::Object3D::Mesh( mesh ) = &node.object
          {
            if !mesh.borrow().is_shadow_caster
            {
              return Ok( () );
            }

            let model = node.get_world_matrix();
            let mvp = light.view_projection() * model;
            self.upload_mvp( mvp );

            for primitive in &mesh.borrow().primitives
            {
              let primitive = primitive.borrow();
              primitive.geometry.borrow().bind( &self.gl );
              primitive.draw( &self.gl );
            }
          }

          Ok( () )
        }
      )?;

      self.gl.bind_framebuffer( gl::FRAMEBUFFER, None );

      Ok( () )
    }
  }

  /// Bakes PCSS shadows into lightmap textures
  #[ derive( Debug ) ]
  pub struct ShadowBaker
  {
    framebuffer : Option< WebGlFramebuffer >,
    program     : Program,
    gl          : GL,
  }

  impl ShadowBaker
  {
    /// Creates shadow baker
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
          gl : gl.clone(),
        }
      )
    }

    /// Sets target lightmap texture and dimensions
    fn set_target( &self, texture : Option< &WebGlTexture > )
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

    /// Activates baker for rendering
    fn bind( &self, width: i32, height : i32 )
    {
      self.program.activate();
      self.gl.bind_framebuffer( gl::FRAMEBUFFER, self.framebuffer.as_ref() );
      self.gl.viewport( 0, 0, width, height );
      self.gl.disable( gl::DEPTH_TEST );
      self.gl.disable( gl::CULL_FACE );
    }

    /// Sets model matrix for geometry
    fn upload_model( &self, model : gl::F32x4x4 )
    {
      self.program.uniform_matrix_upload( "u_model", model.raw_slice(), true );
    }

    /// Binds shadow map for sampling
    fn set_shadowmap( &self, shadowmap : Option< &WebGlTexture > )
    {
      self.gl.active_texture( gl::TEXTURE0 );
      self.gl.bind_texture( gl::TEXTURE_2D, shadowmap );
    }

    /// Uploads light parameters to shader
    fn upload_light( &self, light : &mut Light )
    {
      let light_vp = light.view_projection();
      self.program.uniform_matrix_upload( "u_light_view_projection", light_vp.raw_slice(), true );

      let light_dir = light.direction();
      self.program.uniform_upload( "u_light_dir", light_dir.as_slice() );

      let light_pos = light.position();
      self.program.uniform_upload( "u_light_position", light_pos.as_slice() );

      let is_ortho = light.is_orthographic() as i32;
      self.program.uniform_upload( "u_is_orthographic", &is_ortho );

      let light_size = light.light_size();
      self.program.uniform_upload( "u_light_size", &light_size );

      let ( near, far ) = light.near_far_planes();

      self.program.uniform_upload( "u_near", &near );
      self.program.uniform_upload( "u_far", &far );
    }

    /// Bakes shadows into lightmaps via two-pass rendering: depth map, then PCSS lightmap baking
    pub fn render_soft_shadow_texture
    (
      &self,
      node : &Node,
      texture : Option< &WebGlTexture >,
      width: i32,
      height : i32,
      shadowmap : &ShadowMap,
      mut light : Light,
    ) -> Result< (), gl::WebglError >
    {
      self.bind( width, height );
      self.upload_light( &mut light );
      self.set_shadowmap( shadowmap.depth_buffer() );
      let model = node.get_world_matrix();
      self.upload_model( model );
      self.set_target( texture );

      if let crate::webgl::Object3D::Mesh( mesh ) = &node.object
      {
        for primitive in &mesh.borrow().primitives
        {
          let primitive_ref = primitive.borrow_mut();
          primitive_ref.geometry.borrow().bind( &self.gl );
          primitive_ref.draw( &self.gl );
        }
      }

      Ok( () )
    }
  }

  /// Light source for shadow casting
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Light
  {
    position        : gl::F32x3,
    direction       : gl::F32x3,
    projection      : gl::F32x4x4,
    light_size      : f32,
    view_projection : Option< gl::F32x4x4 >,
  }

  impl Light
  {
    /// Creates light with position, direction, projection, and size
    pub fn new
    (
      position : gl::F32x3,
      direction : gl::F32x3,
      projection : gl::F32x4x4,
      light_size : f32
    ) -> Self
    {
      Self
      {
        position,
        direction : direction.normalize(),
        projection,
        light_size,
        view_projection : None,
      }
    }

    /// Returns light size (controls shadow softness)
    pub fn light_size( &self ) -> f32
    {
      self.light_size
    }

    /// Extracts near and far planes from projection matrix
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

    /// Returns light position
    pub fn position( &self ) -> gl::F32x3
    {
      self.position
    }

    /// Returns light direction
    pub fn direction( &self ) -> gl::F32x3
    {
      self.direction
    }

    /// Returns projection matrix
    pub fn projection( &self ) -> gl::F32x4x4
    {
      self.projection
    }

    /// Returns true if using orthographic projection (checks matrix[3][3] == 1.0)
    pub fn is_orthographic( &self ) -> bool
    {
      let m = self.projection.raw_slice();
      let w_component = m[ 15 ]; // [3][3] in column-major order
      ( w_component - 1.0 ).abs() < 0.01
    }

    /// Returns cached view-projection matrix
    pub fn view_projection( &mut self ) -> gl::F32x4x4
    {
      if let Some( mvp ) = self.view_projection
      {
        mvp
      }
      else
      {
        let view = mat3x3h::look_to_rh( self.position(), self.direction, gl::F32x3::Y );
        let view_projection = self.projection * view;
        self.view_projection = Some( view_projection );

        view_projection
      }
    }
  }

  impl From< crate::webgl::SpotLight > for Light
  {
    fn from( spot : crate::webgl::SpotLight ) -> Self
    {
      // Use outer cone angle for FOV
      let fov = spot.outer_cone_angle * 2.0;
      let near = 0.1;
      let far = spot.range;

      // Light size affects shadow softness - derive from cone angle
      // Smaller angles = tighter beam = smaller physical size
      let radius = spot.outer_cone_angle * 2.0;
      let max_radius = 135.0_f32.to_radians();

      let light_size = ( ( radius / max_radius ).min( 1.0 ) * 1.7 ).min( 0.01 );

      let projection = gl::math::mat3x3h::perspective_rh_gl( fov, 1.0, near, far );

      Self::new( spot.position, spot.direction, projection, light_size )
    }
  }
}

crate::mod_interface!
{
  own use
  {
    ShadowBaker,
    ShadowMap,
    Light,
  };
}
