//! `WebGlBackend`'s surface methods (`adapters/webgl/webgl_surfaces.rs`)
//! against a live WebGL2 context, with pixel read-back: the render-target
//! lifecycle (`render_target_create` / `_begin` / `_end` / `_with` / `_draw`,
//! `render_targets_reset`), `scissor_set`, `fill`, `image_tiled_draw` and
//! `textures_loaded`. `webgl_render_target_test.rs` checks the handle table
//! behind them on the host.
//!
//! wasm32-only for the same reason as `webgl_camera_test.rs`: the subject needs
//! a live `WebGl2RenderingContext`, which only exists under the workspace's
//! headless-browser wasm32 runner.
//!
//! Most cases count pixel values over the whole frame instead of reading
//! chosen coordinates, so they do not depend on where the backend puts the
//! origin or which way y points.

#![ cfg( all( target_arch = "wasm32", feature = "adapter-webgl" ) ) ]

mod helpers;

use helpers::empty_assets;
use helpers::webgl::gl_init;
use minwebgl as gl;
use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::assets::{ Assets, ImageAsset, ImageSource, PixelFormat, SpriteAsset };
use tilemap_renderer::backend::Backend;
use tilemap_renderer::commands::{ BindBatch, Clear, CreateSpriteBatch, RemoveInstance, RenderCommand, Sprite, SpriteBatchParams };
use tilemap_renderer::types::{ BlendMode, MipmapMode, RenderConfig, ResourceId, SamplerFilter, Transform, WrapMode };
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

/// Canvas size every frame is rendered at, pinned as in `webgl_camera_test.rs`.
const SIZE : u32 = 16;

/// Zoom by 2, no pan.
const ZOOM_2 : [ f32; 9 ] = [ 2.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0 ];

const BLACK : [ f32; 4 ] = [ 0.0, 0.0, 0.0, 1.0 ];
const RED : [ f32; 4 ] = [ 1.0, 0.0, 0.0, 1.0 ];
const GREEN : [ f32; 4 ] = [ 0.0, 1.0, 0.0, 1.0 ];

/// Image 0: a 2×2 bitmap of four colours (red, green, blue, white), repeating,
/// with sprite 0 covering it whole. Its four texels tell the tiled draw's
/// placement and orientation apart from a shifted or flipped one.
fn assets() -> Assets
{
  Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap
      {
        bytes : vec![ 255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255 ],
        width : 2,
        height : 2,
        format : PixelFormat::Rgba8,
      },
      filter : SamplerFilter::Nearest,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Repeat,
      premultiplied : false,
    }],
    sprites : vec![ SpriteAsset { id : ResourceId::new( 0 ), sheet : ResourceId::new( 0 ), region : [ 0.0, 0.0, 2.0, 2.0 ] } ],
    ..empty_assets()
  }
}

/// A fresh `SIZE`×`SIZE` canvas and a backend on it, with [`assets`] loaded.
fn backend_new() -> ( gl::GL, WebGlBackend )
{
  let gl = gl_init();
  let canvas = gl.canvas().unwrap().dyn_into::< web_sys::HtmlCanvasElement >().unwrap();
  canvas.set_width( SIZE );
  canvas.set_height( SIZE );
  let mut backend = WebGlBackend::new( RenderConfig { width : SIZE, height : SIZE, ..RenderConfig::default() }, gl.clone() ).unwrap();
  backend.assets_load( &assets() ).unwrap();
  ( gl, backend )
}

/// The canvas as RGBA pixels.
fn frame( gl : &gl::GL ) -> Vec< [ u8; 4 ] >
{
  let mut bytes = vec![ 0_u8; ( SIZE * SIZE * 4 ) as usize ];
  gl.read_pixels_with_opt_u8_array( 0, 0, SIZE as i32, SIZE as i32, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut bytes ) ).unwrap();
  bytes.as_chunks::< 4 >().0.to_vec()
}

/// How many pixels of `frame` have RGB `rgb`.
fn count( frame : &[ [ u8; 4 ] ], rgb : [ u8; 3 ] ) -> usize
{
  frame.iter().filter( | p | p[ .. 3 ] == rgb ).count()
}

/// Sprite 0 at `( x, y )`, at its own 2×2 size times `scale`.
fn sprite( x : f32, y : f32, scale : f32 ) -> RenderCommand
{
  RenderCommand::Sprite( Sprite
  {
    transform : Transform { position : [ x, y ], scale : [ scale, scale ], ..Transform::default() },
    sprite : ResourceId::new( 0 ),
    tint : [ 1.0; 4 ],
    blend : BlendMode::Normal,
    clip : None,
  })
}

/// A 4×4 cache surface baked red, then composited over the world rectangle
/// `[ 0, 0 ]`–`[ 4, 4 ]` through a 2× zoom: it must cover 8×8 canvas pixels,
/// the rectangle as the camera shows it. Composited without the camera it
/// would cover 4×4, and a bake that did not reach the surface would show
/// nothing.
#[ wasm_bindgen_test ]
fn a_baked_render_target_composites_through_the_camera()
{
  let ( gl, mut backend ) = backend_new();
  let target = backend.render_target_create( 4, 4 ).unwrap();
  assert!( backend.render_target_begin( target, Some( RED ) ) );
  backend.render_target_end();

  backend.submit( &[ RenderCommand::Clear( Clear { color : BLACK } ) ] ).unwrap();
  backend.camera_set( ZOOM_2 );
  backend.render_target_draw( target, [ 0.0, 0.0 ], [ 4.0, 4.0 ] );
  assert_eq!( count( &frame( &gl ), [ 255, 0, 0 ] ), 8 * 8 );
}

/// A composite keeps the baked image's orientation : the four-colour sprite
/// baked into a 4×4 target and composited over `[ 0, 0 ]`–`[ 4, 4 ]` gives
/// the same frame, pixel for pixel, as the sprite drawn there directly. A
/// uniform bake cannot tell a flipped composite from a correct one; four
/// distinct texels can.
#[ wasm_bindgen_test ]
fn a_composite_keeps_the_baked_orientation()
{
  let ( gl, mut backend ) = backend_new();
  let target = backend.render_target_create( 4, 4 ).unwrap();
  let baked = backend.render_target_with( target, Some( BLACK ), | b | b.submit( &[ sprite( 0.0, 0.0, 2.0 ) ] ) );
  assert!( matches!( baked, Some( Ok( () ) ) ), "the bake draws: {baked:?}" );
  backend.submit( &[ RenderCommand::Clear( Clear { color : BLACK } ) ] ).unwrap();
  backend.render_target_draw( target, [ 0.0, 0.0 ], [ 4.0, 4.0 ] );
  let composited = frame( &gl );

  let ( gl, mut backend ) = backend_new();
  backend.submit( &[ RenderCommand::Clear( Clear { color : BLACK } ), sprite( 0.0, 0.0, 2.0 ) ] ).unwrap();
  let direct = frame( &gl );

  for rgb in [ [ 255, 0, 0 ], [ 0, 255, 0 ], [ 0, 0, 255 ], [ 255, 255, 255 ] ]
  {
    assert_eq!( count( &direct, rgb ), 4, "each of the four colours covers 2×2 pixels: {rgb:?}" );
  }
  assert!( composited == direct, "the composite matches the direct draw" );
}

/// After `render_targets_reset` a kept handle is stale: `render_target_begin`
/// returns `false` and binds nothing, and `render_target_draw` draws nothing.
/// Reaching the surface that took its slot would composite the wrong cache.
#[ wasm_bindgen_test ]
fn a_stale_render_target_handle_does_nothing()
{
  let ( gl, mut backend ) = backend_new();
  let old = backend.render_target_create( 4, 4 ).unwrap();
  backend.render_targets_reset();
  let new = backend.render_target_create( 4, 4 ).unwrap();
  assert!( backend.render_target_begin( new, Some( RED ) ) );
  backend.render_target_end();

  assert!( !backend.render_target_begin( old, Some( GREEN ) ), "the old handle binds nothing" );
  backend.submit( &[ RenderCommand::Clear( Clear { color : BLACK } ) ] ).unwrap();
  backend.render_target_draw( old, [ 0.0, 0.0 ], [ 4.0, 4.0 ] );
  assert_eq!( count( &frame( &gl ), [ 0, 0, 0 ] ), ( SIZE * SIZE ) as usize, "and composites nothing" );
}

/// A target pass sets the viewport to the target's size; `render_target_end`
/// restores the canvas's. A 2×2 sprite drawn after the pass must cover 4
/// canvas pixels; under the 4×4 target's viewport it would cover 64.
#[ wasm_bindgen_test ]
fn render_target_end_restores_the_canvas_viewport()
{
  let ( gl, mut backend ) = backend_new();
  let target = backend.render_target_create( 4, 4 ).unwrap();
  assert!( backend.render_target_begin( target, Some( GREEN ) ) );
  backend.render_target_end();

  backend.submit( &[ RenderCommand::Clear( Clear { color : BLACK } ), sprite( 2.0, 2.0, 1.0 ) ] ).unwrap();
  let drawn = frame( &gl );
  assert_eq!( drawn.iter().filter( | p | p[ .. 3 ] != [ 0, 0, 0 ] ).count(), 4, "the 2×2 sprite is 2×2 canvas pixels" );
}

/// `render_target_with` restores the canvas even when the bake fails : a
/// `submit` that errors inside it must not leave the 4×4 target bound, so a 2×2
/// sprite drawn afterwards covers 4 canvas pixels, not 64. A stale handle runs
/// no bake at all.
#[ wasm_bindgen_test ]
fn render_target_with_restores_the_canvas_after_a_failed_bake()
{
  let ( gl, mut backend ) = backend_new();
  let target = backend.render_target_create( 4, 4 ).unwrap();
  // Removing past the end of an empty batch fails the stream, with the batch still bound.
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
  let baked = backend.render_target_with( target, Some( GREEN ), | b | b.submit( &
  [
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch { batch, params } ),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::RemoveInstance( RemoveInstance { index : 0 } ),
  ]));
  assert!( matches!( baked, Some( Err( _ ) ) ), "the bake ran and its submit failed: {baked:?}" );

  backend.submit( &[ RenderCommand::Clear( Clear { color : BLACK } ), sprite( 2.0, 2.0, 1.0 ) ] ).unwrap();
  let drawn = frame( &gl );
  assert_eq!( drawn.iter().filter( | p | p[ .. 3 ] != [ 0, 0, 0 ] ).count(), 4, "the canvas is bound again" );

  backend.render_targets_reset();
  let mut ran = false;
  assert!( backend.render_target_with( target, None, | _ | ran = true ).is_none() );
  assert!( !ran, "a stale handle runs no bake" );
}

/// `scissor_set` limits `fill` to its rectangle, and `None` lifts the limit.
/// Here a green fill scissored to 4×4 must leave the rest of the black canvas
/// alone; a fill after the limit is lifted covers everything.
#[ wasm_bindgen_test ]
fn a_scissored_fill_clears_only_its_rectangle()
{
  let ( gl, mut backend ) = backend_new();
  backend.submit( &[ RenderCommand::Clear( Clear { color : BLACK } ) ] ).unwrap();
  backend.scissor_set( Some( [ 2, 2, 4, 4 ] ) );
  backend.fill( GREEN );
  let scissored = frame( &gl );
  assert_eq!( count( &scissored, [ 0, 255, 0 ] ), 16, "only the 4×4 rectangle is filled" );
  assert_eq!( count( &scissored, [ 0, 0, 0 ] ), ( SIZE * SIZE - 16 ) as usize );

  backend.scissor_set( None );
  backend.fill( RED );
  assert_eq!( count( &frame( &gl ), [ 255, 0, 0 ] ), ( SIZE * SIZE ) as usize, "no limit after None" );
}

/// `image_tiled_draw` repeats an image over a rectangle as one quad. Over
/// `[ 0, 0 ]`–`[ 8, 4 ]` with 2×2 tiles anchored at the origin it must give the
/// same frame, pixel for pixel, as the eight tiles drawn as sprites one by
/// one. A tile anchor off by a texel, or a flipped image, would differ.
#[ wasm_bindgen_test ]
fn a_tiled_image_matches_its_tiles_drawn_one_by_one()
{
  let ( gl, mut backend ) = backend_new();
  backend.submit( &[ RenderCommand::Clear( Clear { color : BLACK } ) ] ).unwrap();
  backend.image_tiled_draw( ResourceId::new( 0 ), [ 0.0, 0.0 ], [ 8.0, 4.0 ], [ 0.0, 4.0 ], [ 2.0, 2.0 ], [ 1.0; 4 ] );
  let tiled = frame( &gl );

  let ( gl, mut backend ) = backend_new();
  let mut commands = vec![ RenderCommand::Clear( Clear { color : BLACK } ) ];
  for row in 0..2_u8
  {
    for column in 0..4_u8
    {
      commands.push( sprite( f32::from( column ) * 2.0, f32::from( row ) * 2.0, 1.0 ) );
    }
  }
  backend.submit( &commands ).unwrap();
  let one_by_one = frame( &gl );

  assert_eq!( count( &tiled, [ 0, 0, 0 ] ), ( SIZE * SIZE - 32 ) as usize, "the tiles cover 8×4 pixels" );
  assert!( tiled == one_by_one, "the tiled quad matches the tiles drawn one by one" );
}

/// `textures_loaded` is `false` until a texture is registered, then reflects
/// whether every one has its pixels. Bitmaps upload synchronously, so it turns
/// `true` as soon as they are loaded. A bake gated on it must not run before
/// the spec's assets are in.
#[ wasm_bindgen_test ]
fn textures_loaded_waits_for_a_registered_texture()
{
  let gl = gl_init();
  let mut backend = WebGlBackend::new( RenderConfig::default(), gl ).unwrap();
  assert!( !backend.textures_loaded(), "nothing registered yet" );
  backend.assets_load( &empty_assets() ).unwrap();
  assert!( !backend.textures_loaded(), "still nothing registered" );
  backend.assets_load( &assets() ).unwrap();
  assert!( backend.textures_loaded(), "the bitmap is uploaded" );
}
