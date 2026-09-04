//! Draws 5+5 points via two independent VAOs, demonstrating switching between vertex
//! configurations with a single binding call (`gl.bind_vertex_array`) per WebGL2's VAO
//! state-encapsulation model.

use minwebgl as gl;
use gl::{ GL };

/// One point's position/point-size/color record — `stride( 6 )` below covers all 6 `f32`
/// fields, matching this struct's own ( `repr( C )`, no padding ) byte layout.
#[ repr( C ) ]
#[ derive( Debug, Default, Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
struct Vertex
{
  position : [ f32 ; 2 ],
  point_size : f32,
  color : [ f32 ; 3 ],
}

impl mingl::Attribute for Vertex
{
  fn describe() -> Vec< mingl::VertexAttribute >
  {
    vec!
    [
      mingl::VertexAttribute::new( 0, mingl::VectorDataType::new( mingl::DataType::F32, 2, 1 ), 0 ),
      mingl::VertexAttribute::new( 1, mingl::VectorDataType::new( mingl::DataType::F32, 1, 1 ), 2 ),
      mingl::VertexAttribute::new( 2, mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ), 3 ),
    ]
  }
}

fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // Vertex data ( position, point size, color )
  let vert_data : [ Vertex ; 5 ] =
  [
    Vertex { position : [ -0.6618, -0.7687 ], point_size : 200.0, color : [ 1.5849, 0.0600, 0.0662 ] },
    Vertex { position : [ -0.3149, 0.7417 ], point_size : 40.0, color : [ 0.9232, 0.9332, 0.4260 ] },
    Vertex { position : [ 0.9749, -0.8996 ], point_size : 160.0, color : [ 0.6969, 0.5353, 0.1471 ] },
    Vertex { position : [ -0.9202, -0.2956 ], point_size : 360.0, color : [ 0.2899, 0.9056, 0.7799 ] },
    Vertex { position : [ 0.4550, -0.0642 ], point_size : 80.0, color : [ 0.2565, 0.6451, 0.8498 ] },
  ];

  // Vertex data ( position, point size, color )
  let vert_data2 : [ Vertex ; 5 ] =
  [
    Vertex { position : [ 0.6192, 0.5755 ], point_size : 280.0, color : [ 0.6133, 0.8137, 0.4046 ] },
    Vertex { position : [ -0.5946, 0.7057 ], point_size : 80.0, color : [ 0.6745, 0.5229, 0.4518 ] },
    Vertex { position : [ 0.6365, 0.7236 ], point_size : 280.0, color : [ 0.4690, 0.0542, 0.7396 ] },
    Vertex { position : [ 0.8625, -0.0835 ], point_size : 80.0, color : [ 0.3708, 0.6588, 0.8611 ] },
    Vertex { position : [ 0.7997, 0.4695 ], point_size : 280.0, color : [ 0.7490, 0.3797, 0.6879 ] },
  ];

  // create buffer and upload vertex data

  let vert_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &vert_buffer, &vert_data, GL::STATIC_DRAW );

  let vert_buffer2 = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &vert_buffer2, &vert_data2, GL::STATIC_DRAW );

  // create vaos
  //
  // Fix(BUG-318): a single `vao` mixed position/point-size from `vert_buffer2` with color
  // from `vert_buffer`, silently leaving `vert_buffer`'s own position/point-size fields
  // (and a whole second draw call) unused. This crate's own readme states the demo shows
  // "switch[ing] between different vertex configurations with a single binding call" — that
  // requires two independent, self-contained VAOs, not one VAO composed from two buffers.
  // Root cause: `vert_buffer`'s attribute_pointer calls were bound against `vao` alongside
  // `vert_buffer2`'s, instead of each buffer getting its own VAO.
  // Pitfall: don't "fix" this by deleting the second dataset — the two datasets existing
  // side by side, each already a complete position+size+color record, is what shows this
  // was meant to demonstrate two configurations, not one.

  let vertex_layout = mingl::VertexBufferLayout::from_attribute::< Vertex >( 6 );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::vertex_buffer_layout_bind( &gl, &vert_buffer, &vertex_layout )?;
  gl.bind_vertex_array( None );

  let vao2 = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao2 ) );
  gl::vertex_buffer_layout_bind( &gl, &vert_buffer2, &vertex_layout )?;
  gl.bind_vertex_array( None );

  // Bind each VAO in turn and draw — switching between two independent vertex
  // configurations with a single binding call each.

  gl.bind_vertex_array( Some( &vao ) );
  gl.draw_arrays( GL::POINTS, 0, 5 );
  gl.bind_vertex_array( None );

  gl.bind_vertex_array( Some( &vao2 ) );
  gl.draw_arrays( GL::POINTS, 0, 5 );
  gl.bind_vertex_array( None );

  Ok(())
}

fn main()
{
  app_run().unwrap();
}
