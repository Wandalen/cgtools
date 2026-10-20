//! `WebGlBackend::set_camera` end to end, against a live WebGL2 context with
//! pixel read-back: the matrix a caller sets (in `tilemap_scene`,
//! `Camera::to_view_mat3`) reaches every world-space draw path through
//! `submit`, and skips `ScreenSpaceSprite`.
//!
//! wasm32-only for the same reason as `webgl_context_loss_test.rs`: the
//! subject compiles real shaders against a live `WebGl2RenderingContext`,
//! which only exists under the workspace's headless-browser wasm32 runner.
//!
//! Each case draws a quad twice: once through a zoom-and-pan camera, and once
//! through the identity camera with the camera already folded into the quad's
//! transform by hand. The two frames must match pixel for pixel. Comparing
//! frames, not single pixels, keeps the cases independent of where the
//! backend puts the origin and which way y points.

#![ cfg( all( target_arch = "wasm32", feature = "adapter-webgl" ) ) ]

mod helpers;

use helpers::webgl::{ f32_bytes, gl_init };
use minwebgl as gl;
use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::assets::{ Assets, DataType, GeometryAsset, ImageAsset, ImageSource, PixelFormat, Source, SpriteAsset };
use tilemap_renderer::backend::Backend;
use tilemap_renderer::commands::
{
  AddMeshInstance,
  AddSpriteInstance,
  BindBatch,
  Clear,
  CreateMeshBatch,
  CreateSpriteBatch,
  DrawBatch,
  Mesh,
  MeshBatchParams,
  RenderCommand,
  Sprite,
  SpriteBatchParams,
  UnbindBatch,
};
use tilemap_renderer::types::{ BlendMode, FillRef, MipmapMode, RenderConfig, ResourceId, SamplerFilter, Topology, Transform, WrapMode };
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

/// A function that builds the commands drawing the quad at a transform.
type Draw = fn( Transform ) -> Vec< RenderCommand >;

/// Column-major identity: the backend's camera until a caller sets one.
const IDENTITY : [ f32; 9 ] = [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ];

/// Drawing-buffer size every frame is rendered at. Pinned, because a fresh
/// canvas takes its size from the page layout, which shifts as each test
/// appends one, and frames of different sizes cannot be compared.
const WIDTH : u32 = 96;
/// See [`WIDTH`].
const HEIGHT : u32 = 64;

/// Zoom by 2, then pan by `( 20, 12 )` pixels.
const CAMERA : [ f32; 9 ] = [ 2.0, 0.0, 0.0, 0.0, 2.0, 0.0, 20.0, 12.0, 1.0 ];

/// The quad every case draws: 12×9 at `( 10, 8 )`. Whole pixels, so the
/// camera's result stays on whole pixels too and the frames compare exactly.
fn quad() -> Transform
{
  Transform { position : [ 10.0, 8.0 ], scale : [ 12.0, 9.0 ], ..Transform::default() }
}

/// [`quad`] with [`CAMERA`] folded in by hand: `2 · ( 10, 8 ) + ( 20, 12 )`,
/// at twice the size.
fn quad_through_camera() -> Transform
{
  Transform { position : [ 40.0, 28.0 ], scale : [ 24.0, 18.0 ], ..Transform::default() }
}

/// A batch parent that moves its instances by `( 3, 2 )`. Applied after the
/// camera instead of before, it would move the quad by `( 3, 2 )` on screen
/// rather than `( 6, 4 )`, so the composition order shows.
const PARENT : Transform = Transform { position : [ 3.0, 2.0 ], scale : [ 1.0, 1.0 ], rotation : 0.0, depth : 0.0, skew : [ 0.0, 0.0 ] };

/// [`quad`] under [`PARENT`], then [`CAMERA`], folded in by hand:
/// `2 · ( ( 10, 8 ) + ( 3, 2 ) ) + ( 20, 12 )`, at twice the size.
fn quad_through_parent_and_camera() -> Transform
{
  Transform { position : [ 46.0, 32.0 ], scale : [ 24.0, 18.0 ], ..Transform::default() }
}

/// One 1×1 opaque white image ( id 0 ), one sprite ( id 0 ) covering it and one
/// unit quad geometry ( id 0 ).
fn white_assets() -> Assets
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
        source : ImageSource::Bitmap { bytes : vec![ 255; 4 ], width : 1, height : 1, format : PixelFormat::Rgba8 },
        filter : SamplerFilter::Nearest,
        mipmap : MipmapMode::Off,
        wrap : WrapMode::Clamp,
        premultiplied : false,
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

/// On a fresh [`WIDTH`]×[`HEIGHT`] canvas and backend: sets `camera`, clears to
/// opaque black, submits the commands `draw` builds for `transform` and returns
/// the whole frame's RGBA bytes.
fn frame( camera : [ f32; 9 ], transform : Transform, draw : Draw ) -> Vec< u8 >
{
  let gl = gl_init();
  let canvas = gl.canvas().unwrap().dyn_into::< web_sys::HtmlCanvasElement >().unwrap();
  canvas.set_width( WIDTH );
  canvas.set_height( HEIGHT );
  let config = RenderConfig { width : WIDTH, height : HEIGHT, ..RenderConfig::default() };
  let mut backend = WebGlBackend::new( config, gl.clone() ).unwrap();
  backend.assets_load( &white_assets() ).unwrap();
  backend.set_camera( camera );

  let mut commands = vec![ RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 0.0, 1.0 ] } ) ];
  commands.extend( draw( transform ) );
  backend.submit( &commands ).unwrap();

  let mut pixels = vec![ 0_u8; ( WIDTH * HEIGHT * 4 ) as usize ];
  gl.read_pixels_with_opt_u8_array( 0, 0, WIDTH as i32, HEIGHT as i32, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixels ) ).unwrap();
  pixels
}

/// Like [`frame`], but on one backend for several frames: frame `i` sets
/// `cameras[ i ]` and submits `frames[ i ]` after the clear. Returns every
/// frame's RGBA bytes. A command a frame leaves out is not replayed, so a
/// batch created in frame 0 keeps the instance data frame 0 uploaded.
fn frames_on_one_backend( cameras : &[ [ f32; 9 ] ], frames : &[ Vec< RenderCommand > ] ) -> Vec< Vec< u8 > >
{
  let gl = gl_init();
  let canvas = gl.canvas().unwrap().dyn_into::< web_sys::HtmlCanvasElement >().unwrap();
  canvas.set_width( WIDTH );
  canvas.set_height( HEIGHT );
  let config = RenderConfig { width : WIDTH, height : HEIGHT, ..RenderConfig::default() };
  let mut backend = WebGlBackend::new( config, gl.clone() ).unwrap();
  backend.assets_load( &white_assets() ).unwrap();

  cameras.iter().zip( frames ).map( | ( camera, draw ) |
  {
    backend.set_camera( *camera );
    let mut commands = vec![ RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 0.0, 1.0 ] } ) ];
    commands.extend( draw.iter().copied() );
    backend.submit( &commands ).unwrap();
    let mut pixels = vec![ 0_u8; ( WIDTH * HEIGHT * 4 ) as usize ];
    gl.read_pixels_with_opt_u8_array( 0, 0, WIDTH as i32, HEIGHT as i32, gl::RGBA, gl::UNSIGNED_BYTE, Some( &mut pixels ) ).unwrap();
    pixels
  }).collect()
}

/// How many pixels of `frame` are not the black clear.
fn lit( frame : &[ u8 ] ) -> usize
{
  frame.as_chunks::< 4 >().0.iter().filter( | p | p[ .. 3 ] != [ 0, 0, 0 ] ).count()
}

/// Draws `draw` through [`CAMERA`] and requires the frame to match the hand-folded
/// draw through the identity, and to differ from the same draw without a camera.
/// The second check keeps the first from passing because the camera was ignored.
fn assert_camera_applies( draw : Draw, path : &str )
{
  let through_camera = frame( CAMERA, quad(), draw );
  let folded_by_hand = frame( IDENTITY, quad_through_camera(), draw );
  let without_camera = frame( IDENTITY, quad(), draw );

  assert_eq!( lit( &folded_by_hand ), 24 * 18, "{path}: the reference quad covers 24×18 pixels" );
  assert!( through_camera == folded_by_hand, "{path}: the camera must move and scale the draw like the folded transform" );
  assert!( through_camera != without_camera, "{path}: the camera must change the frame" );
}

/// The quad as a solid `Mesh` command.
fn mesh( transform : Transform ) -> Vec< RenderCommand >
{
  vec!
  [
    RenderCommand::Mesh( Mesh
    {
      transform,
      geometry : ResourceId::new( 0 ),
      fill : FillRef::Solid( [ 1.0; 4 ] ),
      texture : None,
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    })
  ]
}

/// The quad as a world-space `Sprite` command.
fn sprite( transform : Transform ) -> Vec< RenderCommand >
{
  vec![ RenderCommand::Sprite( Sprite { transform, sprite : ResourceId::new( 0 ), tint : [ 1.0; 4 ], blend : BlendMode::Normal, clip : None } ) ]
}

/// The quad as a `ScreenSpaceSprite` command, in viewport pixels.
fn screen_space_sprite( transform : Transform ) -> Vec< RenderCommand >
{
  vec![ RenderCommand::ScreenSpaceSprite( Sprite { transform, sprite : ResourceId::new( 0 ), tint : [ 1.0; 4 ], blend : BlendMode::Normal, clip : None } ) ]
}

/// The quad as the one instance of a sprite batch with an identity parent, so the
/// camera reaches it through the batch's parent matrix on the GPU.
fn sprite_batch( transform : Transform ) -> Vec< RenderCommand >
{
  sprite_batch_under( Transform::default(), transform )
}

/// The quad as the one instance of a sprite batch under [`PARENT`].
fn sprite_batch_under_parent( transform : Transform ) -> Vec< RenderCommand >
{
  sprite_batch_under( PARENT, transform )
}

/// The quad as the one instance of a sprite batch whose parent is `parent`.
fn sprite_batch_under( parent : Transform, transform : Transform ) -> Vec< RenderCommand >
{
  let batch = ResourceId::new( 0 );
  let params = SpriteBatchParams
  {
    transform : parent,
    sheet : ResourceId::new( 0 ),
    blend : BlendMode::Normal,
    clip : None,
    alpha_clip : 0.0,
    occlude_overlap : false,
  };
  vec!
  [
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch { batch, params } ),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddSpriteInstance( AddSpriteInstance { transform, sprite : ResourceId::new( 0 ), tint : [ 1.0; 4 ] } ),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch } ),
  ]
}

/// The quad as the one instance of a mesh batch with an identity parent.
fn mesh_batch( transform : Transform ) -> Vec< RenderCommand >
{
  mesh_batch_under( Transform::default(), transform )
}

/// The quad as the one instance of a mesh batch under [`PARENT`].
fn mesh_batch_under_parent( transform : Transform ) -> Vec< RenderCommand >
{
  mesh_batch_under( PARENT, transform )
}

/// The quad as the one instance of a mesh batch whose parent is `parent`.
fn mesh_batch_under( parent : Transform, transform : Transform ) -> Vec< RenderCommand >
{
  let batch = ResourceId::new( 0 );
  let params = MeshBatchParams
  {
    transform : parent,
    geometry : ResourceId::new( 0 ),
    fill : FillRef::Solid( [ 1.0; 4 ] ),
    texture : None,
    topology : Topology::TriangleList,
    blend : BlendMode::Normal,
    clip : None,
  };
  vec!
  [
    RenderCommand::CreateMeshBatch( CreateMeshBatch { batch, params } ),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddMeshInstance( AddMeshInstance { transform, tint : [ 1.0; 4 ] } ),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch } ),
  ]
}

/// A `Mesh` command goes through the camera: zoomed by 2 and panned, exactly as
/// if its transform had been multiplied by the view matrix on the CPU.
#[ wasm_bindgen_test ]
fn set_camera_moves_a_mesh()
{
  assert_camera_applies( mesh, "Mesh" );
}

/// A world-space `Sprite` command goes through the camera the same way.
#[ wasm_bindgen_test ]
fn set_camera_moves_a_sprite()
{
  assert_camera_applies( sprite, "Sprite" );
}

/// A sprite batch gets the camera folded into its parent matrix, so its
/// instances move with it without their instance data being re-uploaded.
#[ wasm_bindgen_test ]
fn set_camera_moves_a_sprite_batch()
{
  assert_camera_applies( sprite_batch, "sprite batch" );
}

/// A mesh batch gets the camera folded into its parent matrix too.
#[ wasm_bindgen_test ]
fn set_camera_moves_a_mesh_batch()
{
  assert_camera_applies( mesh_batch, "mesh batch" );
}

/// A `ScreenSpaceSprite` is already in viewport pixels, so the camera leaves it
/// where it is: a UI element anchored to the viewport must not pan or zoom
/// with the world.
#[ wasm_bindgen_test ]
fn set_camera_leaves_a_screen_space_sprite()
{
  let through_camera = frame( CAMERA, quad(), screen_space_sprite );
  let without_camera = frame( IDENTITY, quad(), screen_space_sprite );

  assert_eq!( lit( &without_camera ), 12 * 9, "the quad covers 12×9 pixels" );
  assert!( through_camera == without_camera, "a screen-space sprite must ignore the camera" );
}

/// A batch's parent transform applies first and the camera after it, for both
/// batch kinds: the frame through the camera must match a plain draw of the
/// quad with parent and camera folded in by hand. The identity parent of the
/// other batch cases cannot tell the order apart.
#[ wasm_bindgen_test ]
fn set_camera_applies_after_a_batch_parent()
{
  let cases : [ ( Draw, Draw, &str ); 2 ] =
  [
    ( sprite_batch_under_parent, sprite, "sprite batch" ),
    ( mesh_batch_under_parent, mesh, "mesh batch" ),
  ];
  for ( batch, plain, path ) in cases
  {
    let through_camera = frame( CAMERA, quad(), batch );
    let folded_by_hand = frame( IDENTITY, quad_through_parent_and_camera(), plain );
    assert_eq!( lit( &folded_by_hand ), 24 * 18, "{path}: the reference quad covers 24×18 pixels" );
    assert!( through_camera == folded_by_hand, "{path}: the parent must apply first, then the camera" );
  }
}

/// A pan or zoom on a batch drawn again in a later frame takes effect with no
/// instance commands at all: frame 0 creates the batch and draws it through
/// the identity camera, frame 1 changes only the camera and issues only
/// `DrawBatch`. If the camera were baked into the uploaded instance data,
/// frame 1 would still show frame 0's quad.
#[ wasm_bindgen_test ]
fn set_camera_moves_a_retained_batch_without_re_upload()
{
  let cases : [ ( Draw, Draw, &str ); 2 ] =
  [
    ( sprite_batch, sprite, "sprite batch" ),
    ( mesh_batch, mesh, "mesh batch" ),
  ];
  for ( batch, plain, path ) in cases
  {
    let draw_again = vec![ RenderCommand::DrawBatch( DrawBatch { batch : ResourceId::new( 0 ) } ) ];
    let frames = frames_on_one_backend( &[ IDENTITY, CAMERA ], &[ batch( quad() ), draw_again ] );
    assert!( frames[ 0 ] == frame( IDENTITY, quad(), plain ), "{path}: frame 0 draws the quad where its transform puts it" );
    assert!( frames[ 1 ] == frame( IDENTITY, quad_through_camera(), plain ), "{path}: frame 1 draws it through the new camera" );
  }
}
