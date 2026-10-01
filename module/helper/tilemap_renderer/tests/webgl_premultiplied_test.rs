//! `WebGlBackend` premultiplied-alpha compositing, against a live WebGL2
//! context with pixel read-back.
//!
//! wasm32-only for the same reason as `webgl_context_loss_test.rs`: the
//! subject compiles real shaders against a live `WebGl2RenderingContext`,
//! which only exists under the workspace's headless-browser wasm32 runner.
//!
//! Every case draws a 1×1 texel stretched over the whole viewport onto a known
//! background and reads pixel `( 0, 0 )` back. The same visible colour
//! is authored twice — once straight-alpha, once premultiplied — and the two
//! results must match under `Normal`, `Add` and the `Overlay` fallback: there a
//! premultiplied texture is a storage format, not a different look.
//!
//! `Multiply` and `Screen` are the exception. Their GL factors approximate the
//! reference formula for a straight source below full alpha, and compute it
//! exactly for a premultiplied one, so the twins differ there; those cases
//! check the premultiplied draw against the reference formula instead.

#![ cfg( all( target_arch = "wasm32", feature = "adapter-webgl" ) ) ]

use minwebgl as gl;
use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::assets::{ Assets, DataType, GeometryAsset, ImageAsset, ImageSource, PixelFormat, Source, SpriteAsset };
use tilemap_renderer::backend::Backend;
use tilemap_renderer::commands::
{
  AddMeshInstance, AddSpriteInstance, BindBatch, Clear, CreateMeshBatch, CreateSpriteBatch, DrawBatch, Mesh,
  MeshBatchParams, RenderCommand, Sprite, SpriteBatchParams, UnbindBatch,
};
use tilemap_renderer::types::{ BlendMode, FillRef, MipmapMode, RenderConfig, ResourceId, SamplerFilter, Topology, Transform, WrapMode };
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

/// Half-transparent mid grey every case composites onto. No channel of a
/// correct result saturates except under `Add`, so both the source and the
/// destination term of the colour blend show up in every channel, and the
/// alpha blend shows up in the alpha channel: a wrong destination factor, a
/// tint alpha applied to the texel's alpha twice, or one mode's factors in
/// place of another's each move the read-back pixel well outside the ±2
/// tolerance.
const BACKGROUND : [ f32; 4 ] = [ 0.5, 0.5, 0.5, 0.5 ];

/// White at 50% coverage, stored straight-alpha.
const STRAIGHT_TEXEL : [ u8; 4 ] = [ 255, 255, 255, 128 ];
/// The same white at 50% coverage, stored premultiplied ( RGB = 255 · 128/255 ).
const PREMULTIPLIED_TEXEL : [ u8; 4 ] = [ 128, 128, 128, 128 ];
/// Mid grey at 50% coverage, stored premultiplied: unlike white, it changes the
/// destination under both `Multiply` and `Screen`.
const PREMULTIPLIED_GREY_TEXEL : [ u8; 4 ] = [ 64, 64, 64, 128 ];

/// Identity tint.
const WHITE : [ f32; 4 ] = [ 1.0, 1.0, 1.0, 1.0 ];
/// White tint at half alpha — how the scene compiler expresses a faded layer.
const HALF_ALPHA : [ f32; 4 ] = [ 1.0, 1.0, 1.0, 0.5 ];

/// 50% white over [`BACKGROUND`] under `Normal`: `0.502 + 0.5 · 0.498` in every
/// channel.
const HALF_WHITE_OVER_GREY : [ u8; 4 ] = [ 191, 191, 191, 191 ];
/// 25% white over [`BACKGROUND`]: the 50%-coverage texel at tint alpha 0.5,
/// `0.251 + 0.5 · 0.749` in every channel.
const QUARTER_WHITE_OVER_GREY : [ u8; 4 ] = [ 159, 159, 159, 159 ];
/// 50% white added to [`BACKGROUND`]: colour `0.502 + 0.5` saturates, alpha
/// still composites "over" as under `Normal`.
const HALF_WHITE_ADDED_TO_GREY : [ u8; 4 ] = [ 255, 255, 255, 191 ];

/// Same live-context helper as `webgl_context_loss_test.rs::gl_init`.
fn gl_init() -> gl::GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

/// Little-endian `f32` bytes for a geometry `Source::Bytes` buffer.
fn f32_bytes( values : &[ f32 ] ) -> Vec< u8 >
{
  values.iter().flat_map( | v | v.to_le_bytes() ).collect()
}

/// One 1×1 RGBA image ( id 0 ), one sprite ( id 0 ) covering it and one unit
/// quad geometry ( id 0, non-indexed triangle list with matching UVs ).
fn texel_assets( texel : [ u8; 4 ], premultiplied : bool ) -> Assets
{
  let unit_quad = [ 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0 ];
  Assets
  {
    fonts : Vec::new(),
    images : vec!
    [
      ImageAsset
      {
        id : ResourceId::new( 0 ),
        source : ImageSource::Bitmap { bytes : texel.to_vec(), width : 1, height : 1, format : PixelFormat::Rgba8 },
        filter : SamplerFilter::Nearest,
        mipmap : MipmapMode::Off,
        wrap : WrapMode::Clamp,
        premultiplied,
      }
    ],
    sprites : vec![ SpriteAsset { id : ResourceId::new( 0 ), sheet : ResourceId::new( 0 ), region : [ 0.0, 0.0, 1.0, 1.0 ] } ],
    geometries : vec!
    [
      GeometryAsset
      {
        id : ResourceId::new( 0 ),
        positions : Source::Bytes( f32_bytes( &unit_quad ) ),
        uvs : Some( Source::Bytes( f32_bytes( &unit_quad ) ) ),
        indices : None,
        data_type : DataType::U16,
      }
    ],
    gradients : Vec::new(),
    patterns : Vec::new(),
    clip_masks : Vec::new(),
    paths : Vec::new(),
  }
}

/// Clears to [`BACKGROUND`], submits the commands `draw` builds and returns
/// the RGBA bytes of pixel `( 0, 0 )`. `draw` receives the transform that
/// stretches the 1×1 sprite / unit quad over the whole viewport.
fn pixel_after( assets : &Assets, draw : impl FnOnce( Transform ) -> Vec< RenderCommand > ) -> [ u8; 4 ]
{
  let gl = gl_init();
  let config = RenderConfig::default();
  let full_view = Transform { scale : [ config.width as f32, config.height as f32 ], ..Transform::default() };
  let mut backend = WebGlBackend::new( config, gl.clone() ).unwrap();
  backend.assets_load( assets ).unwrap();

  let mut commands = vec![ RenderCommand::Clear( Clear { color : BACKGROUND } ) ];
  commands.extend( draw( full_view ) );
  backend.submit( &commands ).unwrap();

  let mut pixel = [ 0_u8; 4 ];
  gl.read_pixels_with_opt_u8_array( 0, 0, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixel ) ).unwrap();
  pixel
}

/// Single `Sprite` command over the whole viewport.
fn sprite_pixel( assets : &Assets, tint : [ f32; 4 ], blend : BlendMode ) -> [ u8; 4 ]
{
  pixel_after( assets, | transform |
    vec![ RenderCommand::Sprite( Sprite { transform, sprite : ResourceId::new( 0 ), tint, blend, clip : None } ) ] )
}

/// Single `Mesh` command over the whole viewport, textured by image 0 or not.
fn mesh_pixel( assets : &Assets, fill : [ f32; 4 ], textured : bool ) -> [ u8; 4 ]
{
  pixel_after( assets, | transform |
    vec![ RenderCommand::Mesh( Mesh
    {
      transform,
      geometry : ResourceId::new( 0 ),
      fill : FillRef::Solid( fill ),
      texture : textured.then( || ResourceId::new( 0 ) ),
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    })])
}

/// One-instance sprite batch over the whole viewport.
fn sprite_batch_pixel( assets : &Assets, tint : [ f32; 4 ] ) -> [ u8; 4 ]
{
  let batch = ResourceId::new( 0 );
  let params = SpriteBatchParams { transform : Transform::default(), sheet : ResourceId::new( 0 ), blend : BlendMode::Normal, clip : None };
  pixel_after( assets, | transform | vec!
  [
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch { batch, params } ),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddSpriteInstance( AddSpriteInstance { transform, sprite : ResourceId::new( 0 ), tint } ),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch } ),
  ])
}

/// One-instance textured mesh batch over the whole viewport.
fn mesh_batch_pixel( assets : &Assets, tint : [ f32; 4 ] ) -> [ u8; 4 ]
{
  let batch = ResourceId::new( 0 );
  let params = MeshBatchParams
  {
    transform : Transform::default(),
    geometry : ResourceId::new( 0 ),
    fill : FillRef::Solid( WHITE ),
    texture : Some( ResourceId::new( 0 ) ),
    topology : Topology::TriangleList,
    blend : BlendMode::Normal,
    clip : None,
  };
  pixel_after( assets, | transform | vec!
  [
    RenderCommand::CreateMeshBatch( CreateMeshBatch { batch, params } ),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddMeshInstance( AddMeshInstance { transform, tint } ),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch } ),
  ])
}

/// Straight texels, their premultiplied twins and tints outside the plain 0–1
/// product: a tint alpha above 1, a brightening tint over a bright opaque
/// texel, and both over a half-covered one. The straight path's colour and
/// alpha are each clamped to 0–1 where the RGBA8 target is written, so the
/// premultiplied path has to reproduce `clamp( c · tint.rgb ) · clamp( a · tint.a )`
/// rather than scale the stored `c · a` by the tint.
const OUT_OF_RANGE_TINTS : [ ( [ u8; 4 ], [ u8; 4 ], [ f32; 4 ] ); 3 ] =
[
  ( [ 128, 128, 128, 255 ], [ 128, 128, 128, 255 ], [ 1.0, 1.0, 1.0, 1.5 ] ),
  ( [ 230, 230, 230, 255 ], [ 230, 230, 230, 255 ], [ 1.3, 1.3, 1.3, 0.5 ] ),
  ( [ 204, 204, 204, 128 ], [ 102, 102, 102, 128 ], [ 1.5, 1.5, 1.5, 1.5 ] ),
];

/// The read-back pixel of a reference blend over [`BACKGROUND`]: `colour` maps the
/// destination channel to the result, and alpha composites "over", as every
/// mode's alpha factors do.
fn reference_over_background( source_alpha : f32, colour : impl Fn( f32 ) -> f32 ) -> [ u8; 4 ]
{
  let dst = BACKGROUND[ 0 ];
  let byte = | v : f32 | ( v.clamp( 0.0, 1.0 ) * 255.0 ).round() as u8;
  let c = byte( colour( dst ) );
  [ c, c, c, byte( source_alpha + BACKGROUND[ 3 ] * ( 1.0 - source_alpha ) ) ]
}

/// A draw path: the pixel a texel's assets read back under a tint.
type PixelFn = fn( &Assets, [ f32; 4 ] ) -> [ u8; 4 ];

/// Asserts two read-back pixels agree within 8-bit rounding ( ±2 per channel ).
fn assert_pixel_close( actual : [ u8; 4 ], expected : [ u8; 4 ], what : &str )
{
  let close = actual.iter().zip( expected ).all( | ( a, e ) | a.abs_diff( e ) <= 2 );
  assert!( close, "{what}: got {actual:?}, expected ≈ {expected:?}" );
}

/// ## Root Cause
/// The premultiplied path swaps the blend source factor from `SRC_ALPHA` to
/// `ONE`, which is only correct when the fragment colour is itself
/// premultiplied. The shaders multiplied the texel by the tint component-wise,
/// so a tint alpha below 1 reduced coverage ( `A·ta` ) but left RGB at `R·A·t`
/// instead of `R·A·t·ta` — a faded premultiplied sprite composited too bright.
///
/// ## Why Not Caught
/// No test exercised the WebGL2 blend path of the premultiplied flag at all;
/// the only coverage stopped at the compiled `ImageAsset`.
///
/// ## Fix Applied
/// All four fragment shaders scale the tint's RGB by the tint's alpha under a
/// new `u_premultiplied` uniform, which every draw path uploads from its
/// texture's flag.
///
/// ## Prevention
/// Draws the same half-covered white at tint alpha 0.5 as a straight and as a
/// premultiplied texture over grey and requires identical pixels: 25% white
/// over grey ≈ `( 159, 159, 159, 159 )`. Before the fix the premultiplied draw
/// read back ≈ `( 223, 223, 223, 159 )`.
///
/// ## Pitfall
/// The tint alpha is where the scene compiler folds layer alpha and instance
/// alpha, so this is the path every faded layer takes — not an edge case.
// test_kind: bug_reproducer
#[ wasm_bindgen_test ]
fn sprite_premultiplied_tint_alpha_matches_straight()
{
  let straight = sprite_pixel( &texel_assets( STRAIGHT_TEXEL, false ), HALF_ALPHA, BlendMode::Normal );
  let premultiplied = sprite_pixel( &texel_assets( PREMULTIPLIED_TEXEL, true ), HALF_ALPHA, BlendMode::Normal );

  assert_pixel_close( straight, QUARTER_WHITE_OVER_GREY, "straight texel, tint alpha 0.5" );
  assert_pixel_close( premultiplied, straight, "premultiplied texel, tint alpha 0.5" );
}

/// Pins the core of the flag: under `Normal` a premultiplied texel composites
/// with source factor `ONE`. Drawn under `SRC_ALPHA` instead, its already
/// alpha-scaled RGB would be scaled again and read back ≈ `( 128, 128, 128, 191 )`
/// ( the darkened-edge artefact the flag exists to remove ).
#[ wasm_bindgen_test ]
fn sprite_premultiplied_normal_matches_straight()
{
  let straight = sprite_pixel( &texel_assets( STRAIGHT_TEXEL, false ), WHITE, BlendMode::Normal );
  let premultiplied = sprite_pixel( &texel_assets( PREMULTIPLIED_TEXEL, true ), WHITE, BlendMode::Normal );

  assert_pixel_close( straight, HALF_WHITE_OVER_GREY, "straight texel, Normal" );
  assert_pixel_close( premultiplied, straight, "premultiplied texel, Normal" );
}

/// `Add` swaps its source factor the same way as `Normal`: `src·a + dst` for
/// a straight texel equals `src + dst` for its premultiplied twin.
#[ wasm_bindgen_test ]
fn sprite_premultiplied_add_matches_straight()
{
  let straight = sprite_pixel( &texel_assets( STRAIGHT_TEXEL, false ), WHITE, BlendMode::Add );
  let premultiplied = sprite_pixel( &texel_assets( PREMULTIPLIED_TEXEL, true ), WHITE, BlendMode::Add );

  assert_pixel_close( straight, HALF_WHITE_ADDED_TO_GREY, "straight texel, Add" );
  assert_pixel_close( premultiplied, straight, "premultiplied texel, Add" );
}

/// `Overlay` falls back to `Normal` and takes the same source factor swap. With
/// `SRC_ALPHA` kept for a premultiplied texel, the fallback would read back
/// ≈ `( 128, 128, 128, 191 )`, the darkened edge of the plain `Normal` case.
#[ wasm_bindgen_test ]
fn sprite_premultiplied_overlay_fallback_matches_straight()
{
  let straight = sprite_pixel( &texel_assets( STRAIGHT_TEXEL, false ), WHITE, BlendMode::Overlay );
  let premultiplied = sprite_pixel( &texel_assets( PREMULTIPLIED_TEXEL, true ), WHITE, BlendMode::Overlay );

  assert_pixel_close( straight, HALF_WHITE_OVER_GREY, "straight texel, Overlay fallback" );
  assert_pixel_close( premultiplied, straight, "premultiplied texel, Overlay fallback" );
}

/// Under `Multiply` a premultiplied source composites the reference formula,
/// `dst · ( src · a + 1 - a )` ( `BlendMode::Multiply` ), at any coverage: GL's
/// `DST_COLOR` factor multiplies the destination by the stored `src · a`. The
/// tint's alpha is folded into the colour first, so a faded sprite stays exact.
/// A straight source takes the approximation `dst · ( src + 1 - a )` instead,
/// which here reads back like `Normal` ( 127 ) rather than the reference ( 96 ).
#[ wasm_bindgen_test ]
fn sprite_premultiplied_multiply_follows_the_reference()
{
  let assets = texel_assets( PREMULTIPLIED_GREY_TEXEL, true );
  let ( src, a ) = ( 128.0 / 255.0, 128.0 / 255.0 );

  let full = sprite_pixel( &assets, WHITE, BlendMode::Multiply );
  let faded = sprite_pixel( &assets, HALF_ALPHA, BlendMode::Multiply );

  assert_pixel_close( full, reference_over_background( a, | dst | dst * ( src * a + 1.0 - a ) ), "premultiplied grey, Multiply" );
  let a = a * 0.5;
  assert_pixel_close( faded, reference_over_background( a, | dst | dst * ( src * a + 1.0 - a ) ), "premultiplied grey, Multiply, tint alpha 0.5" );
}

/// Under `Screen` a premultiplied source composites screen at its coverage,
/// `dst + src · a - dst · src · a`: GL's `ONE` / `ONE_MINUS_SRC_COLOR` factors
/// read the stored `src · a`. A straight source screens at full strength
/// whatever its alpha, which here reads back 191 against the reference 160.
#[ wasm_bindgen_test ]
fn sprite_premultiplied_screen_follows_the_reference()
{
  let assets = texel_assets( PREMULTIPLIED_GREY_TEXEL, true );
  let ( src, a ) = ( 128.0 / 255.0, 128.0 / 255.0 );

  let full = sprite_pixel( &assets, WHITE, BlendMode::Screen );
  let faded = sprite_pixel( &assets, HALF_ALPHA, BlendMode::Screen );

  assert_pixel_close( full, reference_over_background( a, | dst | dst + src * a - dst * src * a ), "premultiplied grey, Screen" );
  let a = a * 0.5;
  assert_pixel_close( faded, reference_over_background( a, | dst | dst + src * a - dst * src * a ), "premultiplied grey, Screen, tint alpha 0.5" );
}

/// A textured mesh inherits its texture's flag ( `mesh_premultiplied` ), and
/// its fill alpha is kept premultiplied by `mesh.frag` like a sprite tint.
#[ wasm_bindgen_test ]
fn mesh_textured_premultiplied_matches_straight()
{
  let straight = mesh_pixel( &texel_assets( STRAIGHT_TEXEL, false ), HALF_ALPHA, true );
  let premultiplied = mesh_pixel( &texel_assets( PREMULTIPLIED_TEXEL, true ), HALF_ALPHA, true );

  assert_pixel_close( straight, QUARTER_WHITE_OVER_GREY, "straight textured mesh, fill alpha 0.5" );
  assert_pixel_close( premultiplied, straight, "premultiplied textured mesh, fill alpha 0.5" );
}

/// An untextured mesh is straight-alpha even while a premultiplied image is
/// loaded: its solid fill never passes through a premultiplied texel, so
/// taking the `ONE` source factor would draw it at full strength.
#[ wasm_bindgen_test ]
fn mesh_untextured_stays_straight()
{
  let pixel = mesh_pixel( &texel_assets( PREMULTIPLIED_TEXEL, true ), HALF_ALPHA, false );

  assert_pixel_close( pixel, HALF_WHITE_OVER_GREY, "untextured mesh, fill alpha 0.5" );
}

/// A tint outside the plain 0–1 product still draws a premultiplied texel like
/// its straight twin, on every draw path, since each fragment shader tints
/// through the shared `tint.glsl`. Before the fix the three cases read back
/// 192 / 213 / 255 premultiplied against 128 / 191 / 223 straight on every
/// path.
#[ wasm_bindgen_test ]
fn premultiplied_out_of_range_tint_matches_straight()
{
  let paths : [ ( &str, PixelFn ); 4 ] =
  [
    ( "sprite", | assets, tint | sprite_pixel( assets, tint, BlendMode::Normal ) ),
    ( "textured mesh", | assets, tint | mesh_pixel( assets, tint, true ) ),
    ( "sprite batch", sprite_batch_pixel ),
    ( "mesh batch", mesh_batch_pixel ),
  ];
  for ( path, pixel ) in paths
  {
    for ( straight_texel, premultiplied_texel, tint ) in OUT_OF_RANGE_TINTS
    {
      let straight = pixel( &texel_assets( straight_texel, false ), tint );
      let premultiplied = pixel( &texel_assets( premultiplied_texel, true ), tint );
      assert_pixel_close( premultiplied, straight, &format!( "{path}, texel {straight_texel:?}, tint {tint:?}" ) );
    }
  }
}

/// The sprite batch path reads the flag from its sheet and applies the
/// per-instance tint alpha premultiplied ( `sprite_batch.frag` ).
#[ wasm_bindgen_test ]
fn sprite_batch_premultiplied_matches_straight()
{
  let straight = sprite_batch_pixel( &texel_assets( STRAIGHT_TEXEL, false ), HALF_ALPHA );
  let premultiplied = sprite_batch_pixel( &texel_assets( PREMULTIPLIED_TEXEL, true ), HALF_ALPHA );

  assert_pixel_close( straight, QUARTER_WHITE_OVER_GREY, "straight sprite batch, instance alpha 0.5" );
  assert_pixel_close( premultiplied, straight, "premultiplied sprite batch, instance alpha 0.5" );
}

/// The mesh batch path reads the flag from its texture and applies the
/// per-instance tint alpha premultiplied ( `mesh_batch.frag` ).
#[ wasm_bindgen_test ]
fn mesh_batch_premultiplied_matches_straight()
{
  let straight = mesh_batch_pixel( &texel_assets( STRAIGHT_TEXEL, false ), HALF_ALPHA );
  let premultiplied = mesh_batch_pixel( &texel_assets( PREMULTIPLIED_TEXEL, true ), HALF_ALPHA );

  assert_pixel_close( straight, QUARTER_WHITE_OVER_GREY, "straight mesh batch, instance alpha 0.5" );
  assert_pixel_close( premultiplied, straight, "premultiplied mesh batch, instance alpha 0.5" );
}
