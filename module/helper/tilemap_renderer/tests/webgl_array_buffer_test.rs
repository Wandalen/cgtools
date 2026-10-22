//! `ArrayBuffer` against a live WebGL2 context: what reaches the GPU, and when.
//! `webgl_helpers_test.rs` checks the GL-free `StagedVec` bookkeeping on the
//! host; this suite checks that `flush` uploads what that bookkeeping asks for
//! and that `gpu_len` counts only uploaded elements.
//!
//! wasm32-only for the same reason as `webgl_camera_test.rs`: the subject needs
//! a live `WebGl2RenderingContext`, which only exists under the workspace's
//! headless-browser wasm32 runner.

#![ cfg( all( target_arch = "wasm32", feature = "adapter-webgl" ) ) ]

mod helpers;

use helpers::webgl::gl_init;
use minwebgl as gl;
use tilemap_renderer::adapters::webgl::webgl_helpers::ArrayBuffer;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

/// The first `count` elements of `buffer`'s GPU copy.
fn gpu_contents( gl : &gl::GL, buffer : &ArrayBuffer< [ f32; 2 ] >, count : usize ) -> Vec< [ f32; 2 ] >
{
  let mut bytes = vec![ 0_u8; count * 8 ];
  gl.bind_buffer( gl::ARRAY_BUFFER, Some( buffer.buffer() ) );
  gl.get_buffer_sub_data_with_i32_and_u8_array( gl::ARRAY_BUFFER, 0, &mut bytes );
  gl.bind_buffer( gl::ARRAY_BUFFER, None );
  bytes.as_chunks::< 8 >().0.iter().map( | c |
    [ f32::from_le_bytes( [ c[ 0 ], c[ 1 ], c[ 2 ], c[ 3 ] ] ), f32::from_le_bytes( [ c[ 4 ], c[ 5 ], c[ 6 ], c[ 7 ] ] ) ]
  ).collect()
}

/// Writes are staged until `flush`, and `gpu_len` counts what the last flush
/// left on the GPU, which is what a batch draws. Three pushes into a buffer
/// of two grow it on the flush; a swap-remove then moves the last element
/// into the hole, and the GPU still holds three until the next flush. A draw
/// count taken from `len` before the flush would read elements the GPU does
/// not have yet.
#[ wasm_bindgen_test ]
fn array_buffer_gpu_len_counts_only_flushed_elements()
{
  let gl = gl_init();
  let mut buffer = ArrayBuffer::< [ f32; 2 ] >::new( &gl, 2 ).unwrap();
  for i in 0..3_u8 { buffer.push( &[ f32::from( i ), 0.5 ] ).unwrap(); }
  assert_eq!( ( buffer.len(), buffer.gpu_len() ), ( 3, 0 ), "pushes are only staged" );

  buffer.flush().unwrap();
  assert_eq!( buffer.gpu_len(), 3, "the flush grows the buffer and uploads all three" );
  assert_eq!( gpu_contents( &gl, &buffer, 3 ), vec![ [ 0.0, 0.5 ], [ 1.0, 0.5 ], [ 2.0, 0.5 ] ] );

  assert_eq!( buffer.swap_remove( 0 ), 2 );
  assert_eq!( ( buffer.len(), buffer.gpu_len() ), ( 2, 3 ), "the remove is only staged" );
  buffer.flush().unwrap();
  assert_eq!( buffer.gpu_len(), 2 );
  assert_eq!( gpu_contents( &gl, &buffer, 2 ), vec![ [ 2.0, 0.5 ], [ 1.0, 0.5 ] ], "the last element moved into the hole" );
}
