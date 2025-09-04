mod private
{
  use crate::*;
  use minwebgl as gl;
  use ndarray_cg as math;

  /// Represents a 3D line strip, composed of a series of points.
  #[ derive( Debug, Clone, Default ) ]
  pub struct Line
  {
    /// The series of 3D points that define the line strip.
    points : Vec< math::F32x3 >,
    /// Colors for the points
    colors : Vec< math::F32x3 >,
    // The optional `Mesh` object that holds the WebGL resources for rendering.
    /// `None` until `create_mesh` is called.
    mesh : Option< Mesh >,
    /// A flag to indicate whether the line's points have changed since the last update.
    points_changed : bool,
    /// A flag to indicate the colors have been changed
    colors_changed : bool,
    /// A flag to set whether to use the vertex color or not. Should be set before the mesh creation
    use_vertex_color : bool
  }
  
  impl Line
  {
    /// Creates the WebGL mesh for the line.
    ///
    /// This function compiles shaders, generates the line's geometry, creates buffers and a VAO,
    /// and initializes the `Mesh` object. It sets up the vertex attributes for instanced drawing,
    /// where each instance is a segment of the line.
    pub fn mesh_create( &mut self, gl : &gl::WebGl2RenderingContext, fragment_shader : Option< &str > ) -> Result< (), gl::WebglError >
    {
      let fragment_shader_source = fragment_shader.unwrap_or( d3::MAIN_FRAGMENT_SHADER );

      let fragment_shader = 
      if self.use_vertex_color
      {
        fragment_shader_source.replace( "// #include <defines>", "#define USE_VERTEX_COLORS\n" )
      }
      else
      {
        fragment_shader_source.to_string()
      };

      let fragment_shader = gl::ShaderSource::former()
      .shader_type( gl::FRAGMENT_SHADER )
      .source( &fragment_shader )
      .compile( &gl )?;

      let ( vertices, indices, uvs ) = helpers::four_piece_rectangle_geometry();

      let points_buffer = gl.create_buffer().expect( "Failed to create a buffer" );
      let position_buffer = gl.create_buffer().expect( "Failed to create a position_buffer" );
      let index_buffer = gl.create_buffer().expect( "Failed to create a index_buffer" );
      let uv_buffer = gl.create_buffer().expect( "Failed to create a uv_buffer" );
      let color_buffer = gl.create_buffer().expect( "Failed to create a color_buffer" );

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

      let vertex_shader = 
      if self.use_vertex_color
      {
        d3::MAIN_VERTEX_SHADER.replace( "// #include <defines>", "#define USE_VERTEX_COLORS\n" )
      }
      else
      {
        d3::MAIN_VERTEX_SHADER.to_string()
      };


      let vertex_shader = gl::ShaderSource::former()
      .shader_type( gl::VERTEX_SHADER )
      .source( &vertex_shader )
      .compile( &gl )?;

      let program = gl::ProgramShaders::new( &vertex_shader, &fragment_shader ).link( &gl )?;
      let program = Program
      {
        vertex_shader : Some( vertex_shader ),
        fragment_shader : Some( fragment_shader ),
        vao : vao,
        program : Some( program ),
        draw_mode : gl::TRIANGLES,
        instance_count : Some( ( self.points.len() as f32 - 1.0 ).max( 0.0 ) as u32 ),
        index_count : Some( indices.len() as u32 ),
        vertex_count : vertices.len() as u32,
        index_buffer : Some( index_buffer )
      };

      let mut mesh = Mesh::default();
      mesh.add_program( "body", program );

      mesh.add_buffer( "position", position_buffer );
      mesh.add_buffer( "points", points_buffer );
      mesh.add_buffer( "uv", uv_buffer );
      mesh.add_buffer( "colors", color_buffer );

      self.mesh = Some( mesh );

      self.points_changed = true;
      self.colors_changed = true;

      self.mesh_update( gl )?;

      Ok( () )
    }

    /// Updates the mesh's vertex buffers if the line's points have changed.
    pub fn mesh_update( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      let mesh = self.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;

      if self.points_changed
      {
        let points_buffer = mesh.get_buffer( "points" );
        
        let points : Vec< f32 > = self.points.iter().flat_map( | p | p.to_array() ).collect();
        gl::buffer::upload( &gl, &points_buffer, &points, gl::STATIC_DRAW );

        let b_program = mesh.get_program_mut( "body" );
        b_program.instance_count = Some( ( self.points.len() as f32 - 1.0 ).max( 0.0 ) as u32 );

        self.points_changed = false;
      }

      if self.colors_changed && self.use_vertex_color
      {
        let colors_buffer = mesh.get_buffer( "colors" );

        let colors : Vec< f32 > = self.colors.iter().flat_map( | c | c.to_array() ).collect();
        gl::buffer::upload( &gl, &colors_buffer, &colors, gl::STATIC_DRAW );

        self.colors_changed = false;
      }

      Ok( () )
    }

    /// Sets whether the vertex color attribute will be used or not
    pub fn use_vertex_color( &mut self, value : bool )
    {
      self.use_vertex_color = value;
    }

    /// Adds a new point to the end of the line strip.
    pub fn point_add< P : gl::VectorIter< f32, 3 > >( &mut self, point : P )
    {
      let mut iter = point.vector_iter();
      let point = gl::F32x3::new( *iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap() );

      self.points.push( point );
      self.points_changed = true;
    }

    /// Adds the color to a list of colors. Each color belongs to a point with the same index;
    pub fn color_add< C : gl::VectorIter< f32, 3 > >( &mut self, color : C )
    {
      let mut iter = color.vector_iter();
      let color = gl::F32x3::new( *iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap() );

      self.colors.push( color );
      self.colors_changed = true;
    }

    /// Retrieves the points at the specified position.
    /// Will panic if index is out of range
    pub fn point_get( &self, index : usize ) -> gl::F32x3
    {
      self.points[ index ]
    }

    /// Sets the points at the specified position.
    /// Will panic if index is out of range
    pub fn point_set< P : gl::VectorIter< f32, 3 > >( &mut self, point : P, index : usize )
    {
      let mut iter = point.vector_iter();
      let point = gl::F32x3::new( *iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap() );
      self.points[ index ] = point;
      self.points_changed = true;
    }

    /// Draws the line mesh.
    pub fn draw( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      self.mesh_update( gl )?;

      let mesh = self.mesh.as_ref().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;
      mesh.draw( gl, "body" );

      Ok( () )
    }

    /// Retrieves a reference to the mesh.
    pub fn mesh_get( &self ) -> Result< &Mesh, gl::WebglError >
    {
      self.mesh.as_ref().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )
    }  

    /// Retrieves a mutable reference to the mesh.
    pub fn mesh_get_mut( &mut self ) -> Result< &mut Mesh, gl::WebglError >
    {
      self.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )
    }  

    /// Retrieves a slice of the line's points.
    pub fn get_points( &self ) -> &[ math::F32x3 ]
    {
      &self.points
    }  

    /// Return the number of points that form this line
    pub fn num_points( &self ) -> usize
    {
      self.points.len()
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