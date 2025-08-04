mod private
{

  use crate::*;
  use minwebgl::{self as gl, IntoArray};
  use ndarray_cg as math;

  #[ derive( Debug, Clone, Default ) ]
  pub struct Line
  {
    pub points : Vec< math::F32x2 >,
    pub cap : Cap,
    pub join : Join,
    mesh : Option< Mesh >,
    join_changed : bool,
    cap_changed : bool,
    points_changed : bool
  }

  impl Line
  {
    pub fn create_mesh( &mut self, gl : &gl::WebGl2RenderingContext, fragment_shader : &str ) -> Result< (), gl::WebglError >
    {
      let fragment_shader = gl::ShaderSource::former()
      .shader_type( gl::FRAGMENT_SHADER )
      .source( fragment_shader )
      .compile( &gl )?;

      // Buffers
      let points_buffer = gl.create_buffer().expect( "Failed to create a points buffer" );
      let body_instanced_buffer = gl.create_buffer().expect( "Failed to create a body_instanced_buffer" );
      let join_instanced_buffer = gl.create_buffer().expect( "Failed to create a join_instanced_buffer" );
      let cap_instanced_buffer = gl.create_buffer().expect( "Failed to create a cap_instanced_buffer" );

      let body_vertex_shader = gl::ShaderSource::former().shader_type( gl::VERTEX_SHADER ).source( d2::BODY_VERTEX_SHADER ).compile( gl )?;
      gl::buffer::upload( gl, &body_instanced_buffer, &helpers::BODY_GEOMETRY, gl::STATIC_DRAW );

      let mut body_program = Program::default();
      body_program.program = Some( gl::ProgramShaders::new( &body_vertex_shader, &fragment_shader ).link( gl )? );
      body_program.vertex_shader = Some( body_vertex_shader );
      body_program.fragment_shader = Some( fragment_shader.clone() );
      body_program.draw_mode = gl::TRIANGLES;
      body_program.instance_count = Some( ( self.points.len() - 1 ) as u32 );
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
      mesh.add_program( "cap", cap_program );
      mesh.add_program( "join", join_program );
      mesh.add_program( "body", body_program );

      mesh.add_buffer( "body", body_instanced_buffer );
      mesh.add_buffer( "cap", cap_instanced_buffer );
      mesh.add_buffer( "join", join_instanced_buffer );
      mesh.add_buffer( "points", points_buffer );

      self.mesh = Some( mesh );

      self.cap_changed = true;
      self.join_changed = true;
      self.points_changed = true;

      self.update_mesh( gl )?;

      Ok( () )
    }

    pub fn set_join( &mut self, join : Join )
    {
      self.join = join;
      self.join_changed = true;
    }

    pub fn set_cap( &mut self, cap : Cap )
    {
      self.cap = cap;
      self.cap_changed = true;
    }

    pub fn add_point( &mut self, point : gl::F32x2 )
    {
      self.points.push( point );
      self.points_changed = true;
    }

    pub fn update_mesh( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      let mesh = self.mesh.as_mut().expect( "Mesh has not been created yet" );

      if self.points_changed
      {
        let points_buffer = mesh.get_buffer( "points" );
        let points : Vec< f32 > = self.points.iter().flat_map( | p | p.to_array() ).collect();
        gl::buffer::upload( &gl, &points_buffer, &points, gl::STATIC_DRAW );

        let b_program = mesh.get_program_mut( "body" );
        b_program.instance_count = Some( ( self.points.len() - 1 ) as u32 );
        
        let j_program = mesh.get_program_mut( "join" );
        j_program.instance_count = Some( ( self.points.len() - 2 ) as u32 );

        self.points_changed = false;
      }

      if self.join_changed
      {
        let points_buffer = mesh.get_buffer( "points" );
        let join_buffer = mesh.get_buffer( "join" );

        let ( join_geometry_list, join_geometry_count ) = self.join.geometry(); 
        gl::buffer::upload( gl, &join_buffer, &join_geometry_list, gl::STATIC_DRAW );

        let j_program = mesh.get_program( "join" );
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

        let vertex_shader =
        match self.join 
        {
          Join::Round( _ ) => d2::JOIN_ROUND_VERTEX_SHADER,
          Join::Miter => d2::JOIN_MITER_VERTEX_SHADER,
          Join::Bevel => d2::JOIN_BEVEL_VERTEX_SHADER
        };

        let vertex_shader = gl::ShaderSource::former()
        .shader_type( gl::VERTEX_SHADER )
        .source( vertex_shader )
        .compile( &gl )?;
        let join_program = gl::ProgramShaders::new( &vertex_shader, j_program.fragment_shader.as_ref().expect( "Fragment shader has not been set" ) ).link( &gl )?;
        j_program.copy_uniforms_to( gl, &join_program )?;

        let j_program = mesh.get_program_mut( "join" );

        j_program.delete_vertex_shader( gl );
        j_program.delete_program( gl );
        j_program.delete_vao( gl );

        j_program.vao = vao;
        j_program.draw_mode = gl::TRIANGLE_FAN;
        j_program.vertex_shader = Some( vertex_shader );
        j_program.program = Some( join_program );
        j_program.instance_count = Some( ( self.points.len() - 2 ) as u32 );
        j_program.vertex_count = join_geometry_count as u32;

        self.join_changed = false;
      }

      if self.cap_changed
      {
        let cap_buffer = mesh.get_buffer( "cap" );
        let ( cap_geometry_list, cap_geometry_count ) = self.cap.geometry();
        gl::buffer::upload( gl, &cap_buffer, &cap_geometry_list, gl::STATIC_DRAW );

        let c_program = mesh.get_program( "cap" );

        let vao = gl.create_vertex_array();
        gl.bind_vertex_array( vao.as_ref() );
        match self.cap
        {
          Cap::Round( _ ) =>
          {
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &cap_buffer )?;
          },
          Cap::Square =>
          {
            gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &cap_buffer )?;
          }
          _ => {}
        }

        let ( vertex_shader, cap_draw_mode ) =
        match self.cap
        {
          Cap::Round( _ ) =>( d2::CAP_ROUND_VERTEX_SHADER, gl::TRIANGLE_FAN ),
          Cap::Square =>( d2::CAP_SQUARE_VERTEX_SHADER, gl::TRIANGLES ),
          _ => ( d2::CAP_BUTT_VERTEX_SHADER, gl::TRIANGLES )
        };

        let vertex_shader = gl::ShaderSource::former()
        .shader_type( gl::VERTEX_SHADER )
        .source( vertex_shader )
        .compile( &gl )?;
        let cap_program = gl::ProgramShaders::new( &vertex_shader, c_program.fragment_shader.as_ref().expect( "Fragment shader has not been set" ) ).link( &gl )?;
        mesh.get_program( "join" ).copy_uniforms_to( gl, &cap_program )?;
        c_program.copy_uniforms_to( gl, &cap_program )?;

        let c_program = mesh.get_program_mut( "cap" );

        c_program.delete_vertex_shader( gl );
        c_program.delete_program( gl );
        c_program.delete_vao( gl );

        c_program.vao = vao;
        c_program.vertex_shader = Some( vertex_shader );
        c_program.program = Some( cap_program );
        c_program.instance_count = None;
        c_program.vertex_count = cap_geometry_count as u32;
        c_program.draw_mode = cap_draw_mode;

        self.cap_changed = false;
      }

      Ok( () )
    }

    pub fn draw( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      self.update_mesh( gl )?;

      let mesh = self.mesh.as_ref().expect( "Mesh has not been created yet" );
      mesh.draw( gl, "body" );
      mesh.draw( gl, "join" );

      match self.cap
      {
        Cap::Round( _ ) => 
        {
          mesh.upload_to( gl, "cap", "u_inPoint", self.points[ 0 ].as_slice() )?;
          mesh.draw( gl, "cap" );

          mesh.upload_to( gl, "cap", "u_inPoint", self.points[ self.points.len() - 1 ].as_slice() )?;
          mesh.draw( gl, "cap" );
        },
        Cap::Square =>
        {
          mesh.upload_to( gl, "cap", "u_inPointA", self.points[ 1 ].as_slice() )?;
          mesh.upload_to( gl, "cap", "u_inPointB", self.points[ 0 ].as_slice() )?;
          mesh.draw( gl, "cap" );

          let len = self.points.len();

          mesh.upload_to( gl, "cap", "u_inPointA", self.points[ len - 2 ].as_slice() )?;
          mesh.upload_to( gl, "cap", "u_inPointB", self.points[ len - 1 ].as_slice() )?;
          mesh.draw( gl, "cap" );
        },
        _ => {}
      }

      Ok( () )
    }

    pub fn get_mesh( &self ) -> &Mesh
    {
      self.mesh.as_ref().expect( "Mesh has not been created yet" )
    }   

    pub fn get_mesh_mut( &mut self ) -> &mut Mesh
    {
      self.mesh.as_mut().expect( "Mesh has not been created yet" )
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