mod private
{
  use std::collections::HashMap;

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
    join_changed : bool
  }

  impl Line
  {
    pub fn create_mesh( &mut self, gl : &gl::WebGl2RenderingContext, fragment_shader : &str ) -> Result< (), gl::WebglError >
    {
      let points : Vec< f32 > = self.points.iter().flat_map( | p | p.to_array() ).collect();
      let ( join_geometry_list, join_geometry_count ) = self.join.geometry(); 
      let ( cap_geometry_list, cap_geometry_count ) = self.cap.geometry();

      // Buffers
      let body_buffer = gl.create_buffer().expect( "Failed to create a buffer" );
      let body_instanced_buffer = gl.create_buffer().expect( "Failed to create a instanced_buffer" );
      let join_instanced_buffer = gl.create_buffer().expect( "Failed to create a join_instanced_buffer" );
      let cap_instanced_buffer = gl.create_buffer().expect( "Failed to create a cap_instanced_buffer" );

      gl.bind_buffer( gl::ARRAY_BUFFER, Some( &body_instanced_buffer ) );
      gl::buffer::upload( gl, &body_instanced_buffer, &helpers::BODY_GEOMETRY, gl::STATIC_DRAW );

      gl.bind_buffer( gl::ARRAY_BUFFER, Some( &body_buffer ) );
      gl::buffer::upload( &gl, &body_buffer, &points, gl::STATIC_DRAW );

      gl.bind_buffer( gl::ARRAY_BUFFER, Some( &join_instanced_buffer ) );
      gl::buffer::upload( &gl, &join_instanced_buffer, &join_geometry_list, gl::STATIC_DRAW );

      gl.bind_buffer( gl::ARRAY_BUFFER, Some( &cap_instanced_buffer ) );
      gl::buffer::upload( &gl, &cap_instanced_buffer, &cap_geometry_list, gl::STATIC_DRAW );

      

      // Body vao
      let body_vao = gl.create_vertex_array();
      gl.bind_vertex_array( body_vao.as_ref() );

      gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &body_instanced_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 1, &body_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 2, &body_buffer )?;

      let join_vao = gl.create_vertex_array();
      gl.bind_vertex_array( join_vao.as_ref() ); 

      match self.join
      {
        Join::Round( _ ) =>
        {
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &join_instanced_buffer )?;
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 1, &body_buffer )?;
        },
        Join::Miter =>
        {
          gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 3 ).divisor( 0 ).attribute_pointer( &gl, 0, &join_instanced_buffer )?;
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 1, &body_buffer )?;
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 2, &body_buffer )?;
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 4 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 3, &body_buffer )?;
        },
        Join::Bevel =>
        {
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &join_instanced_buffer )?;
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 1, &body_buffer )?;
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 2, &body_buffer )?;
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 4 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 3, &body_buffer )?;
        },
      }

      let cap_vao = gl.create_vertex_array();
      gl.bind_vertex_array( cap_vao.as_ref() ); 

      match self.cap
      {
        Cap::Round( _ ) =>
        {
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &cap_instanced_buffer )?;
        },
        Cap::Square =>
        {
          gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &cap_instanced_buffer )?;
        }
        _ => {}
      }

      // Programs
      let body_program = gl::ProgramFromSources::new( include_str!( "./shaders/body.vert" ), fragment_shader ).compile_and_link( &gl )?;
      let b_program = Program
      {
        vao : body_vao,
        program : body_program,
        draw_mode : gl::TRIANGLES,
        instance_count : Some( ( self.points.len() - 1 ) as u32 ),
        index_count : None,
        vertex_count : helpers::BODY_GEOMETRY.len() as u32
      };

      let join_vert =
      match self.join 
      {
        Join::Round( _ ) => include_str!( "./shaders/round_join.vert" ),
        Join::Miter => include_str!( "./shaders/miter_join.vert" ),
        Join::Bevel => include_str!( "./shaders/bevel_join.vert" )
      };

      let ( cap_vert, cap_draw_mode ) =
      match self.cap
      {
        Cap::Round( _ ) =>
        {
          ( include_str!( "./shaders/round_cap.vert" ), gl::TRIANGLE_FAN )
        },
        Cap::Square =>
        {
          ( include_str!( "./shaders/square_cap.vert" ), gl::TRIANGLES )
        }
        _ => { ( include_str!( "./shaders/empty.vert" ), gl::TRIANGLES ) }
      };

      let join_program = gl::ProgramFromSources::new( join_vert, fragment_shader ).compile_and_link( &gl )?;
      let cap_program = gl::ProgramFromSources::new( cap_vert, fragment_shader ).compile_and_link( &gl )?;

      let j_program = Program
      {
        vao : join_vao,
        program : join_program,
        draw_mode : gl::TRIANGLE_FAN,
        instance_count : Some( ( self.points.len() - 2 ) as u32 ),
        index_count : None,
        vertex_count : join_geometry_count as u32
      };

      let c_program = Program
      {
        vao : cap_vao,
        program : cap_program,
        draw_mode : cap_draw_mode,
        instance_count : None,
        index_count : None,
        vertex_count : cap_geometry_count as u32
      };

      let mut mesh = Mesh::default();
      mesh.add_program( "cap", c_program );
      mesh.add_program( "join", j_program );
      mesh.add_program( "body", b_program );

      mesh.add_buffer( "body", body_instanced_buffer );
      mesh.add_buffer( "cap", cap_instanced_buffer );
      mesh.add_buffer( "join", join_instanced_buffer );
      mesh.add_buffer( "points", body_buffer );

      self.mesh = Some( mesh );

      Ok( () )
    }

    pub fn set_join( &mut self, join : Join )
    {
      self.join = join;
      self.join_changed = true;
    }

    pub fn update_mesh( &self )
    {
      let mesh = self.get_mesh();

      if self.join_changed
      {
        let join_buffer = mesh.get_buffer( "join" );
        let join_program = mesh.get_program( "join" );

        let vao = join_program.vao;

        self.join_changed = false;
      }
    }

    pub fn draw( &self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
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
  }

  #[ derive( Debug, Clone, Default ) ]
  pub struct LineMerged
  {
    pub points : Vec< math::F32x2 >,
    mesh : Option< Mesh >
  }

  impl LineMerged
  {
    pub fn create_mesh( &mut self, gl : &gl::WebGl2RenderingContext, segments : u32, fragment_shader : &str ) -> Result< (), gl::WebglError >
    {
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


      let points : Vec< f32 > = self.points.iter().flat_map( | p | p.to_array() ).collect();

      let body_count = body_geometry.len();
      let circle_left_half_count = circle_left_half_geometry.len();
      let circle_right_half_count = circle_right_half_geometry.len();

      let vertex_count = body_count + circle_left_half_count + circle_right_half_count;

      let mut geometry = Vec::new();
      geometry.extend_from_slice( &circle_left_half_geometry );
      geometry.extend_from_slice( &body_geometry );
      geometry.extend_from_slice( &circle_right_half_geometry );


      let buffer = gl.create_buffer().expect( "Failed to create a buffer" );
      let instanced_buffer = gl.create_buffer().expect( "Failed to create a instanced_buffer" );

      gl::buffer::upload( gl, &buffer, &points, gl::DYNAMIC_DRAW );
      gl::buffer::upload( gl, &instanced_buffer, &geometry, gl::STATIC_DRAW );

      let vao = gl.create_vertex_array();
      gl.bind_vertex_array( vao.as_ref() );

      gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).divisor( 0 ).attribute_pointer( gl, 0, &instanced_buffer )?;
      gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 2 ).offset( 0 ).divisor( 1 ).attribute_pointer( gl, 1, &buffer )?;
      gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 2 ).offset( 2 ).divisor( 1 ).attribute_pointer( gl, 2, &buffer )?;

      let program = gl::ProgramFromSources::new( include_str!( "./shaders/merged.vert" ), fragment_shader ).compile_and_link( gl )?;
      let program = Program
      {
        vao : vao,
        program : program,
        draw_mode : gl::TRIANGLES,
        instance_count : Some( ( self.points.len() - 1 ) as u32 ),
        index_count : None,
        vertex_count : vertex_count as u32
      };

      let mut mesh = Mesh::default();
      mesh.add_program( "body", program );

      self.mesh = Some( mesh );

      Ok( () )
    }

    pub fn draw( &self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
    {
      let mesh = self.mesh.as_ref().expect( "Mesh has not been created yet" );
      mesh.draw( gl, "body" );

      Ok( () )
    }

    pub fn get_mesh( &self ) -> &Mesh
    {
      self.mesh.as_ref().expect( "Mesh has not been created yet" )
    } 
  }
  
}

crate::mod_interface!
{

  orphan use
  {
    Line,
    LineMerged
  };
}