mod private
{

  use crate::*;
  use minwebgl as gl;
  use ndarray_cg as math;
  use d2::uv::*;

  /// Represents a renderable 2D line with configurable caps and joins.
  #[ derive( Debug, Clone, Default ) ]
  pub struct Line
  {
    /// A vector of 2D points that define the line's path.
    points : Vec< math::F32x2 >,
    /// The distance from the beginning of the line to the current point
    distances : Vec< f32 >,
    /// Total length of the line
    total_distance : f32,
    /// The style of the line's end caps (`Butt`, `Round`, or `Square`).
    cap : Cap,
    /// The style of the line's joins between segments (`Round`, `Miter`, or `Bevel`).
    join : Join,
    /// The mesh object that encapsulates all WebGL buffers and shader programs.
    mesh : Option< Mesh >,
    /// A flag to indicate if the join style has been changed.
    join_changed : bool,
    /// A flag to indicate if the cap style has been changed.
    cap_changed : bool,
    /// A flag to indicate if the line's points have been changed.
    points_changed : bool
  }

  impl Line
  {
    /// Creates and initializes the WebGL mesh for the line.
    ///
    /// This function compiles all necessary shaders, creates all buffers, sets up
    /// the vertex attribute pointers, and links the shader programs.
    pub fn mesh_create( &mut self, gl : &gl::WebGl2RenderingContext, fragment_shader : &str ) -> Result< (), gl::WebglError >
    {
      let fragment_shader = gl::ShaderSource::former()
      .shader_type( gl::FRAGMENT_SHADER )
      .source( fragment_shader )
      .compile( &gl )?;

      // Buffers
      let points_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to points_buffer" ) )?;
      let distance_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to distance_buffer" ) )?;
      let points_terminal_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to points_terminal_buffer" ) )?;
      let body_instanced_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to body_instanced_buffer" ) )?;
      let join_instanced_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to join_instanced_buffer" ) )?;
      let join_indices_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to join_indices_buffer" ) )?;
      let join_uv_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to join_uv_buffer" ) )?;
      let cap_instanced_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to cap_instanced_buffer" ) )?;
      let cap_indices_buffer = gl.create_buffer().ok_or( gl::WebglError::Other( "Failed to cap_indices_buffer" ) )?;


      let uv_buffer = gl.create_buffer().expect( "Failed to create a uv_buffer" );

      gl::buffer::upload( gl, &body_instanced_buffer, &helpers::BODY_GEOMETRY, gl::STATIC_DRAW );

      let body_vertex_shader = gl::ShaderSource::former().shader_type( gl::VERTEX_SHADER ).source( BODY_VERTEX_SHADER ).compile( gl )?;
      let body_terminal_vertex_shader = gl::ShaderSource::former().shader_type( gl::VERTEX_SHADER ).source( BODY_TERMINAL_VERTEX_SHADER ).compile( gl )?;

      let mut body_program = Program::default();
      body_program.program = Some( gl::ProgramShaders::new( &body_vertex_shader, &fragment_shader ).link( gl )? );
      body_program.vertex_shader = Some( body_vertex_shader );
      body_program.fragment_shader = Some( fragment_shader.clone() );
      body_program.draw_mode = gl::TRIANGLES;
      body_program.instance_count = Some( ( self.points.len() as f32 - 1.0 ).max( 0.0 ) as u32 );
      body_program.vertex_count = helpers::BODY_GEOMETRY.len() as u32;
      

      body_program.vao = gl.create_vertex_array();
      gl.bind_vertex_array( body_program.vao.as_ref() );

      gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &body_instanced_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 3 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 2, &points_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 6 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 3, &points_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 9 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 4, &points_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 1 ] >().offset( 0 ).stride( 1 ).divisor( 0 ).attribute_pointer( &gl, 5, &distance_buffer )?;

      let mut body_terminal_program = Program::default();
      body_terminal_program.program = Some( gl::ProgramShaders::new( &body_terminal_vertex_shader, &fragment_shader ).link( gl )? );
      body_terminal_program.vertex_shader = Some( body_terminal_vertex_shader );
      body_terminal_program.fragment_shader = Some( fragment_shader.clone() );
      body_terminal_program.draw_mode = gl::TRIANGLES;
      body_terminal_program.instance_count = Some( 2 );
      body_terminal_program.vertex_count = helpers::BODY_GEOMETRY.len() as u32;

      body_terminal_program.vao = gl.create_vertex_array();
      gl.bind_vertex_array( body_terminal_program.vao.as_ref() );

      gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &body_instanced_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 9 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_terminal_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 3 ).stride( 9 ).divisor( 1 ).attribute_pointer( &gl, 2, &points_terminal_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 6 ).stride( 9 ).divisor( 1 ).attribute_pointer( &gl, 3, &points_terminal_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 1 ] >().offset( 0 ).stride( 1 ).divisor( 0 ).attribute_pointer( &gl, 4, &distance_buffer )?;

      let mut join_program = Program::default();
      join_program.fragment_shader = Some( fragment_shader.clone() );
      join_program.vao = gl.create_vertex_array();
      join_program.index_buffer = Some( join_indices_buffer.clone() );

      let mut cap_program = Program::default();
      cap_program.fragment_shader = Some( fragment_shader.clone() );
      cap_program.index_buffer = Some( cap_indices_buffer.clone() );

      let mut mesh = Mesh::default();
      mesh.program_add( "cap", cap_program );
      mesh.program_add( "join", join_program );
      mesh.program_add( "body", body_program );
      mesh.program_add( "body_terminal", body_terminal_program );

      mesh.buffer_add( "body", body_instanced_buffer );
      mesh.buffer_add( "cap", cap_instanced_buffer );
      mesh.buffer_add( "cap_indices", cap_indices_buffer );
      mesh.buffer_add( "join", join_instanced_buffer );
      mesh.buffer_add( "join_indices", join_indices_buffer );
      mesh.buffer_add( "join_uv", join_uv_buffer );
      mesh.buffer_add( "points", points_buffer );
      mesh.buffer_add( "points_terminal", points_terminal_buffer );
      mesh.buffer_add( "distance", distance_buffer );

      mesh.buffer_add( "uv", uv_buffer );

      self.mesh = Some( mesh );

      self.cap_changed = true;
      self.join_changed = true;
      self.points_changed = true;

      self.mesh_update( gl )?;

      Ok( () )
    }

    /// Clears the points from the line without releasing the memory
    pub fn clear( &mut self )
    {
      self.points.clear();
      self.distances.clear();
      self.total_distance = 0.0;
      self.points_changed = true;
    }

    /// Sets the join style of the line and marks it for an update.
    pub fn join_set( &mut self, join : Join )
    {
      self.join = join;
      self.join_changed = true;
    }

    /// Sets the cap style of the line and marks it for an update.
    pub fn cap_set( &mut self, cap : Cap )
    {
      self.cap = cap;
      self.cap_changed = true;
    }

    /// Adds a new point to the line and marks the points as changed.
    pub fn point_add< P : gl::VectorIter< f32, 2 > >( &mut self, point : P )
    {
      let mut iter = point.vector_iter();
      let point = gl::F32x2::new( *iter.next().unwrap(), *iter.next().unwrap() );
      let distance = if let Some( last ) = self.points.last().copied()
      {
        const EPSILON : f32 = 1e-8;
        if ( last.x() - point.x() ).abs() < EPSILON && ( last.y() - point.y() ).abs() < EPSILON 
        {
          return;
        }

        ( point - last ).mag() 
      }
      else
      {
        0.0
      };

      self.total_distance += distance;
      self.distances.push( self.total_distance );

      self.points.push( point );
      self.points_changed = true;
    }

    /// Retrieves the points at the specified position.
    /// Will panic if index is out of range
    pub fn point_get( &self, index : usize ) -> gl::F32x2
    {
      self.points[ index ]
    }

    /// Sets the points at the specified position.
    /// Will panic if index is out of range
    pub fn point_set< P : gl::VectorIter< f32, 2 > >( &mut self, point : P, index : usize )
    {
      let mut iter = point.vector_iter();
      let point = gl::F32x2::new( *iter.next().unwrap(), *iter.next().unwrap() );
      self.points[ index ] = point;
      self.recalculate_distances();
      self.points_changed = true;
    }

    /// Return the total lenth of the line
    pub fn total_distance( &self ) -> f32
    {
      self.total_distance
    }

    /// Return the number of points that form this line
    pub fn num_points( &self ) -> usize
    {
      self.points.len()
    }

    /// Updates the mesh's WebGL resources if any part of the line has changed.
    pub fn mesh_update( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      self.points_change( gl )?;
      self.join_change( gl )?;
      self.cap_change( gl )?;

      gl.bind_buffer( gl::ARRAY_BUFFER, None );
      gl.bind_buffer( gl::ELEMENT_ARRAY_BUFFER, None );

      Ok( () )
    }

    /// Update the line and draw it if it has more than one point.
    pub fn draw( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {

      self.mesh_update( gl )?;

      let mesh = self.mesh.as_ref().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;

      mesh.upload_to( gl, "body", "u_total_distance", &self.total_distance )?;
      mesh.upload_to( gl, "body_terminal", "u_total_distance", &self.total_distance )?;

      mesh.draw( gl, "body" );
      mesh.draw( gl, "body_terminal" );
      
      if self.points.len() > 2
      {
        mesh.upload_to( gl, "join", "u_total_distance", &self.total_distance )?;
        mesh.draw( gl, "join" );
      }

      if self.points.len() > 1
      {
        mesh.draw( gl, "cap" );
      }

      Ok( () )
    }

    /// Returns a reference to the internal mesh.
    pub fn mesh_get( &self ) -> Result< &Mesh, gl::WebglError >
    {
      self.mesh.as_ref().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )
    }   

    /// Returns a mutable reference to the internal mesh.
    pub fn mesh_get_mut( &mut self ) -> Result< &mut Mesh, gl::WebglError >
    {
      self.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )
    } 

    /// Returns a slice of the line's points.
    pub fn points_get( &self ) -> &[ math::F32x2 ]
    {
      &self.points
    }

    fn recalculate_distances( &mut self )
    {
      self.total_distance = 0.0;
      self.distances.clear();
      self.distances.push( 0.0 );
      for i in 1..self.points.len()
      {
        let point = self.points[ i ];
        let last = self.points[ i - 1 ];
        self.total_distance += ( point - last ).mag() ;
        self.distances.push( self.total_distance );
      }
    }

    fn points_change( &mut self, gl : &gl::WebGl2RenderingContext  ) -> Result< (), gl::WebglError >
    {
      let mesh = self.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;

      if self.points_changed
      {
        let points_buffer = mesh.buffer_get( "points" );
        let distance_buffer = mesh.buffer_get( "distance" );
        let points_terminal_buffer = mesh.buffer_get( "points_terminal" );
        let points : Vec< f32 > = self.points.iter().zip( self.distances.iter() ).flat_map( | ( p, d ) | [ p.x(), p.y(), *d / self.total_distance ] ).collect();
        let ( points_terminal, uvs_terminal, terminal_instance_count ) = 
        if self.points.len() >= 3
        {
          let len = self.points.len();
          (
            [ 
              self.points[ 0 ], self.points[ 1 ], self.points[ 2 ],
              self.points[ len - 1 ], self.points[ len - 2 ], self.points[ len - 3 ]
            ],
            [ 
              self.distances[ 0 ], self.distances[ 1 ], self.distances[ 2 ],
              self.distances[ len - 1 ], self.distances[ len - 2 ], self.distances[ len - 3 ]
            ],
            2
          )
        }
        else if self.points.len() == 2
        {
          let dir = self.points[ 1 ] - self.points[ 0 ];
          (
            [ 
              self.points[ 0 ], self.points[ 1 ], self.points[ 1 ] + dir,
              self.points[ 1 ], self.points[ 0 ], self.points[ 0 ] - dir,
            ],
            [ 
              self.distances[ 0 ], self.distances[ 1 ], self.distances[ 1 ],
              self.distances[ 1 ], self.distances[ 0 ], self.distances[ 0 ],
            ],
            1
          )
        }
        else
        {
          let zero = math::F32x2::default();
          ( [ zero; 6 ], [ 0.0; 6 ], 0 )
        };

        let points_terminal : Vec< f32 > = points_terminal.into_iter().zip( uvs_terminal.iter() ).flat_map( | ( p, d ) | [ p.x(), p.y(), d / self.total_distance ] ).collect();

        gl::buffer::upload( &gl, &points_buffer, &points, gl::STATIC_DRAW );
        gl::buffer::upload( &gl, &points_terminal_buffer, &points_terminal, gl::STATIC_DRAW );
        gl::buffer::upload( &gl, &distance_buffer, &self.distances, gl::STATIC_DRAW );

        let b_program = mesh.program_get_mut( "body" );
        b_program.instance_count = Some( ( self.points.len() as f32 - 3.0 ).max( 0.0 ) as u32 );

        let bt_program = mesh.program_get_mut( "body_terminal" );
        bt_program.instance_count = Some( terminal_instance_count );
        
        let j_program = mesh.program_get_mut( "join" );
        j_program.instance_count = Some( ( self.points.len() as f32 - 2.0 ).max( 0.0 ) as u32 );

        self.points_changed = false;
      }

      Ok( () )
    }

    fn join_change( &mut self, gl : &gl::WebGl2RenderingContext  ) -> Result< (), gl::WebglError >
    {
      let mesh = self.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;

      if self.join_changed
      {
        let points_buffer = mesh.buffer_get( "points" );
        let join_buffer = mesh.buffer_get( "join" );
        let join_indices_buffer = mesh.buffer_get( "join_indices" );
        let join_uv_buffer = mesh.buffer_get( "join_uv" );
        let distance_buffer = mesh.buffer_get( "distance" );

        let ( join_geometry_list, join_indices, join_uvs, join_geometry_count ) = self.join.geometry(); 
        gl::buffer::upload( gl, &join_buffer, &join_geometry_list, gl::STATIC_DRAW );
        gl::buffer::upload( gl, &join_uv_buffer, &join_uvs, gl::STATIC_DRAW );
        gl::index::upload( gl, &join_indices_buffer, &join_indices, gl::STATIC_DRAW );

        let j_program = mesh.program_get( "join" );
        let vao = gl.create_vertex_array();
        gl.bind_vertex_array( vao.as_ref() ); 
        match self.join
        {
          Join::Round( _, _ ) =>
          {
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &join_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 3 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 2, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 6 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 3, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 1 ] >().offset( 0 ).stride( 1 ).divisor( 0 ).attribute_pointer( &gl, 4, &join_uv_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 1 ] >().offset( 0 ).stride( 1 ).divisor( 0 ).attribute_pointer( &gl, 5, &distance_buffer )?;
          },
          Join::Miter( _, _ ) =>
          {
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 3 ).divisor( 0 ).attribute_pointer( &gl, 0, &join_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 3 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 2, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 6 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 3, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 1 ] >().offset( 0 ).stride( 1 ).divisor( 0 ).attribute_pointer( &gl, 4, &join_uv_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 1 ] >().offset( 0 ).stride( 1 ).divisor( 0 ).attribute_pointer( &gl, 5, &distance_buffer )?;
          },
          Join::Bevel( _, _ ) =>
          {
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &join_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 3 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 2, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 6 ).stride( 3 ).divisor( 1 ).attribute_pointer( &gl, 3, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 1 ] >().offset( 0 ).stride( 1 ).divisor( 0 ).attribute_pointer( &gl, 4, &join_uv_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 1 ] >().offset( 0 ).stride( 1 ).divisor( 0 ).attribute_pointer( &gl, 5, &distance_buffer )?;
          },
        }

        let ( vertex_shader, draw_mode ) =
        match self.join 
        {
          Join::Round( _, _ ) => ( JOIN_ROUND_VERTEX_SHADER, gl::TRIANGLES ),
          Join::Miter( _, _ ) => ( JOIN_MITER_VERTEX_SHADER,gl::TRIANGLES ),
          Join::Bevel( _, _ ) => ( JOIN_BEVEL_VERTEX_SHADER, gl::TRIANGLES )
        };

        let vertex_shader = gl::ShaderSource::former()
        .shader_type( gl::VERTEX_SHADER )
        .source( vertex_shader )
        .compile( &gl )?;
        let join_program = gl::ProgramShaders::new( &vertex_shader, j_program.fragment_shader.as_ref()
        .ok_or( gl::WebglError::Other( "Fragment shader has not been set" ) )? )
        .link( &gl )?;
        j_program.uniforms_copy_to( gl, &join_program )?;

        let j_program = mesh.program_get_mut( "join" );

        j_program.vertex_shader_delete( gl );
        j_program.program_delete( gl );
        j_program.vao_delete( gl );

        j_program.vao = vao;
        j_program.draw_mode = draw_mode;
        j_program.vertex_shader = Some( vertex_shader );
        j_program.program = Some( join_program );
        j_program.instance_count = Some( ( self.points.len() as f32 - 2.0 ).max( 0.0 ) as u32 );
        j_program.vertex_count = join_geometry_count as u32;
        j_program.index_count = if join_indices.len() > 0 { Some( join_indices.len() as u32 ) } else { None };

        self.join_changed = false;
      }

      Ok( () )
    }

    fn cap_change( &mut self, gl : &gl::WebGl2RenderingContext  ) -> Result< (), gl::WebglError >
    {
      let mesh = self.mesh.as_mut().ok_or( gl::WebglError::Other( "Mesh has not been created yet" ) )?;

      if self.cap_changed
      {
        let cap_buffer = mesh.buffer_get( "cap" );
        let cap_index_buffer = mesh.buffer_get( "cap_indices" );
        let points_terminal_buffer = mesh.buffer_get( "points_terminal" );

        let ( cap_geometry_list, cap_indices, _cap_uvs, cap_geometry_count ) = self.cap.geometry();
        gl::buffer::upload( gl, &cap_buffer, &cap_geometry_list, gl::STATIC_DRAW );
        gl::index::upload( gl, &cap_index_buffer, &cap_indices, gl::STATIC_DRAW );

        let c_program = mesh.program_get( "cap" );

        let vao = gl.create_vertex_array();
        gl.bind_vertex_array( vao.as_ref() );
        let mut instance_count = None;
        match self.cap
        {
          Cap::Round( _ ) =>
          {
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &cap_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 9 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_terminal_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 3 ).stride( 9 ).divisor( 1 ).attribute_pointer( &gl, 2, &points_terminal_buffer )?;
            instance_count = Some( 2 );
          },
          Cap::Square =>
          {
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &cap_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 9 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_terminal_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 3 ).stride( 9 ).divisor( 1 ).attribute_pointer( &gl, 2, &points_terminal_buffer )?;
            instance_count = Some( 2 );
          }
          _ => {}
        }

        let ( vertex_shader, cap_draw_mode ) =
        match self.cap
        {
          Cap::Round( _ ) =>( CAP_ROUND_VERTEX_SHADER, gl::TRIANGLES ),
          Cap::Square =>( CAP_SQUARE_VERTEX_SHADER, gl::TRIANGLES ),
          _ => ( CAP_BUTT_VERTEX_SHADER, gl::TRIANGLES )
        };

        let vertex_shader = gl::ShaderSource::former()
        .shader_type( gl::VERTEX_SHADER )
        .source( vertex_shader )
        .compile( &gl )?;
        let cap_program = gl::ProgramShaders::new( &vertex_shader, c_program.fragment_shader.as_ref()
        .ok_or( gl::WebglError::Other( "Fragment shader has not been set" ) )? )
        .link( &gl )?;
        mesh.program_get( "join" ).uniforms_copy_to( gl, &cap_program )?;
        c_program.uniforms_copy_to( gl, &cap_program )?;

        let c_program = mesh.program_get_mut( "cap" );

        c_program.vertex_shader_delete( gl );
        c_program.program_delete( gl );
        c_program.vao_delete( gl );

        c_program.vao = vao;
        c_program.vertex_shader = Some( vertex_shader );
        c_program.program = Some( cap_program );
        c_program.instance_count = instance_count;
        c_program.vertex_count = cap_geometry_count as u32;
        c_program.draw_mode = cap_draw_mode;
        c_program.index_count = if cap_indices.len() > 0 { Some( cap_indices.len() as u32 ) } else { None };

        self.cap_changed = false;
      }

      Ok( () )
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