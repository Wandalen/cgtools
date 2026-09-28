//! `WebGlBackend` premultiplied-alpha compositing, against a live WebGL2
//! context with pixel read-back.
//!
//! wasm32-only for the same reason as `webgl_context_loss_test.rs`: the
//! subject compiles real shaders against a live `WebGl2RenderingContext`,
//! which only exists under the workspace's headless-browser wasm32 runner.
//!
//! Every case draws a 1×1 texel stretched over the whole viewport onto a known
//! opaque background and reads pixel `( 0, 0 )` back. The same visible colour
//! is authored twice — once straight-alpha, once premultiplied — and the two
//! results must match: a premultiplied texture is a storage format, not a
//! different look.

#![ cfg( all( target_arch = "wasm32", feature = "adapter-webgl" ) ) ]

use minwebgl as gl;
use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::assets::{ Assets, ImageAsset, ImageSource, PixelFormat, SpriteAsset };
use tilemap_renderer::backend::Backend;
use tilemap_renderer::commands::{ Clear, RenderCommand, Sprite };
use tilemap_renderer::types::{ BlendMode, MipmapMode, RenderConfig, ResourceId, SamplerFilter, Transform, WrapMode };
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

/// Opaque blue background every case composites onto, so both the source and
/// the destination term of the blend show up in the read-back pixel.
const BACKGROUND : [ f32; 4 ] = [ 0.0, 0.0, 1.0, 1.0 ];

/// White at 50% coverage, stored straight-alpha.
const STRAIGHT_TEXEL : [ u8; 4 ] = [ 255, 255, 255, 128 ];
/// The same white at 50% coverage, stored premultiplied ( RGB = 255 · 128/255 ).
const PREMULTIPLIED_TEXEL : [ u8; 4 ] = [ 128, 128, 128, 128 ];

/// Same live-context helper as `webgl_context_loss_test.rs::gl_init`.
fn gl_init() -> gl::GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

/// One 1×1 RGBA image ( id 0 ) and one sprite ( id 0 ) covering it.
fn texel_assets( texel : [ u8; 4 ], premultiplied : bool ) -> Assets
{
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
    geometries : Vec::new(),
    gradients : Vec::new(),
    patterns : Vec::new(),
    clip_masks : Vec::new(),
    paths : Vec::new(),
  }
}

/// Draws `assets`' sprite over the whole viewport with `tint` / `blend` onto
/// [`BACKGROUND`] and returns the RGBA bytes of pixel `( 0, 0 )`.
fn sprite_pixel( assets : &Assets, tint : [ f32; 4 ], blend : BlendMode ) -> [ u8; 4 ]
{
  let gl = gl_init();
  let config = RenderConfig::default();
  let ( width, height ) = ( config.width as f32, config.height as f32 );
  let mut backend = WebGlBackend::new( config, gl.clone() ).unwrap();
  backend.assets_load( assets ).unwrap();

  // The 1×1 region stretched to the viewport size covers every pixel.
  let transform = Transform { scale : [ width, height ], ..Transform::default() };
  backend.submit
  (
    &[
      RenderCommand::Clear( Clear { color : BACKGROUND } ),
      RenderCommand::Sprite( Sprite { transform, sprite : ResourceId::new( 0 ), tint, blend, clip : None } ),
    ]
  ).unwrap();

  let mut pixel = [ 0_u8; 4 ];
  gl.read_pixels_with_opt_u8_array( 0, 0, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixel ) ).unwrap();
  pixel
}

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
/// premultiplied texture over blue and requires identical pixels: 25% white
/// over blue ≈ `( 64, 64, 255 )`. Before the fix the premultiplied draw read
/// back ≈ `( 128, 128, 255 )`.
///
/// ## Pitfall
/// The tint alpha is where the scene compiler folds layer alpha and instance
/// alpha, so this is the path every faded layer takes — not an edge case.
// test_kind: bug_reproducer
#[ wasm_bindgen_test ]
fn sprite_premultiplied_tint_alpha_matches_straight()
{
  let tint = [ 1.0, 1.0, 1.0, 0.5 ];
  let straight = sprite_pixel( &texel_assets( STRAIGHT_TEXEL, false ), tint, BlendMode::Normal );
  let premultiplied = sprite_pixel( &texel_assets( PREMULTIPLIED_TEXEL, true ), tint, BlendMode::Normal );

  assert_pixel_close( straight, [ 64, 64, 255, 255 ], "straight texel, tint alpha 0.5" );
  assert_pixel_close( premultiplied, straight, "premultiplied texel, tint alpha 0.5" );
}
