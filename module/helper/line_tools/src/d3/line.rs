mod private
{
  use crate::*;
  use minwebgl::{self as gl, IntoArray};
  use ndarray_cg as math;

  #[ derive( Debug, Clone, Default ) ]
  pub struct Line
  {
    pub points : Vec< math::F32x3 >,
    mesh : Option< Mesh >
  }
  impl Line
  {
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
      gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).divisor( 1 ).attribute_pointer( gl, 1, &buffer )?;
      gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 3 ).divisor( 1 ).attribute_pointer( gl, 2, &buffer )?;

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
    Line
  };
}