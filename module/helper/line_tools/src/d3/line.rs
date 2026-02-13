mod private
{
  use crate::*;
  use minwebgl as gl;
  use std::collections::VecDeque;

  /// Encapsulates geometry related state of the line
  #[ derive( Debug, Clone, Default ) ]
  pub struct LineGeometry
  {
    /// The series of 3D points that define the line strip.
    pub points : VecDeque< gl::F32x3 >,
    /// Colors for the points
    pub colors : VecDeque< gl::F32x3 >,
    /// The distance from the beginning of the line to the current point
    #[ cfg( feature = "distance" ) ]
    pub distances : VecDeque< f32 >,
    /// Total length of the line
    #[ cfg( feature = "distance" ) ]
    pub total_distance : f32,
  }

  /// Encapsulates render related state of the line
  #[ derive( Debug, Clone, Default ) ]
  pub struct LineRenderState
  {
    // The optional `Mesh` object that holds the WebGL resources for rendering.
    /// `None` until `create_mesh` is called.
    pub mesh : Option< Mesh >,
    /// A flag to set whether to use the vertex color or not. Should be set before the mesh creation
    pub use_vertex_color : bool,
    /// A flag to set where to use alpha to coverage blending technique instead of alpha testing 
    pub use_alpha_to_coverage : bool,
    /// A flag to set where to use width in world units, or screen space units
    pub use_world_units : bool,
    /// Fragment shader source
    pub fragment_shader : String,
    /// Specifies the pattern of the dash, if `use_dash`` is `true``
    #[ cfg( feature = "distance" ) ]
    pub dash_pattern : DashPattern,
    #[ cfg( feature = "distance" ) ]
    /// A flag to set whether to use dashed line or not
    pub use_dash : bool,
  }

  impl LineRenderState
  {
    /// Return shader defines to use during shader compilation
    fn get_defines( &self ) -> String
    {
      let mut s = String::new();
      
      if self.use_vertex_color
      {
        s += "#define USE_VERTEX_COLORS\n";
      }

      if self.use_alpha_to_coverage
      {
        s += "#define USE_ALPHA_TO_COVERAGE\n";
      }

      if self.use_world_units
      {
        s += "#define USE_WORLD_UNITS\n";
      }

      #[ cfg( feature = "distance" ) ]
      if self.use_dash
      {
        s += "#define USE_DASH\n";
        
        match self.dash_pattern
        {
          DashPattern::V1( _ ) => s += "#define USE_DASH_V1\n",
          DashPattern::V2( _ ) => s += "#define USE_DASH_V2\n",
          DashPattern::V3( _ ) => s += "#define USE_DASH_V3\n",
          DashPattern::V4( _ ) => s += "#define USE_DASH_V4\n",
        }

      }

      s
    }
  }

  #[ derive( Debug, Clone, Copy ) ]
  pub enum DashPattern
  {
    V1( f32 ),
    V2( [ f32; 2 ] ),
    V3( [ f32; 3 ] ),
    V4( [ f32; 4 ] )
  }

  impl Default for DashPattern
  {
    fn default() -> Self 
    {
      Self::V1( 0.5 )   
    }
  }
    
  /// Tracks the state change of the line
  #[ derive( Debug, Clone, Default ) ]
  pub struct LineChangeState
  {
    /// A flag to indicate whether the line's points have changed since the last update.
    pub points_changed : bool,
    /// A flag to indicate the colors have been changed
    pub colors_changed : bool,
    /// A flag to indicate any shader defines have been changed
    pub defines_changed : bool
  }

  /// Represents a 3D line strip, composed of a series of points.
  #[ derive( Debug, Clone, Default ) ]
  pub struct Line
  {
    /// Geometry of the line
    geometry : LineGeometry,
    /// Render state of the line( defines, mesh, alpha-to-coverage)
    render_state : LineRenderState,
    /// Change state of the line
    change_state : LineChangeState
  }

  impl_basic_line!( Line, f32, 3 );
  
  impl Line
  {
    /// Creates the WebGL mesh for the line.
    ///
    /// This function compiles shaders, generates the line's geometry, creates buffers and a VAO,
    /// and initializes the `Mesh` object. It sets up the vertex attributes for instanced drawing,
    /// where each instance is a segment of the line.
    pub fn mesh_create( &mut self, gl : &gl::WebGl2RenderingContext, fragment_shader : Option< &str > ) -> Result< (), gl::WebglError >
    {
      self.render_state.fragment_shader = fragment_shader.unwrap_or( d3::MAIN_FRAGMENT_SHADER ).to_string();

      let ( vertices, indices, uvs ) = helpers::four_piece_rectangle_geometry();

      let points_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to points_buffer" ) )?;
      let position_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to position_buffer" ) )?;
      let index_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to index_buffer" ) )?;
      let uv_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to uv_buffer" ) )?;
      let color_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to color_buffer" ) )?;
      let distances_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to distance_buffer" ) )?;

      gl::buffer::upload( gl, &position_buffer, &vertices.iter().copied().flatten().collect::< Vec< f32 > >(), gl::STATIC_DRAW );
      gl::buffer::upload( gl, &uv_buffer, &uvs.iter().copied().flatten().collect::< Vec< f32 > >(), gl::STATIC_DRAW );
      gl::index::upload( gl, &index_buffer, &indices, gl::STATIC_DRAW );

      let vao = gl.create_vertex_array();
      gl.bind_vertex_array( vao.as_ref() );

      gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 2 ).offset( 0 ).divisor( 0 ).attribute_pointer( gl, 0, &position_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 2 ).offset( 0 ).divisor( 0 ).attribute_pointer( gl, 1, &uv_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).divisor( 1 ).attribute_pointer( gl, 2, &points_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 3 ).divisor( 1 ).attribute_pointer( gl, 3, &points_buffer )?;

      if self.render_state.use_vertex_color
      {
        gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).divisor( 1 ).attribute_pointer( gl, 4, &color_buffer )?;
        gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 3 ).divisor( 1 ).attribute_pointer( gl, 5, &color_buffer )?;
      }

      #[ cfg( feature = "distance" ) ]
      if self.render_state.use_dash
      {
        gl::BufferDescriptor::new::< [ f32; 1 ] >().stride( 1 ).offset( 0 ).divisor( 1 ).attribute_pointer( gl, 6, &distances_buffer )?;
        gl::BufferDescriptor::new::< [ f32; 1 ] >().stride( 1 ).offset( 1 ).divisor( 1 ).attribute_pointer( gl, 7, &distances_buffer )?;
      }

      let program = Program
      {
        vertex_shader : None,
        fragment_shader : None,
        vao : vao,
        program : None,
        draw_mode : gl::TRIANGLES,
        instance_count : Some( ( self.geometry.points.len() as f32 - 1.0 ).max( 0.0 ) as u32 ),
        index_count : Some( indices.len() as u32 ),
        vertex_count : vertices.len() as u32,
        index_buffer : Some( index_buffer ),
        uniforms : UniformStorage::default()
      };

      let mut mesh = Mesh::default();
      mesh.program_add( "body", program );

      mesh.buffer_add( "position", position_buffer );
      mesh.buffer_add( "points", points_buffer );
      mesh.buffer_add( "uv", uv_buffer );
      mesh.buffer_add( "colors", color_buffer );
      mesh.buffer_add( "distances", distances_buffer );

      self.render_state.mesh = Some( mesh );

      self.change_state.points_changed = true;
      self.change_state.colors_changed = true;
      self.change_state.defines_changed = true;

      self.mesh_update( gl )?;

      Ok( () )
    }

    /// Updates the mesh's vertex buffers if the line's points have changed.
    pub fn mesh_update( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      if self.change_state.defines_changed
      {
        let defines = self.get_defines();
        let vertex_shader = d3::MAIN_VERTEX_SHADER.replace( "// #include <defines>", &defines );
        let vertex_shader = gl::ShaderSource::former()
        .shader_type( gl::VERTEX_SHADER )
        .source( &vertex_shader )
        .compile( &gl )?;

        let fragment_shader = self.render_state.fragment_shader.replace( "// #include <defines>", &defines );
        let fragment_shader = gl::ShaderSource::former()
        .shader_type( gl::FRAGMENT_SHADER )
        .source( &fragment_shader )
        .compile( &gl )?;

        let program = gl::ProgramShaders::new( &vertex_shader, &fragment_shader ).link( &gl )?;

        let mesh = self.render_state.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;
        let b_program = mesh.program_get_mut( "body" );

        b_program.fragment_shader_delete( gl );
        b_program.vertex_shader_delete( gl );
        b_program.program_delete( gl );

        b_program.program = Some( program );
        b_program.fragment_shader = Some( fragment_shader );
        b_program.vertex_shader = Some( vertex_shader );

        b_program.uniform_locations_clear();

        if self.render_state.use_dash
        {
          match self.render_state.dash_pattern
          {
            DashPattern::V1( v ) => b_program.upload( gl, "u_dash_pattern", &v )?,
            DashPattern::V2( v ) => b_program.upload( gl, "u_dash_pattern", &v )?,
            DashPattern::V3( v ) => b_program.upload( gl, "u_dash_pattern", &v )?,
            DashPattern::V4( v ) => b_program.upload( gl, "u_dash_pattern", &v )?,
          }
        }

        b_program.all_uniforms_upload( gl )?;

        self.change_state.defines_changed = false;
      }

      if self.change_state.points_changed
      {
        let mesh = self.render_state.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;
        let points_buffer = mesh.buffer_get( "points" );
        
        let points : Vec< f32 > = self.geometry.points.iter().flat_map( | p | p.to_array() ).collect();
        gl::buffer::upload( &gl, &points_buffer, &points, gl::STATIC_DRAW );

        #[ cfg( feature = "distance" ) ]
        {
          let distances_buffer = mesh.buffer_get( "distances" );
          let distances : Vec< f32 > = self.geometry.distances.iter().copied().collect();
          gl::buffer::upload( &gl, &distances_buffer, &distances, gl::STATIC_DRAW );
        }

        let b_program = mesh.program_get_mut( "body" );
        b_program.instance_count = Some( ( self.geometry.points.len() as f32 - 1.0 ).max( 0.0 ) as u32 );

        self.change_state.points_changed = false;
      }

      if self.change_state.colors_changed && self.render_state.use_vertex_color
      {
        let mesh = self.render_state.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;
        let colors_buffer = mesh.buffer_get( "colors" );

        let colors : Vec< f32 > = self.geometry.colors.iter().flat_map( | c | c.to_array() ).collect();
        gl::buffer::upload( &gl, &colors_buffer, &colors, gl::STATIC_DRAW );

        self.change_state.colors_changed = false;
      }

      Ok( () )
    }

    #[ cfg( feature = "distance" ) ]
    pub fn dash_pattern_set( &mut self, value : DashPattern )
    {
      if std::mem::discriminant( &self.render_state.dash_pattern ) != std::mem::discriminant( &value )
      {
        self.change_state.defines_changed = true;
      }
      self.render_state.dash_pattern = value;
    }

    #[ cfg( feature = "distance" ) ]
    /// Sets whether the world units for the line width will be used
    pub fn use_dash( &mut self, value : bool )
    {
      self.render_state.use_dash = value;
      self.change_state.defines_changed = true;
    }

    /// Sets whether the alpha to coverage will be used or not
    pub fn use_alpha_to_coverage( &mut self, value : bool )
    {
      self.render_state.use_alpha_to_coverage = value;
      self.change_state.defines_changed = true;
    }

    /// Sets whether the world units for the line width will be used
    pub fn use_world_units( &mut self, value : bool )
    {
      self.render_state.use_world_units = value;
      self.change_state.defines_changed = true;
    }

    /// Draws the line mesh.
    pub fn draw( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      self.mesh_update( gl )?;

      let mesh = self.render_state.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;

      if self.render_state.use_dash
      {
        let b_program = mesh.program_get_mut( "body" );
        match self.render_state.dash_pattern
        {
          DashPattern::V1( v ) => b_program.upload( gl, "u_dash_pattern", &v )?,
          DashPattern::V2( v ) => b_program.upload( gl, "u_dash_pattern", &v )?,
          DashPattern::V3( v ) => b_program.upload( gl, "u_dash_pattern", &v )?,
          DashPattern::V4( v ) => b_program.upload( gl, "u_dash_pattern", &v )?,
        }
      }

      mesh.draw( gl, "body" );

      Ok( () )
    }

    fn get_defines( &self ) -> String
    {
      self.render_state.get_defines()
    }
  }
}

crate::mod_interface!
{

  orphan use
  {
    Line,
    DashPattern
  };
}