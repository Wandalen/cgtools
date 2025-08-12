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
    // The optional `Mesh` object that holds the WebGL resources for rendering.
    /// `None` until `create_mesh` is called.
    mesh : Option< Mesh >,
    /// A flag to indicate whether the line's points have changed since the last update.
    points_changed : bool
  }
  
  impl Line
  {
    /// Creates the WebGL mesh for the line.
    ///
    /// This function compiles shaders, generates the line's geometry, creates buffers and a VAO,
    /// and initializes the `Mesh` object. It sets up the vertex attributes for instanced drawing,
    /// where each instance is a segment of the line.
    pub fn create_mesh( &mut self, gl : &gl::WebGl2RenderingContext, segments : u32, fragment_shader : &str ) -> Result< (), gl::WebglError >
    {
      let fragment_shader = gl::ShaderSource::former()
      .shader_type( gl::FRAGMENT_SHADER )
      .source( fragment_shader )
      .compile( &gl )?;


      let body_geometry : Vec< [ f32; 3 ] > = helpers::BODY_GEOMETRY.into_iter()
      .map( | v | { [ 0.0, v[ 1 ], v[ 0 ] ] } )
      .collect();
      let circle_left_half_geometry : Vec< [ f32; 3 ]> = helpers::circle_left_half_geometry( segments as usize )
      .into_iter()
      .map( | v | { [ v[ 0 ], v[ 1 ], 0.0 ] } )
      .collect();
      let circle_right_half_geometry : Vec< [ f32; 3 ]> = helpers::circle_right_half_geometry( segments as usize )
      .into_iter()
      .map( | v | { [ v[ 0 ], v[ 1 ], 1.0 ] } )
      .collect();

      let body_count = body_geometry.len();
      let circle_left_half_count = circle_left_half_geometry.len();
      let circle_right_half_count = circle_right_half_geometry.len();

      let vertex_count = body_count + circle_left_half_count + circle_right_half_count;

      let mut geometry = Vec::new();
      geometry.extend_from_slice( &circle_left_half_geometry );
      geometry.extend_from_slice( &body_geometry );
      geometry.extend_from_slice( &circle_right_half_geometry );


      let points_buffer = gl.create_buffer().expect( "Failed to create a buffer" );
      let body_instanced_buffer = gl.create_buffer().expect( "Failed to create a instanced_buffer" );

      gl::buffer::upload( gl, &body_instanced_buffer, &geometry, gl::STATIC_DRAW );

      let vao = gl.create_vertex_array();
      gl.bind_vertex_array( vao.as_ref() );

      gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).divisor( 0 ).attribute_pointer( gl, 0, &body_instanced_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).divisor( 1 ).attribute_pointer( gl, 1, &points_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 3 ).divisor( 1 ).attribute_pointer( gl, 2, &points_buffer )?;

      let vertex_shader = gl::ShaderSource::former()
      .shader_type( gl::VERTEX_SHADER )
      .source( d3::MERGED_VERTEX_SHADER )
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
        index_count : None,
        vertex_count : vertex_count as u32,
        index_buffer : None
      };

      let mut mesh = Mesh::default();
      mesh.add_program( "body", program );

      mesh.add_buffer( "body", body_instanced_buffer );
      mesh.add_buffer( "points", points_buffer );

      self.mesh = Some( mesh );

      self.points_changed = true;
      self.update_mesh( gl )?;

      Ok( () )
    }

    /// Updates the mesh's vertex buffers if the line's points have changed.
    pub fn update_mesh( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      let mesh = self.mesh.as_mut().expect( "Mesh has not been created yet" );

      if self.points_changed
      {
        let points_buffer = mesh.get_buffer( "points" );
        
        let points : Vec< f32 > = self.points.iter().flat_map( | p | p.to_array() ).collect();
        gl::buffer::upload( &gl, &points_buffer, &points, gl::STATIC_DRAW );

        let b_program = mesh.get_program_mut( "body" );
        b_program.instance_count = Some( ( self.points.len() as f32 - 1.0 ).max( 0.0 ) as u32 );

        self.points_changed = false;
      }

      Ok( () )
    }

    /// Adds a new point to the end of the line strip.
    pub fn add_point( &mut self, point : math::F32x3 )
    {
      self.points.push( point );
      self.points_changed = true;
    }

    /// Draws the line mesh.
    pub fn draw( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      self.update_mesh( gl )?;

      let mesh = self.mesh.as_ref().expect( "Mesh has not been created yet" );
      mesh.draw( gl, "body" );

      Ok( () )
    }

    /// Retrieves a reference to the mesh.
    pub fn get_mesh( &self ) -> &Mesh
    {
      self.mesh.as_ref().expect( "Mesh has not been created yet" )
    }  

    /// Retrieves a mutable reference to the mesh.
    pub fn get_mesh_mut( &mut self ) -> &mut Mesh
    {
      self.mesh.as_mut().expect( "Mesh has not been created yet" )
    }  

    /// Retrieves a slice of the line's points.
    pub fn get_points( &self ) -> &[ math::F32x3 ]
    {
      &self.points
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