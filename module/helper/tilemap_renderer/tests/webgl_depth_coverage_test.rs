//! The WebGL backend's depth and coverage controls against a live WebGL2
//! context, with pixel read-back: `SetDepthWrite`, a sprite batch's
//! `occlude_overlap` and its `alpha_clip`. `tilemap_scene`'s opaque two-pass
//! split and single-coverage buckets are built on these three, and its
//! `opaque_pass_test` checks only the command stream it emits.
//!
//! wasm32-only for the same reason as `webgl_camera_test.rs`: the subject
//! compiles real shaders against a live `WebGl2RenderingContext`, which only
//! exists under the workspace's headless-browser wasm32 runner.
//!
//! The cases count pixel values over the whole frame instead of reading
//! chosen coordinates, so they do not depend on where the backend puts the
//! origin or which way y points.

#![ cfg( all( target_arch = "wasm32", feature = "adapter-webgl" ) ) ]

mod helpers;

use helpers::webgl::gl_init;
use minwebgl as gl;
use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::assets::{ Assets, ImageAsset, ImageSource, PixelFormat, SpriteAsset };
use tilemap_renderer::backend::Backend;
use tilemap_renderer::commands::
{
  AddSpriteInstance,
  BindBatch,
  Clear,
  CreateSpriteBatch,
  DrawBatch,
  RenderCommand,
  SetDepthWrite,
  Sprite,
  SpriteBatchParams,
  UnbindBatch,
};
use tilemap_renderer::types::{ BlendMode, MipmapMode, RenderConfig, ResourceId, SamplerFilter, Transform, WrapMode };
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

/// Drawing-buffer size every frame is rendered at, pinned as in
/// `webgl_camera_test.rs`.
const WIDTH : u32 = 32;
/// See [`WIDTH`].
const HEIGHT : u32 = 16;

/// Sprite 0: a 1×1 opaque white image. Sprite 1: a 2×1 image whose left texel
/// is opaque white and whose right texel is white at alpha 64 / 255.
fn assets() -> Assets
{
  let image = | id : u32, bytes : Vec< u8 >, width : u32 | ImageAsset
  {
    id : ResourceId::new( id ),
    source : ImageSource::Bitmap { bytes, width, height : 1, format : PixelFormat::Rgba8 },
    filter : SamplerFilter::Nearest,
    mipmap : MipmapMode::Off,
    wrap : WrapMode::Clamp,
    premultiplied : false,
  };
  Assets
  {
    fonts : Vec::new(),
    images : vec!
    [
      image( 0, vec![ 255; 4 ], 1 ),
      image( 1, vec![ 255, 255, 255, 255, 255, 255, 255, 64 ], 2 ),
    ],
    sprites : vec!
    [
      SpriteAsset { id : ResourceId::new( 0 ), sheet : ResourceId::new( 0 ), region : [ 0.0, 0.0, 1.0, 1.0 ] },
      SpriteAsset { id : ResourceId::new( 1 ), sheet : ResourceId::new( 1 ), region : [ 0.0, 0.0, 2.0, 1.0 ] },
    ],
    geometries : Vec::new(),
    gradients : Vec::new(),
    patterns : Vec::new(),
    clip_masks : Vec::new(),
    paths : Vec::new(),
  }
}

/// A quad at `( x, y )`, at `depth`, scaled by `( width, height )`. A sprite
/// draws at its region's pixel size times the scale, so on sprite 0 ( 1×1 )
/// this covers `width`×`height` pixels and on sprite 1 ( 2×1 ) twice as wide.
fn quad( x : f32, y : f32, width : f32, height : f32, depth : f32 ) -> Transform
{
  Transform { position : [ x, y ], scale : [ width, height ], depth, ..Transform::default() }
}

/// On a fresh canvas and backend: clears to opaque black, submits `draw` and
/// returns the frame as RGBA pixels.
fn frame( draw : &[ RenderCommand ] ) -> Vec< [ u8; 4 ] >
{
  let gl = gl_init();
  let canvas = gl.canvas().unwrap().dyn_into::< web_sys::HtmlCanvasElement >().unwrap();
  canvas.set_width( WIDTH );
  canvas.set_height( HEIGHT );
  let config = RenderConfig { width : WIDTH, height : HEIGHT, ..RenderConfig::default() };
  let mut backend = WebGlBackend::new( config, gl.clone() ).unwrap();
  backend.assets_load( &assets() ).unwrap();

  let mut commands = vec![ RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 0.0, 1.0 ] } ) ];
  commands.extend( draw.iter().copied() );
  backend.submit( &commands ).unwrap();

  let mut bytes = vec![ 0_u8; ( WIDTH * HEIGHT * 4 ) as usize ];
  gl.read_pixels_with_opt_u8_array( 0, 0, WIDTH as i32, HEIGHT as i32, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut bytes ) ).unwrap();
  bytes.as_chunks::< 4 >().0.to_vec()
}

/// How many pixels of `frame` have RGB `rgb`.
fn count( frame : &[ [ u8; 4 ] ], rgb : [ u8; 3 ] ) -> usize
{
  frame.iter().filter( | p | p[ .. 3 ] == rgb ).count()
}

/// Whether `value` is within 2 of `want`: blending rounds differently on
/// different GPUs.
fn near( value : u8, want : u8 ) -> bool
{
  value.abs_diff( want ) <= 2
}

/// A per-sprite draw in `tint`.
fn sprite( sprite : u32, transform : Transform, tint : [ f32; 4 ] ) -> RenderCommand
{
  RenderCommand::Sprite( Sprite { transform, sprite : ResourceId::new( sprite ), tint, blend : BlendMode::Normal, clip : None } )
}

/// A sprite batch on `sheet` with `alpha_clip` and `occlude_overlap`, holding
/// one instance of `sprite` per transform, in `tint`.
fn sprite_batch( sheet : u32, sprite : u32, alpha_clip : f32, occlude_overlap : bool, instances : &[ Transform ], tint : [ f32; 4 ] ) -> Vec< RenderCommand >
{
  let batch = ResourceId::new( 0 );
  let params = SpriteBatchParams
  {
    transform : Transform::default(),
    sheet : ResourceId::new( sheet ),
    blend : BlendMode::Normal,
    clip : None,
    alpha_clip,
    occlude_overlap,
  };
  let mut commands = vec!
  [
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch { batch, params } ),
    RenderCommand::BindBatch( BindBatch { batch } ),
  ];
  commands.extend( instances.iter().map( | &transform |
    RenderCommand::AddSpriteInstance( AddSpriteInstance { transform, sprite : ResourceId::new( sprite ), tint } )
  ));
  commands.push( RenderCommand::UnbindBatch( UnbindBatch ) );
  commands.push( RenderCommand::DrawBatch( DrawBatch { batch } ) );
  commands
}

const RED : [ f32; 4 ] = [ 1.0, 0.0, 0.0, 1.0 ];
const GREEN : [ f32; 4 ] = [ 0.0, 1.0, 0.0, 1.0 ];

/// A nearer red quad, then a farther green one over the same pixels. With
/// depth writes on, the red quad's depth rejects the green one. After
/// `SetDepthWrite( false )` the red quad leaves no depth, so the green one
/// draws over it. This is how `tilemap_scene` keeps its transparent pass from
/// being cut by its own earlier draws.
#[ wasm_bindgen_test ]
fn set_depth_write_false_leaves_no_depth_to_reject_a_farther_draw()
{
  let near_red = sprite( 0, quad( 8.0, 4.0, 8.0, 4.0, 0.5 ), RED );
  let far_green = sprite( 0, quad( 8.0, 4.0, 8.0, 4.0, 0.25 ), GREEN );

  let writes_on = frame( &[ near_red, far_green ] );
  assert_eq!( count( &writes_on, [ 255, 0, 0 ] ), 32, "depth writes on: the red quad keeps its 8×4 pixels" );
  assert_eq!( count( &writes_on, [ 0, 255, 0 ] ), 0, "and the farther green quad is rejected" );

  let writes_off = frame( &[ RenderCommand::SetDepthWrite( SetDepthWrite { enabled : false } ), near_red, far_green ] );
  assert_eq!( count( &writes_off, [ 0, 255, 0 ] ), 32, "depth writes off: the green quad draws over the red one" );
  assert_eq!( count( &writes_off, [ 255, 0, 0 ] ), 0 );
}

/// Two half-transparent white instances of one batch overlap by 4×4 pixels.
/// With `occlude_overlap` the overlap is painted once, at the same 50 % grey
/// as the rest; without it the second instance blends over the first there.
/// This is the dual-grid drop shadow that must not darken where it bleeds.
#[ wasm_bindgen_test ]
fn occlude_overlap_paints_an_overlap_once()
{
  let half_white = [ 1.0, 1.0, 1.0, 0.5 ];
  let instances = [ quad( 8.0, 4.0, 8.0, 4.0, 0.0 ), quad( 12.0, 4.0, 8.0, 4.0, 0.0 ) ];
  let lit = | frame : &[ [ u8; 4 ] ] | frame.iter().filter( | p | p[ 0 ] != 0 ).map( | p | p[ 0 ] ).collect::< Vec< u8 > >();

  let once = lit( &frame( &sprite_batch( 0, 0, 0.0, true, &instances, half_white ) ) );
  assert_eq!( once.len(), 12 * 4, "the two instances cover 12×4 pixels" );
  assert!( once.iter().all( | &v | near( v, 128 ) ), "single coverage: every pixel is the one 50 % layer, got {once:?}" );

  let twice = lit( &frame( &sprite_batch( 0, 0, 0.0, false, &instances, half_white ) ) );
  assert_eq!( twice.len(), 12 * 4 );
  assert_eq!( twice.iter().filter( | &&v | near( v, 191 ) ).count(), 4 * 4, "without it the 4×4 overlap blends twice: {twice:?}" );
}

/// `alpha_clip` discards the texels below it: no colour and no depth. A 2×1
/// texture (opaque left texel, alpha 64 / 255 right texel) drawn over 16×4
/// pixels with `alpha_clip = 0.5`, nearer than a green quad drawn after it
/// over the same pixels.
/// The opaque half stays white and keeps the green quad out with its depth;
/// the clipped half writes nothing, so the green quad shows there. Without
/// the clip that half blends a dim grey and its depth still hides the green.
#[ wasm_bindgen_test ]
fn alpha_clip_discards_colour_and_depth_below_the_threshold()
{
  // Sprite 1 is 2×1 pixels, so a 8×4 scale covers 16×4, like the green quad.
  let instances = [ quad( 8.0, 4.0, 8.0, 4.0, 0.5 ) ];
  let far_green = sprite( 0, quad( 8.0, 4.0, 16.0, 4.0, 0.25 ), GREEN );
  let white = [ 1.0; 4 ];

  let mut clipped = sprite_batch( 1, 1, 0.5, false, &instances, white );
  clipped.push( far_green );
  let clipped = frame( &clipped );
  assert_eq!( count( &clipped, [ 255, 255, 255 ] ), 32, "the opaque half stays white" );
  assert_eq!( count( &clipped, [ 0, 255, 0 ] ), 32, "the clipped half leaves no depth, so the green quad shows there" );

  let mut unclipped = sprite_batch( 1, 1, 0.0, false, &instances, white );
  unclipped.push( far_green );
  let unclipped = frame( &unclipped );
  assert_eq!( count( &unclipped, [ 0, 255, 0 ] ), 0, "without the clip the faint half writes depth and hides the green quad" );
  assert_eq!( unclipped.iter().filter( | p | p[ 1 ] == p[ 0 ] && near( p[ 0 ], 64 ) ).count(), 32, "and blends a dim grey" );
}
