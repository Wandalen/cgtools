//! `NativeBackend` real-GPU pixel-readback contract tests.
//!
//! Constructs a real `gpu_hal` native device (a software Vulkan ICD such as
//! lavapipe suffices) and asserts on pixels read back from the offscreen
//! surface -- proving `submit`/`output` actually render and read back real
//! GPU content, not merely that the trait compiles. Mirrors `gpu_hal`'s own
//! `native_backend_test.rs::triangle_render_readback` exact-equality style.

#![ cfg( all( feature = "adapter-native", not( target_arch = "wasm32" ) ) ) ]

use tilemap_renderer::adapters::native::NativeBackend;
use tilemap_renderer::assets::{ Assets, ImageAsset, ImageSource, PixelFormat, SpriteAsset };
use tilemap_renderer::backend::{ Backend, Bitmap, Output };
use tilemap_renderer::commands::{ Clear, RenderCommand, Sprite };
use tilemap_renderer::types::{ BlendMode, MipmapMode, RenderConfig, ResourceId, SamplerFilter, Transform, WrapMode };

mod helpers;
use helpers::empty_assets;

/// Clear color, distinct from `SPRITE_RGBA` below.
const CLEAR : [ f32; 4 ] = [ 0.0, 0.0, 0.0, 1.0 ];
/// Sprite source color -- solid red, distinct from `CLEAR`.
const SPRITE_RGBA : [ u8; 4 ] = [ 255, 0, 0, 255 ];

/// Builds an 8x8 solid-red `Assets` set: one image, one sprite covering the
/// full sheet.
fn solid_sprite_assets() -> Assets
{
  let mut assets = empty_assets();
  assets.images.push( ImageAsset
  {
    id : ResourceId::new( 0 ),
    source : ImageSource::Bitmap
    {
      bytes : SPRITE_RGBA.repeat( 8 * 8 ),
      width : 8,
      height : 8,
      format : PixelFormat::Rgba8,
    },
    filter : SamplerFilter::default(),
    mipmap : MipmapMode::default(),
    wrap : WrapMode::default(),
  });
  assets.sprites.push( SpriteAsset { id : ResourceId::new( 0 ), sheet : ResourceId::new( 0 ), region : [ 0.0, 0.0, 8.0, 8.0 ] } );
  assets
}

/// A `Sprite` command sized/positioned to cover the center of a
/// `size x size` viewport while leaving every corner outside it: half-extent
/// 12 centered at `size / 2` (e.g. on a 64x64 viewport, spans world
/// `[20, 44] x [20, 44]`).
fn centered_sprite_command( size : f32 ) -> RenderCommand
{
  RenderCommand::Sprite( Sprite
  {
    transform : Transform { position : [ size / 2.0, size / 2.0 ], scale : [ 24.0, 24.0 ], ..Default::default() },
    sprite : ResourceId::new( 0 ),
    tint : [ 1.0, 1.0, 1.0, 1.0 ],
    blend : BlendMode::default(),
    clip : None,
  })
}

/// Loads `solid_sprite_assets`, submits a clear plus a
/// `centered_sprite_command( size )`, and returns the resulting readback
/// `Bitmap`.
fn render_centered_sprite( backend : &mut NativeBackend, size : f32 ) -> Bitmap
{
  backend.load_assets( &solid_sprite_assets() ).expect( "load_assets failed" );
  let commands = [ RenderCommand::Clear( Clear { color : CLEAR } ), centered_sprite_command( size ) ];
  backend.submit( &commands ).expect( "submit failed" );

  match backend.output().expect( "output failed" )
  {
    Output::Bitmap( bitmap ) => bitmap,
    other => panic!( "expected Output::Bitmap, got {other:?}" ),
  }
}

/// T01 -- construct at 64x64, load a solid-color sprite, submit, and read
/// back a `Bitmap` whose dimensions match the configured viewport.
#[ test ]
fn construct_load_submit_output_returns_matching_dimensions()
{
  let mut backend = NativeBackend::new( RenderConfig { width : 64, height : 64, ..Default::default() } )
  .expect( "NativeBackend::new failed -- needs a Vulkan ICD (a software one such as lavapipe suffices)" );

  let bitmap = render_centered_sprite( &mut backend, 64.0 );

  assert_eq!( bitmap.width, 64 );
  assert_eq!( bitmap.height, 64 );
  assert_eq!( bitmap.channels, 4 );
  assert_eq!( bitmap.bytes.len(), 64 * 64 * 4 );
}

/// T02 / C5 / AF1 -- exact byte match at the sprite's known pixel location
/// (the viewport center, well inside the sprite quad) and at a corner
/// outside it: sprite pixel equals the configured sprite RGBA, corner pixel
/// equals the clear color -- rules out an all-clear false pass.
#[ test ]
fn sprite_and_corner_pixels_match_configured_colors()
{
  let mut backend = NativeBackend::new( RenderConfig { width : 64, height : 64, ..Default::default() } )
  .expect( "NativeBackend::new failed -- needs a Vulkan ICD (a software one such as lavapipe suffices)" );

  let bitmap = render_centered_sprite( &mut backend, 64.0 );
  let at = | x : u32, y : u32 |
  {
    let start = ( ( y * bitmap.width + x ) * 4 ) as usize;
    [ bitmap.bytes[ start ], bitmap.bytes[ start + 1 ], bitmap.bytes[ start + 2 ], bitmap.bytes[ start + 3 ] ]
  };

  assert_eq!( at( 32, 32 ), SPRITE_RGBA, "center pixel should be the sprite's configured color" );
  assert_eq!( at( 0, 0 ), [ 0, 0, 0, 255 ], "corner pixel should be the clear color" );
}

/// T03 -- `resize(128, 128)` after construction, then repeat T01's flow: the
/// returned `Bitmap` reflects the new dimensions, not the original.
#[ test ]
fn resize_then_output_reflects_new_dimensions()
{
  let mut backend = NativeBackend::new( RenderConfig { width : 64, height : 64, ..Default::default() } )
  .expect( "NativeBackend::new failed -- needs a Vulkan ICD (a software one such as lavapipe suffices)" );

  backend.resize( 128, 128 );
  let bitmap = render_centered_sprite( &mut backend, 128.0 );

  assert_eq!( bitmap.width, 128 );
  assert_eq!( bitmap.height, 128 );
  assert_eq!( bitmap.bytes.len(), 128 * 128 * 4 );
}
