//! Just draw a large point in the middle of the screen.

use minwebgl as gl;
use gl::{ GL, math::nd, math::nd::array, DebugLog };
use std::
{
  cell::RefCell,
  rc::Rc,
};

// qqq : make usecase more and picture more impressive changing code minimally

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let position_data :  [ f32 ; 36 ] =
  [
    // 12x3x2 position
     -0.6, -0.4, -0.6, -0.5, -0.65, -0.35, // Triangle 5
     -0.4,  0.3, -0.35, 0.4, -0.3,  0.25,  // Triangle 3
     -0.1, -0.1,  0.0,  0.2,  0.0, -0.15,  // Triangle 1
      0.1, -0.3,  0.15, -0.1, 0.05, -0.25, // Triangle 6
      0.3, -0.2,  0.25, 0.1,  0.2,  0.05,  // Triangle 2
      0.5,  0.5,  0.45, 0.6,  0.55, 0.6,   // Triangle 4
  ];

  // Vertex data
  let color_data : [ f32 ; 18 ] =
  [
    // color 2x6x3
    0.9849, 0.0600, 0.0662, 0.1232, 0.9332, 0.4260, 0.6969, 0.5353, 0.1471,
    0.2899, 0.9056, 0.7799, 0.2565, 0.6451, 0.8498, 0.0969, 0.9353, 0.0471,
  ];

  let trans_data : nd::Array< _, _ > = array!
  [
    [
      [ 1.0, 0.0 ],
      [ 0.0, 1.0 ],
      [ 0.0, -0.2 ],
    ],
    [
      [ 1.0, 0.0 ],
      [ 0.0, 1.0 ],
      [ 0.0, -0.1 ],
    ],
    [
      [ 1.0, 0.0 ],
      [ 0.0, 1.0 ],
      [ 0.0, 0.0 ],
    ],
  ];

  // Transformation matrices
  let _trans_data : [ f32 ; 18 ] =
  [

    1.0, 0.0,
    0.0, 1.0,
    0.0, -0.2,

    1.0, 0.0,
    0.0, 1.0,
    0.0, -0.1,

    1.0, 0.0,
    0.0, 1.0,
    0.0, 0.0,

  ];

  // You can use either flat array ( either static or dynamic )
  // or you can prefer nd::Array with it's flexible math.
  // The last one will save you much time on development and performance.
  assert_eq!( &_trans_data[ .. ], trans_data.as_slice().unwrap() );

  // Create buffer and upload vertex data

  let position_slot = 0;
  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, &position_data, GL::STATIC_DRAW );

  let color_slot = 1;
  let color_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &color_buffer, &color_data, GL::STATIC_DRAW );

  let trans_slot = 2;
  let trans_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &trans_buffer, &trans_data, GL::STATIC_DRAW );

  // Create vao.
  // And set attributes.
  // A divisor of 0 indicates that each vertex has its own unique attribute value.
  // A divisor of 1 means that the entire primitive shares the same attribute value.
  // A divisor of 2 or more specifies that the attribute value is shared across multiple primitives.

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32 ; 2 ] >().stride( 2 ).offset( 0 ).divisor( 0 )
  .attribute_pointer( &gl, position_slot, &position_buffer )?;
  gl::BufferDescriptor::new::< [ f32 ; 3 ] >().stride( 3 ).offset( 0 ).divisor( 2 )
  .attribute_pointer( &gl, color_slot, &color_buffer )?;
  gl::BufferDescriptor::new::< [ [ f32 ; 2 ] ; 3 ] >().stride( 3*2 ).offset( 0 ).divisor( 1 )
  .attribute_pointer( &gl, trans_slot, &trans_buffer )?;
  gl.bind_vertex_array( None );

  // xxx
  // Prepare to change transformation every frame
  // std140 alignment require to allocate 4 words for the first row and 4 for the second row.
  let trans = vec!
  [
    1.0_f32, 0.0_f32, 0.0_f32, 0.0_f32,
    0.0_f32, 1.0_f32, 0.0_f32, 0.0_f32,
  ];
  let trans = Rc::new( RefCell::new( trans ) );

  // Transformation
  let trans_buffer = gl::buffer::create( &gl )?;
  let trans_block_index : u32 = gl.get_uniform_block_index( &program, "TransformBlock" );
  let trans_block_point = 1;
  gl.uniform_block_binding( &program, trans_block_index, trans_block_point );

  // Retrieve UBO information for diagnostic purposes only; these lines should be removed in production builds.
  gl::ubo::diagnostic_info( &gl, &program, trans_block_index ).debug_info();

  // Define the update and draw logic
  let update_and_draw =
  {
    let gl = gl.clone();
    let trans = trans.clone();
    move | t : f64 |
    {

      // Update transformation
      let mut trans = trans.borrow_mut();
      let t2 = t / 1000.0;
      let ct = t2.cos();
      let st = t2.sin();
      // std140 alignment require to allocate 4 words for the first row and 4 for the second row.
      trans[ 0 ] = ct as _;
      trans[ 1 ] = st as _;
      trans[ 4 ] = -st as _;
      trans[ 5 ] = ct as _;
      gl::ubo::upload( &gl, &trans_buffer, trans_block_point, &trans[..], GL::DYNAMIC_DRAW );

      // Bind VAO and draw

      gl.bind_vertex_array( Some( &vao ) );
      // gl.draw_arrays( GL::TRIANGLES, 0, 3*4 );
      gl.draw_arrays_instanced( GL::TRIANGLES, 0, 3*6, 3 );
      gl.bind_vertex_array( None );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );
  Ok(())
}

fn main()
{
  run().unwrap()
}
