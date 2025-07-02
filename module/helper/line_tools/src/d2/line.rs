mod private
{
  use crate::*;
  use minwebgl as gl;
  use ndarray_cg as math;

  #[ derive( Debug, Clone, Default ) ]
  pub struct Line
  {
    pub points : Vec< math::F32x2 >,
    pub cap : Cap,
    pub join : Join
  }
  impl Line
  {
    pub fn to_mesh( &self, gl : &gl::WebGl2RenderingContext, fragment_shder : &str ) -> Result< Mesh, gl::WebglError >
    {
      let points : Vec< f32 > = self.points.iter().flat_map( | p | p.to_array() ).collect();
      let ( join_geometry_list, join_geometry_count ) = self.join.geometry(); 

      // Buffers
      let body_buffer = gl.create_buffer().expect( "Failed to create a buffer" );
      let body_instanced_buffer = gl.create_buffer().expect( "Failed to create a instanced_buffer" );
      let join_instanced_buffer = gl.create_buffer().expect( "Failed to create a join_instanced_buffer" );

      gl.bind_buffer( gl::ARRAY_BUFFER, Some( &body_instanced_buffer ) );
      gl::buffer::upload( gl, &body_instanced_buffer, &BODY_GEOMETRY, gl::STATIC_DRAW );

      gl.bind_buffer( gl::ARRAY_BUFFER, Some( &body_buffer ) );
      gl::buffer::upload( &gl, &body_buffer, &points, gl::STATIC_DRAW );

      gl.bind_buffer( gl::ARRAY_BUFFER, Some( &join_instanced_buffer ) );
      gl::buffer::upload( &gl, &join_instanced_buffer, &join_geometry_list, gl::STATIC_DRAW );

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

      // Programs
      let body_program = gl::ProgramFromSources::new( include_str!( "./shaders/body.vert" ), fragment_shder ).compile_and_link( &gl )?;
      let b_program = Program
      {
        vao : body_vao,
        program : body_program,
        draw_mode : gl::TRIANGLES,
        instance_count : Some( ( self.points.len() - 1 ) as u32 ),
        index_count : None,
        vertex_count : BODY_GEOMETRY.len() as u32
      };

      let join_vert =
      match self.join 
      {
        Join::Round( _ ) => include_str!( "./shaders/round_join.vert" ),
        Join::Miter => include_str!( "./shaders/miter_join.vert" ),
        Join::Bevel => include_str!( "./shaders/bevel_join.vert" )
      };

      let join_program = gl::ProgramFromSources::new( join_vert, fragment_shder ).compile_and_link( &gl )?;

      let j_program = Program
      {
        vao : join_vao,
        program : join_program,
        draw_mode : gl::TRIANGLE_FAN,
        instance_count : Some( ( self.points.len() - 2 ) as u32 ),
        index_count : None,
        vertex_count : join_geometry_count as u32
      };

      let mut mesh = Mesh::default();
      mesh.add_program( "join", j_program );
      mesh.add_program( "body", b_program );

      Ok( mesh )
    }    
  }


  const BODY_GEOMETRY : [ [ f32; 2 ]; 6 ] =
  [
    [ 0.0, -0.5 ],
    [ 1.0, -0.5 ],
    [ 1.0,  0.5 ],
    [ 0.0, -0.5 ],
    [ 1.0,  0.5 ],
    [ 0.0,  0.5 ]
  ];

}

crate::mod_interface!
{

  orphan use
  {
    Line
  };
}