mod private
{
  use crate::*;
  use minwebgl as gl;
  use ndarray_cg as math;
  use std::collections::VecDeque;

  /// Represents a 3D line strip, composed of a series of points.
  #[ derive( Debug, Clone, Default ) ]
  pub struct Line
  {
    /// The series of 3D points that define the line strip.
    points : VecDeque< math::F32x3 >,
    /// Colors for the points
    colors : VecDeque< math::F32x3 >,
    // The optional `Mesh` object that holds the WebGL resources for rendering.
    /// `None` until `create_mesh` is called.
    mesh : Option< Mesh >,
    /// A flag to set whether to use the vertex color or not. Should be set before the mesh creation
    use_vertex_color : bool,
    /// A flag to set where to use alpha to coverage blending technique instead of alpha testing 
    use_alpha_to_coverage : bool,
    /// A flag to set where to use width in world units, or screen space units
    use_world_units : bool,
    /// Fragment shader source
    fragment_shader : String,
    /// A flag to indicate whether the line's points have changed since the last update.
    points_changed : bool,
    /// A flag to indicate the colors have been changed
    colors_changed : bool,
    /// A flag to indicate any shader defines have been changed
    defines_changed : bool
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
      self.fragment_shader = fragment_shader.unwrap_or( d3::MAIN_FRAGMENT_SHADER ).to_string();

      let ( vertices, indices, uvs ) = helpers::four_piece_rectangle_geometry();

      let points_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to points_buffer" ) )?;
      let position_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to position_buffer" ) )?;
      let index_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to index_buffer" ) )?;
      let uv_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to uv_buffer" ) )?;
      let color_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to color_buffer" ) )?;

      gl::buffer::upload( gl, &position_buffer, &vertices.iter().copied().flatten().collect::< Vec< f32 > >(), gl::STATIC_DRAW );
      gl::buffer::upload( gl, &uv_buffer, &uvs.iter().copied().flatten().collect::< Vec< f32 > >(), gl::STATIC_DRAW );
      gl::index::upload( gl, &index_buffer, &indices, gl::STATIC_DRAW );

      let vao = gl.create_vertex_array();
      gl.bind_vertex_array( vao.as_ref() );

      gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 2 ).offset( 0 ).divisor( 0 ).attribute_pointer( gl, 0, &position_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 2 ).offset( 0 ).divisor( 0 ).attribute_pointer( gl, 1, &uv_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).divisor( 1 ).attribute_pointer( gl, 2, &points_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 3 ).divisor( 1 ).attribute_pointer( gl, 3, &points_buffer )?;

      if self.use_vertex_color
      {
        gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).divisor( 1 ).attribute_pointer( gl, 4, &color_buffer )?;
        gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 3 ).divisor( 1 ).attribute_pointer( gl, 5, &color_buffer )?;
      }

      let program = Program
      {
        vertex_shader : None,
        fragment_shader : None,
        vao : vao,
        program : None,
        draw_mode : gl::TRIANGLES,
        instance_count : Some( ( self.points.len() as f32 - 1.0 ).max( 0.0 ) as u32 ),
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

      self.mesh = Some( mesh );

      self.points_changed = true;
      self.colors_changed = true;
      self.defines_changed = true;

      self.mesh_update( gl )?;

      Ok( () )
    }

    /// Updates the mesh's vertex buffers if the line's points have changed.
    pub fn mesh_update( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      if self.defines_changed
      {
        let defines = self.get_defines();
        let vertex_shader = d3::MAIN_VERTEX_SHADER.replace( "// #include <defines>", &defines );
        let vertex_shader = gl::ShaderSource::former()
        .shader_type( gl::VERTEX_SHADER )
        .source( &vertex_shader )
        .compile( &gl )?;

        let fragment_shader = self.fragment_shader.replace( "// #include <defines>", &defines );
        let fragment_shader = gl::ShaderSource::former()
        .shader_type( gl::FRAGMENT_SHADER )
        .source( &fragment_shader )
        .compile( &gl )?;

        let program = gl::ProgramShaders::new( &vertex_shader, &fragment_shader ).link( &gl )?;

        let mesh = self.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;
        let b_program = mesh.program_get_mut( "body" );

        b_program.fragment_shader_delete( gl );
        b_program.vertex_shader_delete( gl );
        b_program.program_delete( gl );

        b_program.program = Some( program );
        b_program.fragment_shader = Some( fragment_shader );
        b_program.vertex_shader = Some( vertex_shader );

        b_program.uniform_locations_clear();
        b_program.all_uniforms_upload( gl )?;

        self.defines_changed = false;
      }

      if self.points_changed
      {
        let mesh = self.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;
        let points_buffer = mesh.buffer_get( "points" );
        
        let points : Vec< f32 > = self.points.iter().flat_map( | p | p.to_array() ).collect();
        gl::buffer::upload( &gl, &points_buffer, &points, gl::STATIC_DRAW );

        let b_program = mesh.program_get_mut( "body" );
        b_program.instance_count = Some( ( self.points.len() as f32 - 1.0 ).max( 0.0 ) as u32 );

        self.points_changed = false;
      }

      if self.colors_changed && self.use_vertex_color
      {
        let mesh = self.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;
        let colors_buffer = mesh.buffer_get( "colors" );

        let colors : Vec< f32 > = self.colors.iter().flat_map( | c | c.to_array() ).collect();
        gl::buffer::upload( &gl, &colors_buffer, &colors, gl::STATIC_DRAW );

        self.colors_changed = false;
      }

      Ok( () )
    }

    /// Sets whether the alpha to coverage will be used or not
    pub fn use_alpha_to_coverage( &mut self, value : bool )
    {
      self.use_alpha_to_coverage = value;
      self.defines_changed = true;
    }

    /// Sets whether the world units for the line width will be used
    pub fn use_world_units( &mut self, value : bool )
    {
      self.use_world_units = value;
      self.defines_changed = true;
    }

    /// Draws the line mesh.
    pub fn draw( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      self.mesh_update( gl )?;

      let mesh = self.mesh.as_ref().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;
      mesh.draw( gl, "body" );

      Ok( () )
    }

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

      s
    }
  }
}

crate::mod_interface!
{

  orphan use
  {
    Line
  };
}