mod private
{

  use crate::*;
  use minwebgl as gl;
  use ndarray_cg as math;
  use d2::solid::*;

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
      let points_buffer = gl.create_buffer().expect( "Failed to create a points buffer" );
      let points_terminal_buffer = gl.create_buffer().expect( "Failed to create a points terminal buffer" );
      let body_instanced_buffer = gl.create_buffer().expect( "Failed to create a body_instanced_buffer" );
      let join_instanced_buffer = gl.create_buffer().expect( "Failed to create a join_instanced_buffer" );
      let cap_instanced_buffer = gl.create_buffer().expect( "Failed to create a cap_instanced_buffer" );

      gl::buffer::upload( gl, &body_instanced_buffer, &helpers::BODY_GEOMETRY, gl::STATIC_DRAW );

      let body_vertex_shader = gl::ShaderSource::former().shader_type( gl::VERTEX_SHADER ).source( BODY_VERTEX_SHADER ).compile( gl )?;

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
      gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 2, &points_buffer )?;

      let mut join_program = Program::default();
      join_program.fragment_shader = Some( fragment_shader.clone() );
      join_program.vao = gl.create_vertex_array();

      let mut cap_program = Program::default();
      cap_program.fragment_shader = Some( fragment_shader.clone() );

      let mut mesh = Mesh::default();
      mesh.program_add( "cap", cap_program );
      mesh.program_add( "join", join_program );
      mesh.program_add( "body", body_program );

      mesh.buffer_add( "body", body_instanced_buffer );
      mesh.buffer_add( "cap", cap_instanced_buffer );
      mesh.buffer_add( "join", join_instanced_buffer );
      mesh.buffer_add( "points", points_buffer );
      mesh.buffer_add( "terminal", points_terminal_buffer );

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

      if self.points.len() > 1
      {
        mesh.draw( gl, "body" );
        mesh.draw( gl, "cap" );
      }

      if self.points.len() > 2
      {
        mesh.draw( gl, "join" );
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
        let terminal_buffer = mesh.buffer_get( "terminal" );
        let points : Vec< f32 > = self.points.iter().map( | p | p.to_array() ).flatten().collect();

        let points_terminal = 
        if self.points.len() >= 2
        {
          let len = self.points.len();
          [ 
            self.points[ 0 ], self.points[ 1 ],
            self.points[ len - 1 ], self.points[ len - 2 ],
          ]
        }
        else
        {
          [ gl::F32x2::ZERO; 4 ]
        };

        let points_terminal : Vec< f32 > = points_terminal.iter().map( | p | p.to_array() ).flatten().collect();

        gl::buffer::upload( &gl, &points_buffer, &points, gl::STATIC_DRAW );
        gl::buffer::upload( &gl, &terminal_buffer, &points_terminal, gl::STATIC_DRAW );

        let b_program = mesh.program_get_mut( "body" );
        b_program.instance_count = Some( ( self.points.len() as f32 - 1.0 ).max( 0.0 ) as u32 );

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

        let ( join_geometry_list, join_indices, _join_uvs, join_geometry_count ) = self.join.geometry(); 
        gl::buffer::upload( gl, &join_buffer, &join_geometry_list, gl::STATIC_DRAW );

        let j_program = mesh.program_get( "join" );
        let vao = gl.create_vertex_array();
        gl.bind_vertex_array( vao.as_ref() ); 
        match self.join
        {
          Join::Round( _ ) =>
          {
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &join_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_buffer )?;
          },
          Join::Miter =>
          {
            gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 3 ).divisor( 0 ).attribute_pointer( &gl, 0, &join_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 2, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 4 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 3, &points_buffer )?;
          },
          Join::Bevel =>
          {
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &join_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 1, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 2, &points_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 4 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 3, &points_buffer )?;
          },
        }

        let ( vertex_shader, draw_mode ) =
        match self.join 
        {
          Join::Round( _) => ( d2::solid::JOIN_ROUND_VERTEX_SHADER, gl::TRIANGLE_FAN ),
          Join::Miter => ( d2::solid::JOIN_MITER_VERTEX_SHADER,gl::TRIANGLES ),
          Join::Bevel => ( d2::solid::JOIN_BEVEL_VERTEX_SHADER, gl::TRIANGLES )
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
        let terminal_buffer = mesh.buffer_get( "terminal" );

        let ( cap_geometry_list, cap_indices, _cap_uvs, cap_geometry_count ) = self.cap.geometry();
        gl::buffer::upload( gl, &cap_buffer, &cap_geometry_list, gl::STATIC_DRAW );

        let c_program = mesh.program_get( "cap" );

        let vao = gl.create_vertex_array();
        gl.bind_vertex_array( vao.as_ref() );
        let mut instance_count = None;
        match self.cap
        {
          Cap::Round( _ ) =>
          {
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &cap_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 4 ).divisor( 1 ).attribute_pointer( &gl, 1, &terminal_buffer )?;
            instance_count = Some( 2 )
          },
          Cap::Square =>
          {
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &cap_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 4 ).divisor( 1 ).attribute_pointer( &gl, 1, &terminal_buffer )?;
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 4 ).divisor( 1 ).attribute_pointer( &gl, 2, &terminal_buffer )?;
            instance_count = Some( 2 )
          }
          _ => {}
        }

        let ( vertex_shader, cap_draw_mode ) =
        match self.cap
        {
          Cap::Round( _ ) =>( d2::solid::CAP_ROUND_VERTEX_SHADER, gl::TRIANGLE_FAN ),
          Cap::Square =>( d2::solid::CAP_SQUARE_VERTEX_SHADER, gl::TRIANGLES ),
          _ => ( d2::solid::CAP_BUTT_VERTEX_SHADER, gl::TRIANGLES )
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