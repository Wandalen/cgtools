//! Touch input test example.
//!
//! Drag with one finger to pan the square.
//! Pinch with two fingers to zoom in/out.
//! On desktop: left-click drag to pan, scroll wheel to zoom.

#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::implicit_return ) ]

use minwebgl as gl;
use gl::GL;
use browser_input::{ EventType, Input, CLIENT };
use gl::JsCast as _;

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  let vert_src = include_str!( "../shaders/shader.vert" );
  let frag_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vert_src, frag_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // A square made of two triangles (TRIANGLE_STRIP)
  let vertices : [ f32; 8 ] =
  [
    -0.15, -0.15,
     0.15, -0.15,
    -0.15,  0.15,
     0.15,  0.15,
  ];

  let buf = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &buf, &vertices, GL::STATIC_DRAW );

  let a_pos = gl.get_attrib_location( &program, "a_position" ) as u32;
  gl.enable_vertex_attrib_array( a_pos );
  gl.bind_buffer( GL::ARRAY_BUFFER, Some( &buf ) );
  gl.vertex_attrib_pointer_with_i32( a_pos, 2, GL::FLOAT, false, 0, 0 );

  let u_offset = gl.get_uniform_location( &program, "u_offset" );
  let u_scale = gl.get_uniform_location( &program, "u_scale" );
  let u_color = gl.get_uniform_location( &program, "u_color" );

  let mut input = Input::new
  (
    Some( canvas.clone().dyn_into().unwrap() ),
    CLIENT,
  ).expect( "Failed to initialize input" );

  // Mutable state captured by move into the closure
  let mut offset = [ 0.0_f32, 0.0 ];
  let mut scale = 1.0_f32;
  // Last single-finger contact: (pointer_id, x_px, y_px)
  let mut last_single : Option< ( i32, f32, f32 ) > = None;
  // Distance between two fingers on the previous frame
  let mut last_pinch_dist : Option< f32 > = None;

  let update_and_draw = move | _t : f64 |
  {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;

    input.update_state();

    // Wheel zoom on desktop
    for browser_input::Event { event_type, .. } in input.event_queue().iter()
    {
      if let EventType::Wheel( delta ) = *event_type
      {
        // delta_y > 0 → scroll down → zoom out
        scale = ( scale * ( 1.0 - delta.0[ 1 ] as f32 * 0.001 ) ).clamp( 0.05, 10.0 );
      }
    }

    input.clear_events();

    // Pan and pinch via active_pointers
    let active = input.active_pointers().to_vec();

    if active.len() == 1
    {
      let ( pid, pos ) = active[ 0 ];
      let px = pos.0[ 0 ] as f32;
      let py = pos.0[ 1 ] as f32;

      if let Some( ( lpid, lx, ly ) ) = last_single
      {
        if lpid == pid
        {
          // Convert pixel delta to NDC delta
          offset[ 0 ] = ( offset[ 0 ] + ( px - lx ) / w * 2.0 ).clamp( -2.0, 2.0 );
          offset[ 1 ] = ( offset[ 1 ] - ( py - ly ) / h * 2.0 ).clamp( -2.0, 2.0 );
        }
      }
      last_single = Some( ( pid, px, py ) );
      last_pinch_dist = None;
    }
    else if active.len() >= 2
    {
      let ( _, p0 ) = active[ 0 ];
      let ( _, p1 ) = active[ 1 ];
      let dx = p1.0[ 0 ] as f32 - p0.0[ 0 ] as f32;
      let dy = p1.0[ 1 ] as f32 - p0.0[ 1 ] as f32;
      let dist = ( dx * dx + dy * dy ).sqrt();

      if let Some( ld ) = last_pinch_dist
      {
        if ld > 1.0
        {
          scale = ( scale * ( dist / ld ) ).clamp( 0.05, 10.0 );
        }
      }
      last_pinch_dist = Some( dist );
      last_single = None;
    }
    else
    {
      last_single = None;
      last_pinch_dist = None;
    }

    // Color shifts with scale for visual feedback
    let r = ( 0.2 + scale * 0.1 ).clamp( 0.0, 1.0 );
    let g = 0.6_f32;
    let b = ( 1.0 - scale * 0.05 ).clamp( 0.2, 1.0 );

    gl::uniform::upload( &gl, u_offset.clone(), &offset ).unwrap();
    gl::uniform::upload( &gl, u_scale.clone(), &scale ).unwrap();
    gl::uniform::upload( &gl, u_color.clone(), &[ r, g, b ] ).unwrap();

    gl.clear_color( 0.07, 0.07, 0.10, 1.0 );
    gl.clear( GL::COLOR_BUFFER_BIT );
    gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );

    true
  };

  gl::exec_loop::run( update_and_draw );
  Ok( () )
}

fn main()
{
  run().unwrap()
}
