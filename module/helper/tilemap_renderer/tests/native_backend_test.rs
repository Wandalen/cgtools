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
/// `size x size` viewport while leaving every corner outside it: extent 24
/// (the 8x8 `region` scaled by `Transform::scale = 3.0`), spanning world
/// `[size / 2 - 12, size / 2 + 12]` (e.g. on a 64x64 viewport,
/// `[20, 44] x [20, 44]`). `position` is the sprite's bottom-left corner
/// (BUG-240's fixed anchor convention), not its center, so it's offset by
/// the half-extent rather than set to `size / 2` directly.
fn centered_sprite_command( size : f32 ) -> RenderCommand
{
  RenderCommand::Sprite( Sprite
  {
    transform : Transform { position : [ size / 2.0 - 12.0, size / 2.0 - 12.0 ], scale : [ 3.0, 3.0 ], ..Default::default() },
    sprite : ResourceId::new( 0 ),
    tint : [ 1.0, 1.0, 1.0, 1.0 ],
    blend : BlendMode::default(),
    clip : None,
  })
}

/// Loads `solid_sprite_assets`, submits a clear plus a
/// `centered_sprite_command( size )`, and returns the resulting readback
/// `Bitmap`.
fn centered_sprite_render( backend : &mut NativeBackend, size : f32 ) -> Bitmap
{
  backend.assets_load( &solid_sprite_assets() ).expect( "assets_load failed" );
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

  let bitmap = centered_sprite_render( &mut backend, 64.0 );

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

  let bitmap = centered_sprite_render( &mut backend, 64.0 );
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
  let bitmap = centered_sprite_render( &mut backend, 128.0 );

  assert_eq!( bitmap.width, 128 );
  assert_eq!( bitmap.height, 128 );
  assert_eq!( bitmap.bytes.len(), 128 * 128 * 4 );
}

// test_kind: bug_reproducer(BUG-240)
/// ## Root Cause
/// `NativeBackend::quad_vertices` (`src/adapters/native.rs`) fed its local
/// quad corners (`[-0.5, 0.5]`) into the transform matrix unscaled by the
/// sprite's own `region` pixel size -- on-screen footprint was
/// `Transform::scale` alone, independent of `region.{2,3}` (width/height).
/// `webgl.rs`'s `sprite.vert` (`world = u_transform * vec3(quad *
/// u_region.zw, 1.0)`), `webgpu.rs`'s `vs_main` (identical shape), and
/// `svg.rs`'s `<symbol viewBox="region...">` (`sprites_load`) all instead
/// scale the local quad by `region`'s pixel size before the transform, so
/// `Transform::scale = 1` means "true source-pixel size" on every other
/// backend. Isolated here with `region = [0,0,8,8]`, `scale = [2,2]`: the
/// fixed convention gives an on-screen footprint of `region.w * scale = 16`
/// pixels; the pre-fix convention gave `scale` alone (2 pixels) -- a point 6
/// pixels from the sprite's center is inside the fixed footprint's
/// half-extent (8) but was outside the broken one's (1).
///
/// ## Why Not Caught
/// The pre-existing `sprite_and_corner_pixels_match_configured_colors` test
/// only sampled the viewport's exact center and a far corner -- both inside
/// (or outside) the sprite under either the correct or the broken formula,
/// since a large, coarse footprint change still covers the same center
/// pixel. No test sampled a point whose in/out status actually depends on
/// `region`'s own pixel size rather than `scale` alone.
///
/// ## Fix Applied
/// `quad_vertices` now scales the local quad's `[0, 1]`-mapped corner
/// (`fx`/`fy`, already computed for UV) by `region[2]`/`region[3]` before
/// applying the transform's linear part, matching `sprite.vert`/`vs_main`
/// exactly. UV math is untouched.
///
/// ## Prevention
/// A backend claiming behavioral parity with sibling backends ("the same
/// minimal command family the WebGPU adapter translates") needs that claim
/// checked against the siblings' actual per-vertex geometry math, not just
/// their command-family support list -- verified here by directly reading
/// `sprite.vert`, `webgpu.rs`'s WGSL `vs_main`, and `svg.rs`'s
/// `sprites_load`, rather than trusting the doc comment's claim at face
/// value.
///
/// ## Pitfall
/// This configuration is deliberately symmetric about the viewport's own
/// center (`position` set to `center - footprint / 2` on both axes) so the
/// assertions below hold regardless of which screen-space direction the
/// renderer's NDC-to-pixel Y axis actually runs -- an asymmetric probe point
/// would silently depend on an axis convention this test never independently
/// confirms.
#[ test ]
fn sprite_footprint_scales_with_region_pixel_size()
{
  let mut backend = NativeBackend::new( RenderConfig { width : 64, height : 64, ..Default::default() } )
  .expect( "NativeBackend::new failed -- needs a Vulkan ICD (a software one such as lavapipe suffices)" );

  // region 8x8 * scale 2 = 16-wide footprint (half-extent 8), centered at
  // the 64x64 viewport's own center (32, 32) by anchoring `position` at the
  // footprint's corner: `32 - 16 / 2 = 24`.
  let command = RenderCommand::Sprite( Sprite
  {
    transform : Transform { position : [ 24.0, 24.0 ], scale : [ 2.0, 2.0 ], ..Default::default() },
    sprite : ResourceId::new( 0 ),
    tint : [ 1.0, 1.0, 1.0, 1.0 ],
    blend : BlendMode::default(),
    clip : None,
  });

  backend.assets_load( &solid_sprite_assets() ).expect( "assets_load failed" );
  let commands = [ RenderCommand::Clear( Clear { color : CLEAR } ), command ];
  backend.submit( &commands ).expect( "submit failed" );
  let bitmap = match backend.output().expect( "output failed" )
  {
    Output::Bitmap( bitmap ) => bitmap,
    other => panic!( "expected Output::Bitmap, got {other:?}" ),
  };
  let at = | x : u32, y : u32 |
  {
    let start = ( ( y * bitmap.width + x ) * 4 ) as usize;
    [ bitmap.bytes[ start ], bitmap.bytes[ start + 1 ], bitmap.bytes[ start + 2 ], bitmap.bytes[ start + 3 ] ]
  };

  // Row 32 is pinned throughout -- the viewport's own symmetric center row,
  // so every assertion is invariant to axis-flip direction (see Pitfall).
  assert_eq!( at( 32, 32 ), SPRITE_RGBA, "viewport center should be the sprite's configured color" );
  assert_eq!( at( 38, 32 ), SPRITE_RGBA, "6px from center is inside the region-scaled footprint (half-extent 8)" );
  assert_eq!( at( 26, 32 ), SPRITE_RGBA, "6px from center (other side) is inside the region-scaled footprint" );
  assert_eq!( at( 10, 32 ), [ 0, 0, 0, 255 ], "22px from center is outside even the region-scaled footprint" );
  assert_eq!( at( 0, 0 ), [ 0, 0, 0, 255 ], "far corner pixel should be the clear color" );
}
