//! `ArrayBuffer` against a live WebGL2 context: what reaches the GPU, and when.
//! `webgl_helpers_test.rs` checks the GL-free `StagedVec` bookkeeping on the
//! host; this suite checks that `flush` uploads what that bookkeeping asks for,
//! that `gpu_len` counts only uploaded elements, and that a batch's staged
//! writes reach the GPU even when the stream that made them failed.
//!
//! wasm32-only for the same reason as `webgl_camera_test.rs`: the subject needs
//! a live `WebGl2RenderingContext`, which only exists under the workspace's
//! headless-browser wasm32 runner.

#![ cfg( all( target_arch = "wasm32", feature = "adapter-webgl" ) ) ]

mod helpers;

use helpers::webgl::gl_init;
use minwebgl as gl;
use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::adapters::webgl::webgl_helpers::ArrayBuffer;
use tilemap_renderer::assets::{ Assets, ImageAsset, ImageSource, PixelFormat, SpriteAsset };
use tilemap_renderer::backend::Backend;
use tilemap_renderer::commands::
{
  AddSpriteInstance,
  BindBatch,
  Clear,
  CreateSpriteBatch,
  DrawBatch,
  RemoveInstance,
  RenderCommand,
  SpriteBatchParams,
};
use tilemap_renderer::types::{ BlendMode, MipmapMode, RenderConfig, ResourceId, SamplerFilter, Transform, WrapMode };
use wasm_bindgen::JsCast;
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

/// A stream that binds a batch, adds an instance and then fails leaves the
/// batch bound with its write only staged. The next `submit` unbinds it the
/// way the missing `UnbindBatch` would, flushing the write, so a `DrawBatch`
/// shows the instance. If the recovery only forgot the binding, the batch
/// would draw nothing (its `gpu_len` still 0) until it was next bound.
#[ wasm_bindgen_test ]
fn a_batch_left_bound_by_a_failed_stream_is_flushed_by_the_next_submit()
{
  const SIZE : u32 = 16;
  let gl = gl_init();
  let canvas = gl.canvas().unwrap().dyn_into::< web_sys::HtmlCanvasElement >().unwrap();
  canvas.set_width( SIZE );
  canvas.set_height( SIZE );
  let mut backend = WebGlBackend::new( RenderConfig { width : SIZE, height : SIZE, ..RenderConfig::default() }, gl.clone() ).unwrap();
  backend.assets_load( &Assets
  {
    fonts : Vec::new(),
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap { bytes : vec![ 255; 4 ], width : 1, height : 1, format : PixelFormat::Rgba8 },
      filter : SamplerFilter::Nearest,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
      premultiplied : false,
    }],
    sprites : vec![ SpriteAsset { id : ResourceId::new( 0 ), sheet : ResourceId::new( 0 ), region : [ 0.0, 0.0, 1.0, 1.0 ] } ],
    geometries : Vec::new(),
    gradients : Vec::new(),
    patterns : Vec::new(),
    clip_masks : Vec::new(),
    paths : Vec::new(),
  }).unwrap();

  let batch = ResourceId::new( 0 );
  let params = SpriteBatchParams
  {
    transform : Transform::default(),
    sheet : ResourceId::new( 0 ),
    blend : BlendMode::Normal,
    clip : None,
    alpha_clip : 0.0,
    occlude_overlap : false,
  };
  let instance = Transform { position : [ 4.0, 4.0 ], scale : [ 8.0, 4.0 ], ..Transform::default() };
  let failed = backend.submit( &
  [
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch { batch, params } ),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddSpriteInstance( AddSpriteInstance { transform : instance, sprite : ResourceId::new( 0 ), tint : [ 1.0; 4 ] } ),
    RenderCommand::RemoveInstance( RemoveInstance { index : 5 } ),
  ]);
  assert!( failed.is_err(), "removing past the batch's end fails the stream" );

  backend.submit( &
  [
    RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 0.0, 1.0 ] } ),
    RenderCommand::DrawBatch( DrawBatch { batch } ),
  ]).unwrap();
  let mut bytes = vec![ 0_u8; ( SIZE * SIZE * 4 ) as usize ];
  gl.read_pixels_with_opt_u8_array( 0, 0, SIZE as i32, SIZE as i32, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut bytes ) ).unwrap();
  let white = bytes.as_chunks::< 4 >().0.iter().filter( | p | p[ .. 3 ] == [ 255, 255, 255 ] ).count();
  assert_eq!( white, 8 * 4, "the instance the failed stream added is drawn" );
}
