//! `Renderer` driving a live `WebGlBackend`, with pixel read-back: the host
//! contract around the GPU camera. A pan changes no command, so the render
//! after it is an idle replay, and the host must hand the backend the new view
//! matrix with `camera_set` for the frame to move. `renderer_cache_test.rs`
//! checks the replay itself on the host; `tilemap_renderer`'s
//! `webgl_camera_test.rs` checks `camera_set` on the backend alone.
//!
//! wasm32-only: the subject needs a live `WebGl2RenderingContext`, which only
//! exists under the workspace's headless-browser wasm32 runner. The crate's
//! wasm32 dev-dependencies add `tilemap_renderer`'s `adapter-webgl`.

#![ cfg( target_arch = "wasm32" ) ]

mod common;

use alloc::sync::Arc;
use minwebgl as gl;
use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::assets::{ ImageSource, PixelFormat };
use tilemap_renderer::backend::Backend;
use tilemap_renderer::commands::RenderCommand;
use tilemap_renderer::types::RenderConfig;
use tilemap_scene::{ AssetKind, AssetResolver, Camera, CompileError, Placement, Renderer, Scene };
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

extern crate alloc;

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

/// Canvas and viewport size.
const SIZE : u32 = 128;

/// Resolves every asset to an opaque white bitmap of two 72×64 atlas tiles, so
/// a sprite covers its whole quad and loads synchronously.
struct WhiteAtlas;

impl AssetResolver for WhiteAtlas
{
  fn resolve( &self, _asset_id : &str, _path : &str, _kind : &AssetKind ) -> Result< ImageSource, CompileError >
  {
    Ok( ImageSource::Bitmap { bytes : vec![ 255; 144 * 64 * 4 ], width : 144, height : 64, format : PixelFormat::Rgba8 } )
  }
}

/// The first and one-past-last canvas columns holding a white pixel.
fn white_columns( gl : &gl::GL ) -> ( usize, usize )
{
  let mut bytes = vec![ 0_u8; ( SIZE * SIZE * 4 ) as usize ];
  gl.read_pixels_with_opt_u8_array( 0, 0, SIZE as i32, SIZE as i32, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut bytes ) ).unwrap();
  let pixels = bytes.as_chunks::< 4 >().0;
  let columns : Vec< usize > = ( 0..SIZE as usize )
    .filter( | x | ( 0..SIZE as usize ).any( | y | pixels[ y * SIZE as usize + x ][ .. 3 ] == [ 255, 255, 255 ] ) )
    .collect();
  ( columns.first().copied().unwrap_or( 0 ), columns.last().map_or( 0, | x | x + 1 ) )
}

/// Whether `commands` edit a batch rather than only drawing.
fn edits_batches( commands : &[ RenderCommand ] ) -> bool
{
  commands.iter().any( | c | matches!
  (
    c,
    RenderCommand::CreateSpriteBatch( _ ) | RenderCommand::AddSpriteInstance( _ ) | RenderCommand::SetSpriteInstance( _ )
      | RenderCommand::RemoveInstance( _ ) | RenderCommand::DeleteBatch( _ )
  ))
}

/// One 72×64 grass tile at the origin, seen through a camera centred on it and
/// then panned 20 px right. The pan is an idle replay that only draws, so the
/// frame follows it only once the host calls `camera_set` with the new view
/// matrix: before that call the tile stays where the old matrix put it.
#[ wasm_bindgen_test ]
fn a_pan_is_replayed_and_the_host_moves_it_with_camera_set()
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make().unwrap();
  let gl = gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap();
  let canvas = gl.canvas().unwrap().dyn_into::< web_sys::HtmlCanvasElement >().unwrap();
  canvas.set_width( SIZE );
  canvas.set_height( SIZE );
  let mut backend = WebGlBackend::new( RenderConfig { width : SIZE, height : SIZE, ..RenderConfig::default() }, gl.clone() ).unwrap();

  let spec = common::compile::minimal_spec();
  let mut renderer = Renderer::new( &spec, &WhiteAtlas ).expect( "renderer" );
  renderer.capabilities_set( &backend.capabilities() );
  backend.assets_load( renderer.assets() ).unwrap();
  let mut scene = Scene::new( Arc::new( spec ) );
  let grass = scene.object( "grass" ).expect( "grass" );
  scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );

  let centred = Camera { world_center : ( 0.0, 0.0 ), zoom : 1.0, viewport_size : ( SIZE, SIZE ) };
  backend.camera_set( centred.to_view_mat3() );
  let first = renderer.render( &scene, &centred ).expect( "render" ).to_vec();
  assert!( edits_batches( &first ), "the first frame builds its batch" );
  backend.submit( &first ).unwrap();
  assert_eq!( white_columns( &gl ), ( 28, 100 ), "the tile centred on the canvas" );

  let panned = Camera { world_center : ( 20.0, 0.0 ), ..centred };
  let replay = renderer.render( &scene, &panned ).expect( "render" ).to_vec();
  assert!( !edits_batches( &replay ), "a pan changes no command: an idle replay that only draws" );
  backend.submit( &replay ).unwrap();
  assert_eq!( white_columns( &gl ), ( 28, 100 ), "without camera_set the backend keeps the old view" );

  backend.camera_set( panned.to_view_mat3() );
  let replay = renderer.render( &scene, &panned ).expect( "render" ).to_vec();
  backend.submit( &replay ).unwrap();
  assert_eq!( white_columns( &gl ), ( 8, 80 ), "with camera_set the tile moves 20 px left" );
}
